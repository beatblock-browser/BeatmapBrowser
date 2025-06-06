use std::ops::Deref;
use crate::api::APIError;
use warp::{Rejection, Reply};
use crate::util::data;
use crate::util::warp::Replyable;
use crate::schema::search::{SearchRequest, SearchResult};

pub async fn search(
    req: SearchRequest
) -> Result<impl Reply, Rejection> {
    let query = urlencoding::decode(req.query.deref()).map_err(|_| APIError::ArgumentError())?.to_string();
    Ok(SearchResult {
        query: query.clone(),
        results: data().await.database.search_songs(&query).await.map_err(APIError::database_error)?,
    }.reply())
}