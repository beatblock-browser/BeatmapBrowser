use crate::api::APIError;
use crate::util::database::BeatMap;
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::LockResultExt;
use crate::SiteData;
use anyhow::Error;
use hyper::body::Incoming;
use hyper::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchArguments {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub results: Vec<BeatMap>,
}

pub async fn search_database(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    data
        .ratelimiter
        .lock()
        .ignore_poison()
        .check_limited(SiteAction::Search, &identifier)?;

    let Ok(arguments) =
        serde_urlencoded::from_str::<SearchArguments>(request.uri().query().unwrap_or(""))
    else {
        return Err(APIError::QueryError(Error::msg(
            "Invalid search arguments!",
        )));
    };

    let mut maps: Vec<BeatMap> = data.amazon.search_songs(&arguments.query).await.map_err(APIError::database_error)?;
    maps.sort_by(|first, second| first.upvotes.cmp(&second.upvotes).reverse());
    serde_json::to_string(&SearchResult {
        query: arguments.query,
        results: maps,
    })
    .map_err(|err| APIError::DatabaseError(err.into()))
}
