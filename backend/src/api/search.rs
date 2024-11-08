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

/*
analyzer:
DEFINE ANALYZER ascii TOKENIZERS blank FILTERS ascii, lowercase;
index:
DEFINE INDEX song_name ON TABLE beatmaps FIELDS song SEARCH ANALYZER ascii;
DEFINE INDEX artist_name ON TABLE beatmaps FIELDS artist SEARCH ANALYZER ascii;
DEFINE INDEX charter_name ON TABLE beatmaps FIELDS charter SEARCH ANALYZER ascii;
 */
pub async fn search_database(request: Request<Incoming>, identifier: UniqueIdentifier, data: &SiteData) -> Result<String, APIError> {
    if data.ratelimiter.lock().ignore_poison().check_limited(SiteAction::Search, &identifier) {
        return Err(APIError::Ratelimited());
    }

    let Ok(arguments) = serde_urlencoded::from_str::<SearchArguments>(request.uri().query().unwrap_or("")) else {
        return Err(APIError::QueryError(Error::msg("Invalid search arguments!")));
    };

    let mut maps: Vec<BeatMap> = data.db.query("SELECT * FROM beatmaps WHERE song @@ $query OR artist @@ $query OR charter @@ $query")
        .bind(("query", arguments.query.clone())).await
        .map_err(|err| APIError::DatabaseError(err.into()))?
        .take(0).map_err(|err| APIError::DatabaseError(err.into()))?;
    maps.sort_by(|first, second| first.upvotes.cmp(&second.upvotes).reverse());
    serde_json::to_string(&SearchResult {
        query: arguments.query,
        results: maps
    }).map_err(|err| APIError::DatabaseError(err.into()))
}