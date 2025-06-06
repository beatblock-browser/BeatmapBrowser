use crate::api::APIError;
use crate::schema::search::{SearchRequest, SearchResult};
use crate::util::mongo::MongoDB;
use actix_web::post;
use actix_web::web::{Data, Json};
use std::sync::Arc;

#[post("/search")]
pub async fn search(
    query: Json<SearchRequest>,
    database: Data<Arc<MongoDB>>
) -> Result<Json<SearchResult>, APIError> {
    Ok(Json(SearchResult {
        query: query.query.clone(),
        results: database.search_songs(&query.query).await.map_err(APIError::database_error)?,
    }))
}