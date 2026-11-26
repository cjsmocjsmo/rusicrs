//! Audio delivery for `/playmusic?rusicid=...`; FLAC conversion is cached on disk.
//! Configure cache/encoding with `RUSIC_TRANSCODE_*` and the executable with `RUSIC_FFMPEG_PATH`.

use axum::{
    Json,
    body::{Body, boxed},
    extract::Query,
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE},
    },
    response::{IntoResponse, Response},
};
use md5;
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
    process::Command,
    sync::Mutex,
    time::timeout,
};
use tokio_util::io::ReaderStream;

const DEFAULT_CACHE_DIR: &str = "/var/cache/rusicrs/mp3";
const DEFAULT_FFMPEG: &str = "ffmpeg";
const DEFAULT_BITRATE: &str = "192k";
const DEFAULT_TIMEOUT_SECS: u64 = 180;
const MAX_TIMEOUT_SECS: u64 = 1800;

static CACHE_LOCKS: OnceLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
pub struct PlayMusicQuery {
    pub rusicid: String,
    pub format: Option<String>,
}

struct TranscodeConfig {
    cache_dir: PathBuf,
    ffmpeg_path: String,
    bitrate: String,
    timeout: Duration,
}

pub async fn play_music_handler(
    Query(params): Query<PlayMusicQuery>,
    headers: HeaderMap,
) -> Response {
    let wants_mp3 = match params.format.as_deref() {
        None => false,
        Some("mp3") => true,
        Some(_) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "Unsupported format; expected format=mp3",
            );
        }
    };

    let db_path = match std::env::var("RUSIC_DB_PATH") {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Missing RUSIC_DB_PATH: {}", err);
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "RUSIC_DB_PATH is not set",
            );
        }
    };

    let rusicid = params.rusicid;
    let lookup = tokio::task::spawn_blocking(move || {
        let conn = Connection::open(db_path)?;
        conn.query_row(
            "SELECT fullpath FROM music WHERE rusicid = ?1 LIMIT 1",
            [rusicid],
            |row| row.get::<_, String>(0),
        )
        .optional()
    })
    .await;

    let source_path = match lookup {
        Ok(Ok(Some(path))) => PathBuf::from(path),
        Ok(Ok(None)) => return api_error(StatusCode::NOT_FOUND, "Song not found"),
        Ok(Err(err)) => {
            eprintln!("Failed to look up song: {}", err);
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to look up song");
        }
        Err(err) => {
            eprintln!("Song lookup task failed: {}", err);
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to look up song");
        }
    };

    let source_metadata = match fs::metadata(&source_path).await {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => return api_error(StatusCode::NOT_FOUND, "Audio file not found"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return api_error(StatusCode::NOT_FOUND, "Audio file not found");
        }
        Err(err) => {
            eprintln!("Failed to inspect audio file {:?}: {}", source_path, err);
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to inspect audio file",
            );
        }
    };

    let audio_path = if wants_mp3
        && source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("flac"))
    {
        let config = match TranscodeConfig::from_environment() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Invalid transcoding configuration: {}", err);
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid transcoding configuration",
                );
            }
        };
        match cached_mp3(&source_path, &source_metadata, &config).await {
            Ok(path) => path,
            Err(err) => {
                eprintln!("Failed to transcode {:?}: {}", source_path, err);
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to transcode audio",
                );
            }
        }
    } else {
        source_path
    };

    match stream_audio(&audio_path, &headers).await {
        Ok(response) => response,
        Err(err) => {
            eprintln!("Failed to serve audio {:?}: {}", audio_path, err);
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve audio")
        }
    }
}

impl TranscodeConfig {
    fn from_environment() -> Result<Self, String> {
        let cache_dir = std::env::var_os("RUSIC_TRANSCODE_CACHE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CACHE_DIR));
        let ffmpeg_path = optional_environment_value("RUSIC_FFMPEG_PATH", DEFAULT_FFMPEG)?;
        if ffmpeg_path.is_empty() {
            return Err("RUSIC_FFMPEG_PATH must not be empty".to_string());
        }
        let bitrate = optional_environment_value("RUSIC_TRANSCODE_BITRATE", DEFAULT_BITRATE)?;
        validate_bitrate(&bitrate)?;

        let timeout_secs = match std::env::var("RUSIC_TRANSCODE_TIMEOUT_SECS") {
            Ok(value) => value
                .parse::<u64>()
                .map_err(|_| "RUSIC_TRANSCODE_TIMEOUT_SECS must be an integer".to_string())?,
            Err(std::env::VarError::NotPresent) => DEFAULT_TIMEOUT_SECS,
            Err(err) => return Err(format!("Failed to read transcode timeout: {}", err)),
        };
        if timeout_secs == 0 || timeout_secs > MAX_TIMEOUT_SECS {
            return Err(format!(
                "RUSIC_TRANSCODE_TIMEOUT_SECS must be between 1 and {}",
                MAX_TIMEOUT_SECS
            ));
        }

        Ok(Self {
            cache_dir,
            ffmpeg_path,
            bitrate,
            timeout: Duration::from_secs(timeout_secs),
        })
    }
}

fn optional_environment_value(name: &str, default: &str) -> Result<String, String> {
    match std::env::var(name) {
        Ok(value) => Ok(value),
        Err(std::env::VarError::NotPresent) => Ok(default.to_string()),
        Err(err) => Err(format!("Failed to read {}: {}", name, err)),
    }
}

fn validate_bitrate(bitrate: &str) -> Result<(), String> {
    let numeric_part = bitrate
        .strip_suffix('k')
        .or_else(|| bitrate.strip_suffix('K'))
        .ok_or_else(|| "RUSIC_TRANSCODE_BITRATE must use the form 192k".to_string())?;
    let value = numeric_part
        .parse::<u16>()
        .map_err(|_| "RUSIC_TRANSCODE_BITRATE must use the form 192k".to_string())?;
    if !(32..=320).contains(&value) {
        return Err("RUSIC_TRANSCODE_BITRATE must be between 32k and 320k".to_string());
    }
    Ok(())
}

async fn cached_mp3(
    source_path: &Path,
    source_metadata: &std::fs::Metadata,
    config: &TranscodeConfig,
) -> Result<PathBuf, String> {
    let key = cache_key(source_path, source_metadata, &config.bitrate);
    let cache_path = config.cache_dir.join(format!("{}.mp3", key));
    if cache_file_is_ready(&cache_path).await {
        return Ok(cache_path);
    }

    let locks = CACHE_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let cache_lock = {
        let mut locks = locks.lock().await;
        Arc::clone(
            locks
                .entry(cache_path.clone())
                .or_insert_with(|| Arc::new(Mutex::new(()))),
        )
    };
    let _guard = cache_lock.lock().await;

    if cache_file_is_ready(&cache_path).await {
        return Ok(cache_path);
    }

    fs::create_dir_all(&config.cache_dir)
        .await
        .map_err(|err| format!("Failed to create cache directory: {}", err))?;

    let temp_id = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_path =
        config
            .cache_dir
            .join(format!(".{}.{}.{}.tmp", key, std::process::id(), temp_id));

    let mut command = Command::new(&config.ffmpeg_path);
    command
        .arg("-nostdin")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(source_path)
        .arg("-map")
        .arg("0:a:0")
        .arg("-vn")
        .arg("-codec:a")
        .arg("libmp3lame")
        .arg("-b:a")
        .arg(&config.bitrate)
        .arg("-f")
        .arg("mp3")
        .arg(&temp_path)
        .kill_on_drop(true);

    let output = match timeout(config.timeout, command.output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(err)) => {
            remove_temp_file(&temp_path).await;
            return Err(format!("Failed to run ffmpeg: {}", err));
        }
        Err(_) => {
            remove_temp_file(&temp_path).await;
            return Err(format!(
                "ffmpeg exceeded the {} second timeout",
                config.timeout.as_secs()
            ));
        }
    };

    if !output.status.success() {
        remove_temp_file(&temp_path).await;
        return Err(format!(
            "ffmpeg exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let current_source_metadata = match fs::metadata(source_path).await {
        Ok(metadata) => metadata,
        Err(err) => {
            remove_temp_file(&temp_path).await;
            return Err(format!("Failed to re-check source file: {}", err));
        }
    };
    if cache_key(source_path, &current_source_metadata, &config.bitrate) != key {
        remove_temp_file(&temp_path).await;
        return Err("Source file changed during transcoding".to_string());
    }

    if !cache_file_is_ready(&temp_path).await {
        remove_temp_file(&temp_path).await;
        return Err("ffmpeg produced an empty or missing MP3 file".to_string());
    }

    if let Err(err) = fs::rename(&temp_path, &cache_path).await {
        remove_temp_file(&temp_path).await;
        return Err(format!("Failed to publish cached MP3: {}", err));
    }

    Ok(cache_path)
}

async fn remove_temp_file(path: &Path) {
    if let Err(err) = fs::remove_file(path).await {
        if err.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "Failed to remove temporary transcode file {:?}: {}",
                path, err
            );
        }
    }
}

fn cache_key(source_path: &Path, metadata: &std::fs::Metadata, bitrate: &str) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let fingerprint = format!(
        "{}:{}:{}:{}",
        source_path.to_string_lossy(),
        metadata.len(),
        modified,
        bitrate
    );
    format!("{:x}", md5::compute(fingerprint.as_bytes()))
}

async fn cache_file_is_ready(path: &Path) -> bool {
    matches!(fs::metadata(path).await, Ok(metadata) if metadata.is_file() && metadata.len() > 0)
}

async fn stream_audio(path: &Path, headers: &HeaderMap) -> Result<Response, String> {
    let metadata = fs::metadata(path)
        .await
        .map_err(|err| format!("Failed to inspect audio file: {}", err))?;
    let file_size = metadata.len();

    let range = match headers.get(RANGE) {
        Some(value) => match value.to_str() {
            Ok(value) => match parse_byte_range(value, file_size) {
                Ok(range) => range,
                Err(()) => return Ok(range_not_satisfiable(file_size)),
            },
            Err(_) => return Ok(range_not_satisfiable(file_size)),
        },
        None => None,
    };

    let (status, start, end) = match range {
        Some((start, end)) => (StatusCode::PARTIAL_CONTENT, start, end),
        None => (StatusCode::OK, 0, file_size.saturating_sub(1)),
    };
    let content_length = if file_size == 0 { 0 } else { end - start + 1 };

    let mut file = File::open(path)
        .await
        .map_err(|err| format!("Failed to open audio file: {}", err))?;
    if start > 0 {
        file.seek(SeekFrom::Start(start))
            .await
            .map_err(|err| format!("Failed to seek audio file: {}", err))?;
    }

    let stream = ReaderStream::with_capacity(file.take(content_length), 64 * 1024);
    let body = Body::wrap_stream(stream);
    let mut response = Response::new(boxed(body));
    *response.status_mut() = status;
    response
        .headers_mut()
        .insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(content_type(path)));
    let content_length = HeaderValue::from_str(&content_length.to_string())
        .map_err(|err| format!("Invalid content length: {}", err))?;
    response
        .headers_mut()
        .insert(CONTENT_LENGTH, content_length);
    if let Some((start, end)) = range {
        let content_range =
            HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, file_size))
                .map_err(|err| format!("Invalid content range: {}", err))?;
        response.headers_mut().insert(CONTENT_RANGE, content_range);
    }

    Ok(response)
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" | "oga" => "audio/ogg",
        "m4a" | "mp4" => "audio/mp4",
        "wav" => "audio/wav",
        "aac" => "audio/aac",
        _ => "application/octet-stream",
    }
}

fn parse_byte_range(value: &str, file_size: u64) -> Result<Option<(u64, u64)>, ()> {
    let value = value.strip_prefix("bytes=").ok_or(())?;
    if value.contains(',') || file_size == 0 {
        return Err(());
    }
    let (start, end) = value.split_once('-').ok_or(())?;

    if start.is_empty() {
        let suffix_length = end.parse::<u64>().map_err(|_| ())?;
        if suffix_length == 0 {
            return Err(());
        }
        let start = file_size.saturating_sub(suffix_length);
        return Ok(Some((start, file_size - 1)));
    }

    let start = start.parse::<u64>().map_err(|_| ())?;
    if start >= file_size {
        return Err(());
    }
    let end = if end.is_empty() {
        file_size - 1
    } else {
        end.parse::<u64>().map_err(|_| ())?.min(file_size - 1)
    };
    if end < start {
        return Err(());
    }
    Ok(Some((start, end)))
}

fn range_not_satisfiable(file_size: u64) -> Response {
    let mut response = Response::new(boxed(Body::empty()));
    *response.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
    response
        .headers_mut()
        .insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    match HeaderValue::from_str(&format!("bytes */{}", file_size)) {
        Ok(content_range) => {
            response.headers_mut().insert(CONTENT_RANGE, content_range);
            response
        }
        Err(err) => {
            eprintln!("Failed to build 416 response: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn api_error(status: StatusCode, message: &'static str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::{parse_byte_range, stream_audio, validate_bitrate};
    use axum::http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CONTENT_LENGTH, CONTENT_RANGE, RANGE},
    };
    use std::{env, fs, path::PathBuf};

    #[test]
    fn parses_closed_open_and_suffix_ranges() {
        assert_eq!(parse_byte_range("bytes=2-5", 10), Ok(Some((2, 5))));
        assert_eq!(parse_byte_range("bytes=7-", 10), Ok(Some((7, 9))));
        assert_eq!(parse_byte_range("bytes=-4", 10), Ok(Some((6, 9))));
        assert_eq!(parse_byte_range("bytes=2-50", 10), Ok(Some((2, 9))));
    }

    #[test]
    fn rejects_unsatisfiable_and_multiple_ranges() {
        assert_eq!(parse_byte_range("bytes=10-", 10), Err(()));
        assert_eq!(parse_byte_range("bytes=5-2", 10), Err(()));
        assert_eq!(parse_byte_range("bytes=-0", 10), Err(()));
        assert_eq!(parse_byte_range("bytes=0-1,4-5", 10), Err(()));
        assert_eq!(parse_byte_range("bytes=0-", 0), Err(()));
    }

    #[test]
    fn validates_mp3_bitrates() {
        assert!(validate_bitrate("192k").is_ok());
        assert!(validate_bitrate("320K").is_ok());
        assert!(validate_bitrate("0k").is_err());
        assert!(validate_bitrate("192").is_err());
    }

    #[tokio::test]
    async fn streams_partial_content_headers() {
        let path: PathBuf =
            env::temp_dir().join(format!("rusicrs-audio-range-{}.mp3", std::process::id()));
        fs::write(&path, b"0123456789").expect("write fixture");

        let mut headers = HeaderMap::new();
        headers.insert(RANGE, HeaderValue::from_static("bytes=2-5"));
        let response = stream_audio(&path, &headers)
            .await
            .expect("serve byte range");

        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            response.headers().get(CONTENT_RANGE).unwrap(),
            "bytes 2-5/10"
        );
        assert_eq!(response.headers().get(CONTENT_LENGTH).unwrap(), "4");
        fs::remove_file(path).expect("remove fixture");
    }
}
