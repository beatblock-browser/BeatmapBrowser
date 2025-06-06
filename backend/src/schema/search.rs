use serde::{Deserialize, Serialize};
use crate::schema::BeatMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub results: Vec<BeatMap>,
} 