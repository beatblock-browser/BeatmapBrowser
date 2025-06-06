use crate::schema::UserID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordTokenRequest {
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: String,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserToken {
    pub id: UserID,
    pub token: String,
} 