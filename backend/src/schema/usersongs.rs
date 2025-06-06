use serde::{Deserialize, Serialize};
use crate::schema::{BeatMap, UserID};

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersongsRequest {
    pub user_id: UserID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserpageArguments {
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongsResult {
    pub results: Vec<BeatMap>,
} 