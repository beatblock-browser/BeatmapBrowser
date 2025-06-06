use serde::{Deserialize, Serialize};
use crate::schema::{UserID, MapID};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadedRequest {
    pub user_id: UserID,
    pub map_id: MapID,
} 