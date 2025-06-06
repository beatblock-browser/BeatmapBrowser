use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuthenticatedRequest {
    #[serde(rename = "firebaseToken")]
    pub firebase_token: String,
} 