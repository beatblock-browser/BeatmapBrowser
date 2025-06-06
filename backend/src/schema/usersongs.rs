use crate::schema::{BeatMap, UserRequest};
use serde::{Deserialize, Serialize};

pub type UsersongsRequest = UserRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserpageArguments {
    pub user: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongsResult {
    pub results: Vec<BeatMap>,
} 