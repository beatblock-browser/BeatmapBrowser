use crate::database::{BeatMap, User};
use crate::parsing::zip::read_zip;
use anyhow::Error;
use chrono::DateTime;
use firebase_auth::{FirebaseAuth, FirebaseUser};
use http_body_util::BodyExt;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, StatusCode};
use multer::Multipart;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use thiserror::Error;
use uuid::Uuid;
use crate::search::SearchArguments;

#[derive(Error, Debug)]
pub enum UploadError {
    #[error("Expected a multi-part form!")]
    ArgumentError(),
    #[error("Database error")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Authentication error")]
    AuthError(String),
    #[error("Unknown database error")]
    UnknownDatabaseError(),
    #[error("Zip error")]
    ZipError(Error),
    #[error("Form error")]
    FormError(#[from] multer::Error),
    #[error("Invalid song name")]
    SongNameError(#[from] serde_urlencoded::ser::Error)
}

impl UploadError {
    pub fn get_code(&self) -> StatusCode {
        match self {
            UploadError::DatabaseError(_) | UploadError::UnknownDatabaseError() | UploadError::ZipError(_) |
            UploadError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UploadError::ArgumentError() | UploadError::AuthError(_) | UploadError::FormError(_) |
            UploadError::SongNameError(_) => StatusCode::BAD_REQUEST
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct UploadForm {
    #[serde(rename = "firebaseToken")]
    firebase_token: String,
    beatmap: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapUpdate {
    maps: Vec<Thing>,
}

pub async fn upload(request: Request<hyper::body::Incoming>, db: Surreal<Client>, auth: FirebaseAuth) -> Result<String, UploadError> {
    let mut form = get_form(request).await?;
    let user: FirebaseUser = auth.verify(&form.firebase_token).map_err(|err| UploadError::AuthError(err.to_string()))?;
    let data = read_zip(&mut form.beatmap).map_err(|err| UploadError::ZipError(err))?;
    let uuid = Uuid::new_v4();
    let path = PathBuf::from("backend/site/output");
    if let Some(ref image) = data.image {
        fs::write(path.join(format!("{}.png", uuid)), image)?;
    }
    fs::write(path.join(format!("{}.zip", uuid)), form.beatmap)?;
    let query = serde_urlencoded::to_string(&SearchArguments {
        query: data.level_data.metadata.song_name.clone()
    }).map_err(|err| UploadError::SongNameError(err))?;
    let beatmap = BeatMap {
        song: data.level_data.metadata.song_name,
        artist: data.level_data.metadata.artist,
        charter: data.level_data.metadata.charter,
        charter_uid: Some(user.user_id.clone()),
        difficulty: data.level_data.metadata.difficulty,
        description: data.level_data.metadata.description,
        artist_list: data.level_data.metadata.artist_list,
        image: data.image.as_ref().map(|_| format!("{}.png", uuid)),
        download: format!("{}.zip", uuid),
        upvotes: 0,
        upload_date: DateTime::from(SystemTime::now()),
        id: None,
    };
    let map: Thing = ("beatmaps", uuid.to_string().as_str()).into();
    let updated: Option<User> = db.update(("users", user.user_id.clone())).merge(MapUpdate {
        maps: vec![map.clone()]
    }).await?;
    match updated {
        Some(_) => {},
        None => {
            // Make sure the user exists
            let Some(_): Option<User> = db.create(("users", user.user_id)).content(User {
                maps: vec![map],
                ..Default::default()
            }).await? else {
                return Err(UploadError::UnknownDatabaseError());
            };
        }
    };
    let Some(_): Option<BeatMap> = db.create(("beatmaps", uuid.to_string())).content(beatmap).await? else {
        return Err(UploadError::UnknownDatabaseError());
    };
    Ok(query)
}

pub async fn get_form(request: Request<hyper::body::Incoming>) -> Result<UploadForm, UploadError> {
    let header = request.headers().get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok())
        .map(|inner| Ok(inner))
        .unwrap_or(Err(UploadError::ArgumentError()))?;

    let bytes = request.into_body().into_data_stream();
    let mut form = UploadForm::default();
    let mut multipart = Multipart::new(bytes, header);
    while let Some(mut field) = multipart.next_field().await? {
        let Some(name) = field.name() else {
            return Err(UploadError::ArgumentError());
        };
        match name {
            "beatmap" => {
                while let Some(chunk) = field.chunk().await? {
                    form.beatmap.extend_from_slice(&chunk);
                }
            }
            "firebaseToken" => {
                let mut token_buf = Vec::new();
                while let Some(chunk) = field.chunk().await? {
                    token_buf.extend_from_slice(&chunk);
                }
                form.firebase_token = match String::from_utf8(token_buf) {
                    Ok(token) => token,
                    Err(_) => return Err(UploadError::ArgumentError())
                }
            }
            _ => return Err(UploadError::ArgumentError())
        }
    }
    Ok(form)
}