use crate::util::database::BeatMap;
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::{LockResultExt, WebError};
use crate::SiteData;
use anyhow::Error;
use hyper::body::Incoming;
use hyper::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Argument error")]
    QueryError(),
    #[error("Database error")]
    DatabaseError(#[from] Error),
    #[error("Ratelimited")]
    Ratelimited()
}

impl WebError for SearchError {
    fn get_code(&self) -> StatusCode {
        match self {
            SearchError::QueryError() => StatusCode::BAD_REQUEST,
            SearchError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            SearchError::Ratelimited() => StatusCode::TOO_MANY_REQUESTS
        }
    }
}

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
pub async fn search_database(request: Request<Incoming>, identifier: UniqueIdentifier, data: &SiteData) -> Result<String, SearchError> {
    if data.ratelimiter.lock().ignore_poison().check_limited(SiteAction::Search, &identifier) {
        return Err(SearchError::Ratelimited());
    }

    let Ok(arguments) = serde_urlencoded::from_str::<SearchArguments>(request.uri().query().unwrap_or("")) else {
        return Err(SearchError::QueryError());
    };

    let mut maps: Vec<BeatMap> = data.db.query("SELECT * FROM beatmaps WHERE song @@ $query OR artist @@ $query OR charter @@ $query")
        .bind(("query", arguments.query.clone())).await
        .map_err(|err| SearchError::DatabaseError(err.into()))?
        .take(0).map_err(|err| SearchError::DatabaseError(err.into()))?;
    maps.sort_by(|first, second| first.upvotes.cmp(&second.upvotes).reverse());
    serde_json::to_string(&SearchResult {
        query: arguments.query,
        results: maps
    }).map_err(|err| SearchError::DatabaseError(err.into()))
}