use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomArt {
	pub AlbumId: String,
	pub HttpThumbPath: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
	pub Idx: String,
	pub Path: String,
	pub MusicId: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistCount {
    pub alpha: String,
    pub count: i64,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumCount {
    pub alpha: String,
    pub count: i64,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongCount {
    pub alpha: String,
    pub count: i64,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AlbumSongs {
    pub page: i32,
    pub albumid: String,
    pub rusicids: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtistAlbums {
    pub page: i32,
    pub artistid: String,
    pub albums: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq)]
pub struct ArtArtidInfo {
    pub rusticid: String,
    pub artist: String,
    pub artistid: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq)]
pub struct AlbAlbidInfo {
    pub rusticid: String,
    pub imageurl: String,
    pub albumid: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MusicImageInfo {
    pub rusicid: String,
    pub width: String,
    pub height: String,
    pub artist: String,
    pub artistid: String,
    pub album: String,
    pub albumid: String,
    pub filesize: String,
    pub fullpath: String,
    pub thumbpath: String,
    pub idx: String,
    pub page: String,
    pub httpthumbpath: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicInfo {
    pub rusicid: String,
    pub imgurl: String,
    pub playpath: String,
    pub artist: String,
    pub artistid: String,
    pub album: String,
    pub albumid: String,
    pub song: String,
    pub fullpath: String,
    pub extension: String,
    pub idx: String,
    pub page: String,
    pub fsizeresults: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstLetterInfo {
    pub rusicid: String,
    pub artist: String,
    pub album: String,
    pub artistid: String,
    pub albumid: String,
    pub song: String,
    pub artist_first_letter: String,
    pub album_first_letter: String,
    pub song_first_letter: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayList {
    pub rusicid: String,
    pub name: String,
    pub songs: String,
    pub numsongs: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub artistcount: String,
    pub albumcount: String,
    pub songcount: String,
    pub imagecount: String,
}