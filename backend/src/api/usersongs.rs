use crate::api::APIError;
use crate::util::database::{BeatMap, User};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::LockResultExt;
use crate::SiteData;
use anyhow::Error;
use hyper::body::Incoming;
use hyper::Request;
use serde::{Deserialize, Serialize};

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
    if data
        .ratelimiter
        .lock()
        .ignore_poison()
        .check_limited(SiteAction::Search, &identifier)
    {
        return Err(APIError::Ratelimited());
    }

    let Ok(arguments) =
        serde_urlencoded::from_str::<UserpageArguments>(request.uri().query().unwrap_or(""))
    else {
        return Err(APIError::QueryError(Error::msg(
            "Invalid userpage arguments!",
        )));
    };

    let Some(user): Option<User> = data.db.select(("users", &arguments.user)).await
        .map_err(APIError::database_error)? else {
        return Err(APIError::KnownArgumentError(Error::msg("No user with that id")))
    };

    let mut maps: Vec<BeatMap> = data
        .db
        .query(
            format!("SELECT * FROM beatmaps WHERE charter_uid == '{}'", user.id.unwrap().to_string()),
        )
        .await
        .map_err(|err| APIError::DatabaseError(err.into()))?
        .take(0)
        .map_err(|err| APIError::DatabaseError(err.into()))?;
    maps.sort_by(|first, second| first.upvotes.cmp(&second.upvotes).reverse());
    serde_json::to_string(&SongsResult {
        results: maps,
    })
    .map_err(|err| APIError::DatabaseError(err.into()))
}
