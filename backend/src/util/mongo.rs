use crate::api::APIError;
use crate::util::database::{AccountLink, BeatMap, User};
use anyhow::Error;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Client;
use serde::{Deserialize, Serialize};

pub const DATABASE: &'static str = "beatmapbrowser";
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

    pub async fn update(&self, collection: &'static str, id: Document, updater: Document)
                        -> Result<(), Error> {
        self.client.database(DATABASE).collection(collection)
            .update_one(id, updater).await.map_err(APIError::database_error)?;
        Ok(())
    }

    pub async fn upload<T: Serialize + Send + Sync>(&mut self, collection: &str, data: T)
                                                    -> Result<(), Error> {
        self.client.database(DATABASE).collection(collection).insert_one(data).await?;
        Ok(())
    }

    pub async fn search_songs(
        &self,
        query: &str,
    ) -> Result<Vec<BeatMap>, Error> {
        self.client.database(DATABASE).collection(MAPS_COLLECTION).find(doc! {
            "$text": {
                "$search": query,
            },
            "song" : 1,
            "mapper" : 1,
            "artist": 1
        }).await?.try_collect()?
    }

    pub async fn query_by_link(
        &self,
        link: AccountLink,
    ) -> Result<User, Error> {
        self.client.database(DATABASE).collection(USERS_COLLECTION)
            .find(doc! {
                "links": {
                    "$elemMatch": {
                        "type": match link {
                            AccountLink::Google(_) => "google",
                            AccountLink::Discord(_) => "discord"
                        },
                        "id": match link {
                            AccountLink::Google(id) => id,
                            AccountLink::Discord(id) => id.to_string()
                        }
                    }
                }
            }).await?.try_next().await?
            .ok_or(Error::msg("No items found"))
    }

    pub async fn query_one<T: for<'a> Deserialize<'a>>(
        &self,
        collection: &'static str,
        document: Document,
    ) -> Result<Option<T>, Error> {
        Ok(self.client.database(DATABASE).collection(collection)
            .find_one(document).await?)
    }

    pub async fn query<T: for<'a> Deserialize<'a>>(
        &self,
        collection: &'static str,
        document: Document,
    ) -> Result<Vec<T>, Error> {
        Ok(self.client.database(DATABASE).collection(collection)
            .find(document).await?.try_collect().await?)
    }

    pub async fn remove(
        &self,
        collection: &'static str,
        document: Document
    ) -> Result<(), Error> {
        self.client.database(DATABASE).collection(collection)
            .delete_one(document).await?;
        Ok(())
    }
}