use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::path::{Component, PathBuf};
use crate::parsing::rar::RarArchiveReader;
use crate::parsing::zip::ZipArchiveReader;
use crate::upload::UploadError;

pub mod zip;
pub mod rar;

pub struct FileData {
    pub level_data: LevelMetadata,
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
    pub variants: Vec<LevelVariant>
}

#[derive(Deserialize)]
pub struct BackgroundData {
    image: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LevelVariant {
    display: String,
    difficulty: f64
}

impl Into<LevelVariant> for f64 {
    fn into(self) -> LevelVariant {
        LevelVariant {
            display: get_difficulty(self),
            difficulty: self
        }
    }
}

fn get_difficulty(difficulty: f64) -> String {
    match difficulty {
        ..=0.0 => "Special",
        ..=5.0 => "Easy",
        ..=10.0 => "Hard",
        ..=15.0 => "Challenge",
        _ => "Apocrypha"
    }.to_string()
}

pub fn check_path(path: &PathBuf) -> Result<(), Error> {
    if path.components().any(|component| component == Component::ParentDir) {
        Err(Error::msg("File path contains directory traversal"))
    } else {
        Ok(())
    }
}

pub fn get_parser<'a>(beatmap: &'a mut Vec<u8>) -> Result<Box<dyn ArchiveParser + 'a>, UploadError> {
    Ok(if beatmap.starts_with("PK".as_bytes()) {
        Box::new(ZipArchiveReader::new(beatmap).map_err(|err| UploadError::ZipError(err))?)
    } else {
        Box::new(RarArchiveReader::new(beatmap).map_err(|err| UploadError::ZipError(err))?)
    })
}

pub fn parse_archive(archive_parser: &mut dyn ArchiveParser) -> Result<FileData, Error> {
    let data = archive_parser.fetch_file("level.json")
        .and_then(|data| serde_json::from_slice::<LevelData>(&data).map_err(Error::new))
        .or_else(|_| archive_parser.fetch_file("manifest.json")
                .and_then(|data| serde_json::from_slice(&data).map_err(Error::new)))?;

    let metadata = data.metadata;
    let mut image = None;
    if let Some(bg_data) = metadata.bg_data.as_ref() {
        if !bg_data.image.is_empty() {
            image = archive_parser.fetch_file(&bg_data.image).map_or(None, Some);
        }
    }

    archive_parser.overwrite_file()?;
    Ok(FileData {
        level_data: metadata,
        image
    })
}

pub trait ArchiveParser {
    fn fetch_file(&self, target_file_name: &str) -> Result<Vec<u8>, Error>;

    fn overwrite_file(&mut self) -> Result<(), Error>;
}