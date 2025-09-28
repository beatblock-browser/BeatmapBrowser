use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::error::Elapsed;
use uuid::Uuid;

pub type UserID = Uuid;
pub type MapID = Uuid;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BeatMap {
    pub song: String,
    pub artist: String,
    pub charter: String,
    pub charter_uid: UserID,
    pub difficulties: Vec<LevelVariant>,
    pub description: String,
    pub artist_list: String,
    pub image: bool,
    pub upvotes: u64,
    pub upload_date: DateTime<Utc>,
    pub update_date: DateTime<Utc>,
    pub id: MapID,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct User {
    pub maps: Vec<MapID>,
    pub downloaded: Vec<MapID>,
    pub upvoted: Vec<MapID>,
    pub id: UserID,
    pub links: Vec<AccountLink>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountLink {
    #[serde(rename = "discord")]
    Discord(u64),
    #[serde(rename = "google")]
    Google(String)
}

impl AccountLink {
    pub fn id(&self) -> String {
        match self {
            AccountLink::Discord(id) => id.to_string(),
            AccountLink::Google(id) => id.clone()
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct LevelVariant {
    pub display: String,
    pub difficulty: f64,
}


#[derive(Error, Debug)]
pub enum APIError {
    #[error("Ratelimited")]
    Ratelimited(),
    #[error("Authentication error")]
    AuthError(String),
    #[error("Database error")]
    DatabaseError(Error),
    #[error("Deserialization error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Already upvoted!")]
    AlreadyUpvoted(),
    #[error("Already downloaded!")]
    AlreadyDownloaded(),
    #[error("Expected a multi-part form!")]
    ArgumentError(),
    #[error("Unknown archive type, please submit a zip or rar!")]
    ArchiveTypeError(),
    #[error("Error with multi-part form!")]
    KnownArgumentError(Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Zip error, please confirm your beatmap file is correct and contains all needed files")]
    ZipError(Error),
    #[error("Zip download error")]
    ZipDownloadError(#[from] serenity::Error),
    #[error("Invalid song name")]
    SongNameError(#[from] serde_urlencoded::ser::Error),
    #[error("Served timed out reading archive")]
    TimeoutError(#[from] Elapsed),
    #[error("You do not have permission to perform this action")]
    PermissionError()
}

impl APIError {
    pub fn database_error<E: Into<Error>>(error: E) -> APIError {
        APIError::DatabaseError(error.into())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserToken {
    pub id: UserID,
    #[serde(rename = "user_token")]
    token: String,
}