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
    pub songName: String,
    #[serde(default)]
    pub artistList: String,
    #[serde(default)]
    pub bgData: Option<BackgroundData>,
}

#[derive(Deserialize)]
pub struct BackgroundData {
    image: String,
}