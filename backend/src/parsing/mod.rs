use serde::Deserialize;

pub mod zip;

pub struct FileData {
    pub level_data: LevelData,
    pub image: Option<Vec<u8>>
}

#[derive(Deserialize)]
pub struct LevelData {
    pub metadata: LevelMetadata,
}

#[derive(Deserialize)]
pub struct LevelMetadata {
    pub artist: String,
    pub charter: String,
    pub difficulty: f32,
    pub description: String,
    #[serde(rename = "songName")]
    pub song_name: String,
    #[serde(rename = "artistList")]
    #[serde(default)]
    pub artist_list: String,
    #[serde(rename = "bgData")]
    #[serde(default)]
    pub bg_data: Option<BackgroundData>,
}

#[derive(Deserialize)]
pub struct BackgroundData {
    image: String,
}