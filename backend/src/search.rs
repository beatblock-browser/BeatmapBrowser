use crate::database::BeatMap;
use crate::search::SearchError::QueryError;
use hyper::StatusCode;
use serde::Deserialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Query error")]
    QueryError(),
    #[error("Database error")]
    DatabaseError(#[from] surrealdb::Error),
}

impl SearchError {
    pub fn get_code(&self) -> StatusCode {
        match self {
            QueryError() => StatusCode::BAD_REQUEST,
            SearchError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchArguments {
    pub query: String,
}

/*
analyzer:
DEFINE ANALYZER ascii TOKENIZERS blank FILTERS ascii, lowercase;
index:
DEFINE INDEX songName ON TABLE beatmaps FIELDS song SEARCH ANALYZER ascii;
 */
pub async fn search_database(query: &str, db: Surreal<Client>) -> Result<Vec<BeatMap>, SearchError> {
    let Ok(arguments) = serde_urlencoded::from_str::<SearchArguments>(query) else {
        return Err(QueryError());
    };

    Ok(db.query("SELECT * FROM beatmaps WHERE song @@ $query")
        .bind(("query", arguments.query)).await?.take(0)?)
}