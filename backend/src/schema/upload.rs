use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct UploadForm {
    #[serde(rename = "firebaseToken")]
    pub firebase_token: String,
    pub beatmap: Vec<u8>,
} 