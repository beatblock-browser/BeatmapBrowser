use crate::types::APIError;
use anyhow::Error;
use mongodb::Client;
use serde::Serialize;

pub const MAPS_COLLECTION: &'static str = "maps";
pub const USERS_COLLECTION: &'static str = "users";
pub const TOKENS_COLLECTION: &'static str = "tokens";

pub struct MongoDB {
    client: Client,
}

impl MongoDB {
    pub async fn connect() -> Result<Self, Error> {
        Ok(Self {
            client: Client::with_uri_str("mongodb://localhost:27017").await?
        })
    }

    pub async fn upload<T: Serialize + Send + Sync>(&mut self, data: T, database: &str)
                                                    -> Result<(), APIError> {
        self.client.database("beatmapbrowser").collection(database).insert_one(data).await
            .map_err(APIError::database_error)?;
        Ok(())
    }
}