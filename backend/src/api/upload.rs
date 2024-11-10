use crate::api::search::SearchArguments;
use crate::api::APIError;
use crate::parsing::{check_archive, get_parser, parse_archive, FileData};
use crate::util::database::{BeatMap, User};
use crate::util::ratelimiter::{Ratelimiter, SiteAction, UniqueIdentifier};
use crate::util::{get_beatmap_id, get_user, LockResultExt};
use crate::SiteData;
use anyhow::Error;
use chrono::{DateTime, Utc};
use firebase_auth::FirebaseUser;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::header::CONTENT_TYPE;
use hyper::Request;
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
use tokio::time::timeout;
use uuid::Uuid;

pub const MAX_SIZE: u32 = 200000000;
const SUPPORTED_FORMATS: [ImageFormat; 3] = [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Bmp];

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

pub async fn upload(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    if data
        .ratelimiter
        .lock()
        .ignore_poison()
        .check_limited(SiteAction::Update, &identifier)
    {
        return Err(APIError::Ratelimited());
    }
    let form = get_form(request).await?;
    let user: FirebaseUser = data
        .auth
        .verify(&form.firebase_token)
        .map_err(|err| APIError::AuthError(err.to_string()))?;
    let id = get_user(true, user.user_id, &data.db).await?;
    let map = timeout(
        Duration::from_millis(1000),
        upload_beatmap(
            form.beatmap,
            &data.db,
            &data.ratelimiter,
            identifier,
            id.id.unwrap(),
        ),
    )
        .await??;

    serde_urlencoded::to_string(&SearchArguments {
        query: map.song.clone(),
    })
        .map_err(|err| APIError::SongNameError(err))
}

pub async fn upload_beatmap(
    mut beatmap: Vec<u8>,
    db: &Surreal<Client>,
    ratelimiter: &Arc<Mutex<Ratelimiter>>,
    ip: UniqueIdentifier,
    charter_id: Thing,
) -> Result<BeatMap, APIError> {
    let mut file_data =
        parse_archive(get_parser(&mut beatmap)?.deref_mut()).map_err(APIError::ZipError)?;
    check_archive(&mut beatmap).map_err(APIError::ZipError)?;

    let uuid = Uuid::new_v4();
    let path = PathBuf::from("site/output");

    save_image(&mut file_data, &path, &uuid)?;

    // Create the beatmap
    fs::write(path.join(format!("{}.zip", uuid)), beatmap)?;

    let name = file_data.level_data.song_name.clone();
    let beatmap = BeatMap {
        song: file_data.level_data.song_name,
        artist: file_data.level_data.artist,
        charter: file_data.level_data.charter,
        difficulties: file_data
            .level_data
            .difficulty
            .map_or(file_data.level_data.variants, |diff| vec![diff.into()]),
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
    Ok(if let Ok(Some(mut map)) = db
        .query(format!(
            "SELECT * FROM beatmaps WHERE charter_uid == '{}' and song == $name",
            charter_id.to_string()
        ))
        .bind(("name", name.clone()))
        .await
        .map_err(APIError::database_error)?
        .take::<Option<BeatMap>>(0)
    {
        // Update the old map instead
        map.upload_date = DateTime::from(SystemTime::now());
        let Some(map): Option<BeatMap> = db
            .update(get_beatmap_id(map.id.as_ref().unwrap()))
            .patch(PatchOp::replace(
                "update_date",
                DateTime::<Utc>::from(SystemTime::now()),
            ))
            .await
            .map_err(APIError::database_error)?
        else {
            return Err(APIError::UnknownDatabaseError(
                "Failed to update the map timestamp".to_string(),
            ));
        };
        map
    } else {
        if ratelimiter
            .lock()
            .ignore_poison()
            .check_limited(SiteAction::Upload, &ip)
        {
            return Err(APIError::Ratelimited());
        }

        let map: Thing = ("beatmaps", uuid.to_string().as_str()).into();
        let Some(_): Option<User> = db
            .update(("users", charter_id.id.to_string()))
            .merge(UserMapUpdate {
                maps: vec![map.clone()],
            })
            .await
            .map_err(APIError::database_error)?
        else {
            return Err(APIError::UnknownDatabaseError(
                "Failed to update the user's maps".to_string(),
            ));
        };
        let Some(map): Option<BeatMap> = db
            .create(("beatmaps", uuid.to_string()))
            .content(beatmap)
            .await
            .map_err(APIError::database_error)?
        else {
            return Err(APIError::UnknownDatabaseError(
                "Failed to create the beatmap".to_string(),
            ));
        };
        map
    })
}

pub fn save_image(data: &mut FileData, path: &PathBuf, uuid: &Uuid) -> Result<(), APIError> {
    if let Some(ref image) = data.image {
        if image.is_empty() {
            data.image = None;
        } else {
            let reader = ImageReader::new(Cursor::new(image)).with_guessed_format()?;
            if !reader
                .format()
                .is_some_and(|format| SUPPORTED_FORMATS.contains(&format))
            {
                return Err(APIError::ZipError(Error::msg(
                    format!("Unknown or unsupported background image format, please use png, bmp or jpeg! {:?}", reader.format()))));
            }
            match reader.decode() {
                Ok(image) => {
                    image
                        .save_with_format(path.join(format!("{uuid}.png")), ImageFormat::Png)
                        .unwrap();
                }
                Err(err) => {
                    return Err(APIError::ZipError(Error::from(err)));
                }
            }
        }
    }
    Ok(())
}

pub async fn get_form(request: Request<Incoming>) -> Result<UploadForm, APIError> {
    let header = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok())
        .map(|inner| Ok(inner))
        .unwrap_or(Err(APIError::ArgumentError()))?;

    let mut form = UploadForm::default();
    let mut multipart = Multipart::with_constraints(
        request.into_body().into_data_stream(),
        header,
        Constraints::new()
            .allowed_fields(vec!["beatmap", "firebaseToken"])
            .size_limit(SizeLimit::new().whole_stream(MAX_SIZE as u64 + 5000)),
    );
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|err| APIError::KnownArgumentError(err.into()))?
    {
        if let Some(name) = field.name() {
            match name {
                "beatmap" => {
                    form.beatmap = read_field(&mut field).await?;
                }
                "firebaseToken" => {
                    form.firebase_token = field
                        .text()
                        .await
                        .map_err(|err| APIError::KnownArgumentError(err.into()))?
                }
                _ => return Err(APIError::ArgumentError()),
            }
        }
    }
    Ok(form)
}

async fn read_field(field: &mut Field<'_>) -> Result<Vec<u8>, APIError> {
    let mut buf = Vec::new();
    while let Some(chunk) = field.chunk().await? {
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}
