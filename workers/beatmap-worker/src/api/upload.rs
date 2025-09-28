use serde::Deserialize;
use uuid::Uuid;
use worker::{Env, Request, Response, Result};

use crate::json_response;
use crate::util::auth::require_auth;
use crate::util::db::{Database, DatabaseBackend, NewMap};
use crate::util::r2::generate_urls;

#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub song: String,
    pub artist: String,
    pub charter: String,
    // In a real impl we would accept multipart for .zip and image; here we ignore bodies on purpose
}

pub async fn upload(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let claims = require_auth(&req, &env)?;
    let payload: UploadRequest = req.json().await?;

    // Create new map document
    let id = Uuid::new_v4();
    let charter_uid = Uuid::parse_str(&claims.sub)
        .map_err(|_| worker::Error::RustError("Invalid user id in token".into()))?;
    let new_map = NewMap {
        id,
        song: &payload.song,
        artist: &payload.artist,
        charter: &payload.charter,
        charter_uid,
        description: "",
        artist_list: "",
        image: false,
    };
    db.create_map(&new_map).await?;

    // Generate placeholder URLs for R2 objects
    let urls = generate_urls(&env, &id)?;

    json_response(200, &serde_json::json!({
        "id": id,
        "song": payload.song,
        "artist": payload.artist,
        "charter": payload.charter,
        "charter_uid": charter_uid,
        "download_url": urls.download_url,
        "thumbnail_url": urls.thumbnail_url,
    }))
}
