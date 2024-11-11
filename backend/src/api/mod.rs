use crate::util::database::{BeatMap, User};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::{collect_stream, get_user, LockResultExt};
use crate::SiteData;
use anyhow::Error;
use firebase_auth::FirebaseUser;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;
use tokio::time::error::Elapsed;

pub mod account_data;
pub mod delete;
pub mod downloaded;
pub mod search;
pub mod upload;
pub mod upvote;
pub mod usersongs;

#[derive(Serialize, Deserialize)]
pub struct AuthenticatedRequest {
    #[serde(rename = "firebaseToken")]
    pub firebase_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct MapRequest {
    #[serde(rename = "firebaseToken")]
    pub firebase_token: String,
    #[serde(rename = "mapId")]
    pub map_id: String,
}

async fn get_map_request(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
    action: SiteAction,
) -> Result<(BeatMap, User, (String, String)), APIError> {
    if data
        .ratelimiter
        .lock()
        .ignore_poison()
        .check_limited(action, &identifier)
    {
        return Err(APIError::Ratelimited());
    }

    let request_data = collect_stream(request.into_data_stream(), 5000)
        .await
        .map_err(|err| APIError::QueryError(err))?;
    let string = String::from_utf8_lossy(request_data.deref());
    let arguments = serde_json::from_str::<MapRequest>(string.deref())
        .map_err(|err| APIError::QueryError(err.into()))?;

    let user: FirebaseUser = data
        .auth
        .verify(&arguments.firebase_token)
        .map_err(|err| APIError::AuthError(err.to_string()))?;
    let user = get_user(true, user.user_id, &data.db).await?;

    let Some(map): Option<BeatMap> = data
        .db
        .select(("beatmaps", arguments.map_id.as_str()))
        .await
        .map_err(APIError::database_error)?
    else {
        return Err(APIError::UnknownMap());
    };

    Ok((map, user, ("beatmaps".to_string(), arguments.map_id)))
}

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Ratelimited")]
    Ratelimited(),
    #[error("Invalid request")]
    QueryError(Error),
    #[error("Authentication error")]
    AuthError(String),
    #[error("Database error")]
    DatabaseError(Error),
    #[error("Unknown database error")]
    UnknownDatabaseError(String),
    #[error("Deserialization error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Already upvoted!")]
    AlreadyUpvoted(),
    #[error("Already downloaded!")]
    AlreadyDownloaded(),
    #[error("Unknown map!")]
    UnknownMap(),
    #[error("Expected a multi-part form!")]
    ArgumentError(),
    #[error("Unknown archive type, please submit a zip or rar!")]
    ArchiveTypeError(),
    #[error("Error with multi-part form!")]
    KnownArgumentError(Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Zip error")]
    ZipError(Error),
    #[error("Zip download error")]
    ZipDownloadError(#[from] serenity::Error),
    #[error("Form error")]
    FormError(#[from] multer::Error),
    #[error("Invalid song name")]
    SongNameError(#[from] serde_urlencoded::ser::Error),
    #[error("Served timed out reading archive")]
    TimeoutError(#[from] Elapsed),
    #[error("You do not have permission to perform this action")]
    PermissionError()
}

impl APIError {
    pub fn get_code(&self) -> StatusCode {
        match self {
            APIError::Ratelimited() => StatusCode::TOO_MANY_REQUESTS,
            APIError::QueryError(_)
            | APIError::AuthError(_)
            | APIError::AlreadyUpvoted()
            | APIError::AlreadyDownloaded()
            | APIError::UnknownMap()
            | APIError::ArgumentError()
            | APIError::KnownArgumentError(_)
            | APIError::FormError(_)
            | APIError::SongNameError(_)
            | APIError::ArchiveTypeError()
            | APIError::PermissionError() => StatusCode::BAD_REQUEST,
            APIError::DatabaseError(_)
            | APIError::UnknownDatabaseError(_)
            | APIError::SerdeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::ZipError(_)
            | APIError::IOError(_)
            | APIError::TimeoutError(_)
            | APIError::ZipDownloadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn database_error<E: Into<Error>>(error: E) -> APIError {
        APIError::DatabaseError(error.into())
    }
}
