use crate::api::upload::get_or_create_user;
use crate::util::database::{BeatMap, User};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::{collect_stream, LockResultExt, WebError};
use crate::SiteData;
use anyhow::Error;
use firebase_auth::FirebaseUser;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use surrealdb::opt::PatchOp;
use surrealdb::sql::Thing;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpvoteError {
    #[error("Ratelimited")]
    Ratelimited(),
    #[error("Invalid request")]
    QueryError(Error),
    #[error("Authentication error")]
    AuthError(String),
    #[error("Database error")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("Unknown database error")]
    UnknownDatabaseError(String),
    #[error("Deserialization error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Already upvoted!")]
    AlreadyUpvoted(),
    #[error("Unknown map!")]
    UnknownMap()
}

impl WebError for UpvoteError {
    fn get_code(&self) -> StatusCode {
        match self {
            UpvoteError::Ratelimited() => StatusCode::TOO_MANY_REQUESTS,
            UpvoteError::QueryError(_) | UpvoteError::AuthError(_) | UpvoteError::AlreadyUpvoted() | UpvoteError::UnknownMap() => StatusCode::BAD_REQUEST,
            UpvoteError::DatabaseError(_) | UpvoteError::UnknownDatabaseError(_) | UpvoteError::SerdeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpvoteRequest {
    #[serde(rename = "firebaseToken")]
    firebase_token: String,
    #[serde(rename = "mapId")]
    map_id: Thing,
}

#[derive(Serialize, Deserialize)]
pub struct UpvoteListRequest {
    #[serde(rename = "firebaseToken")]
    firebase_token: String
}

pub async fn upvote(request: Request<Incoming>, identifier: UniqueIdentifier, data: &SiteData) -> Result<String, UpvoteError> {
    if data.ratelimiter.lock().ignore_poison().check_limited(SiteAction::Search, &identifier) {
        return Err(UpvoteError::Ratelimited());
    }

    let request_data = collect_stream(request.into_data_stream(), 2000).await
        .map_err(|err| UpvoteError::QueryError(err))?;
    let string = String::from_utf8_lossy(request_data.deref());
    let arguments = serde_json::from_str::<UpvoteRequest>(string.deref())
        .map_err(|err| UpvoteError::QueryError(err.into()))?;

    let user: FirebaseUser = data.auth.verify(&arguments.firebase_token).map_err(|err| UpvoteError::AuthError(err.to_string()))?;
    let user = get_or_create_user(format!("SELECT * FROM users WHERE google_id == '{}'", user.user_id), &data.db, User {
        google_id: Some(user.user_id),
        ..Default::default()
    }, || UpvoteError::UnknownDatabaseError("Failed to create a user in the users database".to_string())).await?;

    let map_id = ("beatmaps", arguments.map_id.id.to_string()[3..arguments.map_id.id.to_string().len()-3].to_string());
    let Some(map): Option<BeatMap> = data.db.select(map_id.clone()).await? else {
        return Err(UpvoteError::UnknownMap());
    };

    if user.upvoted.contains(&arguments.map_id) {
        return Err(UpvoteError::AlreadyUpvoted())
    }

    let _: Option<BeatMap> = data.db.update(map_id).patch(PatchOp::replace("upvotes", map.upvotes+1)).await?;
    let _: Option<User> = data.db.update(("users", user.id.unwrap().id.to_string())).patch(PatchOp::add("upvoted", arguments.map_id)).await?;

    Ok("Ok!".to_string())
}

pub async fn upvote_list(request: Request<Incoming>, identifier: UniqueIdentifier, data: &SiteData) -> Result<String, UpvoteError> {
    if data.ratelimiter.lock().ignore_poison().check_limited(SiteAction::UpvoteList, &identifier) {
        return Err(UpvoteError::Ratelimited());
    }

    let request_data = collect_stream(request.into_data_stream(), 2000).await
        .map_err(|err| UpvoteError::QueryError(err))?;
    let string = String::from_utf8_lossy(request_data.deref());
    let arguments = serde_json::from_str::<UpvoteListRequest>(string.deref())
        .map_err(|err| UpvoteError::QueryError(err.into()))?;

    let user: FirebaseUser = data.auth.verify(&arguments.firebase_token).map_err(|err| UpvoteError::AuthError(err.to_string()))?;
    let upvoted = get_or_create_user(format!("SELECT * FROM users WHERE google_id == '{}'", user.user_id), &data.db, User {
        google_id: Some(user.user_id),
        ..Default::default()
    }, || UpvoteError::UnknownDatabaseError("Failed to create a user in the users database".to_string())).await?.upvoted;
    Ok(serde_json::to_string(&upvoted)?)
}