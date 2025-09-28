use serde::{Deserialize, Serialize};
use crate::parsing::get_difficulty;

#[derive(Deserialize)]
pub struct LevelData {
    pub metadata: LevelMetadata,
}

#[derive(Deserialize)]
pub struct LevelMetadata {
    pub artist: String,
    pub charter: String,
    pub difficulty: Option<f64>,
    pub description: String,
    #[serde(rename = "songName")]
    pub song_name: String,
    #[serde(rename = "artistList")]
    #[serde(default)]
    pub artist_list: String,
    #[serde(rename = "bgData")]
    #[serde(default)]
    pub bg_data: Option<BackgroundData>,
    #[serde(default)]
    pub variants: Vec<LevelVariant>,
}

#[derive(Deserialize)]
pub struct BackgroundData {
    pub image: String,
    #[serde(rename = "cyanChannel")]
    pub cyan_channel: Option<ColorChannel>,
    #[serde(rename = "magentaChannel")]
    pub magenta_channel: Option<ColorChannel>,
    #[serde(rename = "yellowChannel")]
    pub yellow_channel: Option<ColorChannel>,
    #[serde(rename = "redChannel")]
    pub red_channel: Option<ColorChannel>,
    #[serde(rename = "greenChannel")]
    pub green_channel: Option<ColorChannel>,
    #[serde(rename = "blueChannel")]
    pub blue_channel: Option<ColorChannel>,
}

#[derive(Deserialize)]
pub struct ColorChannel {
    #[serde(rename = "r")]
    pub red: u8,
    #[serde(rename = "g")]
    pub green: u8,
    #[serde(rename = "b")]
    pub blue: u8
}

impl From<&ColorChannel> for [u8; 3] {
    fn from(val: &ColorChannel) -> Self {
        [val.red, val.green, val.blue]
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LevelVariant {
    display: String,
    difficulty: f64,
}

impl From<f64> for LevelVariant {
    fn from(val: f64) -> Self {
        LevelVariant {
            display: get_difficulty(val),
            difficulty: val,
        }
    }
}
