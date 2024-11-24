use crate::api::APIError;
use crate::util::database::{BeatMap, User};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::LockResultExt;
use crate::SiteData;
use anyhow::Error;
use hyper::body::Incoming;
use hyper::Request;
use serde::{Deserialize, Serialize};
use crate::util::amazon::{MAPS_TABLE_NAME, USERS_TABLE_NAME};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserpageArguments {
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongsResult {
    pub results: Vec<BeatMap>,
}

pub async fn usersongs(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    data
        .ratelimiter
        .lock()
        .ignore_poison()
        .check_limited(SiteAction::Search, &identifier)?;
    
    let arguments = serde_urlencoded::from_str::<UserpageArguments>(request.uri().query().unwrap_or(""))
        .map_err(|_| APIError::QueryError(Error::msg(
            "Invalid userpage arguments!",
        )))?;

    let user: User = data.amazon.query_one(USERS_TABLE_NAME, "id", arguments.user)
        .await
        .map_err(|err| APIError::DatabaseError(err.into()))?
        .ok_or(APIError::KnownArgumentError(Error::msg("No user with that id")))?;
    
    let mut maps: Vec<BeatMap> = data.amazon.query(MAPS_TABLE_NAME, "charter_uid", user.id.to_string())
        .await
        .map_err(|err| APIError::DatabaseError(err.into()))?;
    maps.sort_by(|first, second| first.upvotes.cmp(&second.upvotes).reverse());
    serde_json::to_string(&SongsResult {
        results: maps,
    })
    .map_err(|err| APIError::DatabaseError(err.into()))
}
