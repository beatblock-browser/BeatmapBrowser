use crate::api::search::SearchArguments;
use crate::api::APIError;
use crate::parsing::{check_archive, get_parser, parse_archive, BackgroundData};
use crate::util::amazon::{Amazon, MAPS_TABLE_NAME, USERS_TABLE_NAME};
use crate::util::database::{AccountLink, BeatMap, MapID, UserID};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::{get_user, LockResultExt};
use crate::SiteData;
use anyhow::Error;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use firebase_auth::FirebaseUser;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::header::CONTENT_TYPE;
use hyper::Request;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, ImageFormat, ImageReader, PixelWithColorType, Rgb, RgbImage};
use multer::{Constraints, Field, Multipart, SizeLimit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::ops::DerefMut;
use std::time::{Duration, SystemTime};
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

pub async fn upload(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &mut SiteData,
) -> Result<String, APIError> {
    data.ratelimiter
        .lock()
        .ignore_poison()
        .check_limited(SiteAction::Update, &identifier)?;

    let form = get_form(request).await?;
    let user: FirebaseUser = data
        .auth
        .verify(&form.firebase_token)
        .map_err(|err| APIError::AuthError(err.to_string()))?;
    let id = get_user(AccountLink::Google(user.user_id), &data.amazon).await?;
    let map = timeout(
        Duration::from_millis(10000),
        upload_beatmap(form.beatmap, data, identifier, id.id),
    )
    .await??;

    serde_urlencoded::to_string(&SearchArguments {
        query: map.song.clone(),
    })
    .map_err(|err| APIError::SongNameError(err))
}

pub async fn upload_beatmap(
    mut beatmap_data: Vec<u8>,
    data: &SiteData,
    ip: UniqueIdentifier,
    charter_id: UserID,
) -> Result<BeatMap, APIError> {
    let (mut beatmap, image, bg_data) = create_beatmap(&mut beatmap_data, charter_id)?;

    // Save the beatmap
    if let Some(map) = data
        .amazon
        .query(MAPS_TABLE_NAME, "charter_uid", charter_id.to_string())
        .await
        .map_err(APIError::database_error)?
        .into_iter()
        .filter(|map: &BeatMap| map.song == beatmap.song)
        .next()
    {
        // Update the old map instead
        beatmap.id = map.id;
        data.amazon
            .update(MAPS_TABLE_NAME, beatmap.id.to_string(), |builder| {
                builder
                    .update_expression("SET upload_date = :date")
                    .expression_attribute_values(
                        ":date",
                        AttributeValue::S(<DateTime<Utc> as ToString>::to_string(&DateTime::from(SystemTime::now()))),
                    )
            })
            .await
            .map_err(APIError::database_error)?;
    } else {
        data.ratelimiter
            .lock()
            .ignore_poison()
            .check_limited(SiteAction::Upload, &ip)?;

        data.amazon
            .add_to_list(
                USERS_TABLE_NAME,
                charter_id.to_string(),
                "maps",
                beatmap.id.to_string(),
            )
            .await?;
        data.amazon
            .upload_song(&beatmap)
            .await
            .map_err(APIError::database_error)?;
    }

    save_image(&image, &bg_data, &data.amazon, &beatmap.id).await?;
    data.amazon
        .upload_object(beatmap_data, format!("{}.zip", beatmap.id).as_str())
        .await
        .map_err(APIError::database_error)?;
    Ok(beatmap)
}

pub fn create_beatmap(
    beatmap: &mut Vec<u8>,
    charter_id: UserID,
) -> Result<(BeatMap, Option<Vec<u8>>, Option<BackgroundData>), APIError> {
    let file_data = 
        parse_archive(get_parser(beatmap)?.deref_mut()).map_err(APIError::ZipError)?;
    check_archive(beatmap).map_err(APIError::ZipError)?;

    Ok((
        BeatMap {
            song: file_data.level_data.song_name,
            artist: file_data.level_data.artist,
            charter: file_data.level_data.charter,
            difficulties: file_data
                .level_data
                .difficulty
                .map_or(file_data.level_data.variants, |diff| vec![diff.into()]),
            description: file_data.level_data.description,
            artist_list: file_data.level_data.artist_list,
            charter_uid: charter_id,
            image: file_data.image.is_some(),
            upvotes: 0,
            upload_date: DateTime::from(SystemTime::now()),
            update_date: DateTime::from(SystemTime::now()),
            id: Uuid::new_v4(),
        },
        file_data.image,
        file_data.level_data.bg_data,
    ))
}

pub async fn save_image(
    data: &Option<Vec<u8>>,
    bg_data: &Option<BackgroundData>,
    amazon: &Amazon,
    uuid: &MapID,
) -> Result<(), APIError> {
    let Some(ref image) = data else {
        return Ok(());
    };
    if image.is_empty() {
        return Ok(());
    }
    let reader = ImageReader::new(Cursor::new(image)).with_guessed_format()?;
    if !reader
        .format()
        .is_some_and(|format| SUPPORTED_FORMATS.contains(&format))
    {
        return Err(APIError::ZipError(Error::msg(format!(
            "Unknown or unsupported background image format, please use png, bmp or jpeg! {:?}",
            reader.format()
        ))));
    }

    let image = reader
        .decode()
        .map_err(|err| APIError::ZipError(Error::from(err)))?;
    let size = (image.width(), image.height());
    let mut output = Vec::new();
    let image = replace_image_channels(image.to_rgb8(), size, bg_data);
    PngEncoder::new(&mut output).write_image(image.as_ref(), size.0, size.1, <Rgb<u8> as PixelWithColorType>::COLOR_TYPE)
        .map_err(|err| APIError::ZipError(err.into()))?;
    amazon.upload_object(output, format!("{uuid}.png").as_str()).await
        .map_err(APIError::database_error)?;
    Ok(())
}

fn replace_image_channels(
    mut img_buffer: RgbImage,
    size: (u32, u32),
    bg_data: &Option<BackgroundData>,
) -> RgbImage {
    let Some(bg_data) = bg_data else {
        return img_buffer;
    };
    let mut channels: HashMap<[u8; 3], [u8; 3]> = HashMap::new();
    if let Some(channel) = &bg_data.red_channel {
        channels.insert([255, 0, 0], channel.into());
    }
    if let Some(channel) = &bg_data.green_channel {
        channels.insert([0, 255, 0], channel.into());
    }
    if let Some(channel) = &bg_data.blue_channel {
        channels.insert([0, 0, 255], channel.into());
    }
    if let Some(channel) = &bg_data.magenta_channel {
        channels.insert([255, 0, 255], channel.into());
    }
    if let Some(channel) = &bg_data.cyan_channel {
        channels.insert([0, 255, 255], channel.into());
    }
    if let Some(channel) = &bg_data.yellow_channel {
        channels.insert([255, 255, 0], channel.into());
    }
    channels.insert([255, 255, 255], [255, 255, 255]);
    let mut i = 0;
    for pixel in img_buffer.pixels_mut() {
        if let Some(replacement) = channels.get(&pixel.0) {
            pixel.0 = *replacement;
        } else {
            pixel.0 = if ((i % size.0) % 2 == 0) && (i / size.0) % 2 == 0 {
                [0, 0, 0]
            } else {
                [255, 0, 255]
            }
        }
        i += 1;
    }
    img_buffer
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
