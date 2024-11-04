use crate::database::{BeatMap, User};
use crate::parsing::{check_archive, get_parser, parse_archive, FileData};
use crate::ratelimiter::{Ratelimiter, SiteAction, UniqueIdentifier};
use crate::search::SearchArguments;
use crate::{LockResultExt, SiteData};
use anyhow::Error;
use chrono::{DateTime, Utc};
use firebase_auth::FirebaseUser;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::header::CONTENT_TYPE;
use hyper::{Request, StatusCode};
use image::{ImageFormat, ImageReader};
use multer::{Constraints, Field, Multipart, SizeLimit};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use surrealdb::engine::remote::ws::Client;
use surrealdb::opt::PatchOp;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use thiserror::Error;
use tokio::time::error::Elapsed;
use tokio::time::timeout;
use uuid::Uuid;

pub const MAX_SIZE: u32 = 200000000;
const SUPPORTED_FORMATS: [ImageFormat; 3] = [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Bmp];

#[derive(Error, Debug)]
pub enum UploadError {
    #[error("Expected a multi-part form!")]
    ArgumentError(),
    #[error("Unknown archive type, please submit a zip or rar!")]
    ArchiveTypeError(),
    #[error("Error with multi-part form!")]
    KnownArgumentError(Error),
    #[error("Database error")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("Authentication error")]
    AuthError(String),
    #[error("Unknown database error")]
    UnknownDatabaseError(String),
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
    #[error("Ratelimited")]
    Ratelimited()
}

impl UploadError {
    pub fn get_code(&self) -> StatusCode {
        match self {
            UploadError::DatabaseError(_) | UploadError::UnknownDatabaseError(_) | UploadError::ZipError(_) |
            UploadError::IOError(_) | UploadError::TimeoutError(_) | UploadError::ZipDownloadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UploadError::ArgumentError() | UploadError::KnownArgumentError(_) | UploadError::AuthError(_) | UploadError::FormError(_) |
            UploadError::SongNameError(_) | UploadError::ArchiveTypeError() => StatusCode::BAD_REQUEST,
            UploadError::Ratelimited() => StatusCode::TOO_MANY_REQUESTS
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
pub struct UserMapUpdate {
    maps: Vec<Thing>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserId {
    id: Thing,
}

pub async fn upload(request: Request<Incoming>, identifier: UniqueIdentifier, data: &SiteData) -> Result<String, UploadError> {
    if data.ratelimiter.lock().ignore_poison().check_limited(SiteAction::Update, &identifier) {
        return Err(UploadError::Ratelimited());
    }
    let form = get_form(request).await?;
    let user: FirebaseUser = data.auth.verify(&form.firebase_token).map_err(|err| UploadError::AuthError(err.to_string()))?;
    let id = get_or_create_user(data.db.query(format!("SELECT id FROM users WHERE google_id == '{}'", user.user_id))
        .await?.take::<Option<UserId>>(0)?, &data.db, User {
        google_id: Some(user.user_id),
        ..Default::default()
    }).await?;
    timeout(Duration::from_millis(1000), upload_beatmap(form.beatmap, &data.db, &data.ratelimiter, identifier, id)).await?
}

pub async fn get_or_create_user(id: Option<UserId>, db: &Surreal<Client>, default_user: User) -> Result<Thing, UploadError> {
    Ok(if let Some(id) = id {
        id.id
    } else {
        let Some(user): Option<User> = db.create("users").content(default_user).await? else {
            return Err(UploadError::UnknownDatabaseError("Failed to create a user in the users database".to_string()));
        };
        user.id.unwrap()
    })
}

pub async fn upload_beatmap(mut beatmap: Vec<u8>, db: &Surreal<Client>, ratelimiter: &Arc<Mutex<Ratelimiter>>,
                            ip: UniqueIdentifier, charter_id: Thing) -> Result<String, UploadError> {
    let mut file_data = parse_archive(get_parser(&mut beatmap)?.deref_mut()).map_err(UploadError::ZipError)?;
    check_archive(&mut beatmap).map_err(UploadError::ZipError)?;

    let uuid = Uuid::new_v4();
    let path = PathBuf::from("site/output");

    save_image(&mut file_data, &path, &uuid)?;

    // Create the beatmap
    fs::write(path.join(format!("{}.zip", uuid)), beatmap)?;
    let query = serde_urlencoded::to_string(&SearchArguments {
        query: file_data.level_data.song_name.clone()
    }).map_err(|err| UploadError::SongNameError(err))?;

    let name = file_data.level_data.song_name.clone();
    let beatmap = BeatMap {
        song: file_data.level_data.song_name,
        artist: file_data.level_data.artist,
        charter: file_data.level_data.charter,
        difficulties: file_data.level_data.difficulty.map_or(file_data.level_data.variants, |diff| vec![diff.into()]),
        description: file_data.level_data.description,
        artist_list: file_data.level_data.artist_list,
        charter_uid: Some(charter_id.to_string()),
        image: file_data.image.as_ref().map(|_| format!("{}.png", uuid)),
        download: format!("{}.zip", uuid),
        upvotes: 0,
        upload_date: DateTime::from(SystemTime::now()),
        update_date: DateTime::from(SystemTime::now()),
        id: None,
    };

    // Save the beatmap
    if let Ok(Some(mut map)) = db.query(format!("SELECT * FROM beatmaps WHERE charter_uid == '{}' and song == $name", charter_id.to_string()))
        .bind(("name", name)).await?.take::<Option<BeatMap>>(0) {
        // Update the old map instead
        map.upload_date = DateTime::from(SystemTime::now());
        let found_id = map.id.clone().unwrap().id.to_string();
        let new_id = &found_id[3..found_id.len()-3];
        let Some(_): Option<BeatMap> = db.update(("beatmaps", new_id))
            .patch(PatchOp::replace("update_date", DateTime::<Utc>::from(SystemTime::now()))).await? else {
            return Err(UploadError::UnknownDatabaseError("Failed to update the map timestamp".to_string()));
        };
    } else {
        if ratelimiter.lock().ignore_poison().check_limited(SiteAction::Upload, &ip) {
            return Err(UploadError::Ratelimited());
        }

        let map: Thing = ("beatmaps", uuid.to_string().as_str()).into();
        let Some(_): Option<User> = db.update(("users", charter_id.id.to_string())).merge(UserMapUpdate {
            maps: vec![map.clone()]
        }).await? else {
            return Err(UploadError::UnknownDatabaseError("Failed to update the user's maps".to_string()));
        };
        let Some(_): Option<BeatMap> = db.create(("beatmaps", uuid.to_string())).content(beatmap).await? else {
            return Err(UploadError::UnknownDatabaseError("Failed to create the beatmap".to_string()));
        };
    }
    Ok(query)
}

pub fn save_image(data: &mut FileData, path: &PathBuf, uuid: &Uuid) -> Result<(), UploadError> {
    if let Some(ref image) = data.image {
        if image.is_empty() {
            data.image = None;
        } else {
            let reader = ImageReader::new(Cursor::new(image)).with_guessed_format()?;
            if !reader.format().is_some_and(|format| SUPPORTED_FORMATS.contains(&format)) {
                return Err(UploadError::ZipError(Error::msg(
                    format!("Unknown or unsupported background image format, please use png, bmp or jpeg! {:?}", reader.format()))))
            }
            match reader.decode() {
                Ok(image) => {
                    image.save_with_format(path.join(format!("{uuid}.png")), ImageFormat::Png).unwrap();
                }
                Err(err) => {
                    return Err(UploadError::ZipError(Error::from(err)));
                }
            }
        }
    }
    Ok(())
}

pub async fn get_form(request: Request<Incoming>) -> Result<UploadForm, UploadError> {
    let header = request.headers().get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok())
        .map(|inner| Ok(inner))
        .unwrap_or(Err(UploadError::ArgumentError()))?;

    let mut form = UploadForm::default();
    let mut multipart = Multipart::with_constraints(request.into_body().into_data_stream(), header,
                                                    Constraints::new().allowed_fields(vec!("beatmap", "firebaseToken"))
                                                        .size_limit(SizeLimit::new().whole_stream(MAX_SIZE as u64 + 5000)));
    while let Some(mut field) = multipart.next_field().await.map_err(|err| UploadError::KnownArgumentError(err.into()))? {
        if let Some(name) = field.name() {
            match name {
                "beatmap" => {
                    form.beatmap = read_field(&mut field).await?;
                },
                "firebaseToken" => form.firebase_token = field.text().await
                    .map_err(|err| UploadError::KnownArgumentError(err.into()))?,
                _ => return Err(UploadError::ArgumentError())
            }
        }
    }
    Ok(form)
}

async fn read_field(field: &mut Field<'_>) -> Result<Vec<u8>, UploadError> {
    let mut buf = Vec::new();
    while let Some(chunk) = field.chunk().await? {
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}