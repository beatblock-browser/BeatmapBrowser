use crate::amazon::{setup, MAPS_TABLE_NAME, TOKENS_TABLE_NAME, USERS_TABLE_NAME};
use crate::types::{BeatMap, User, UserToken};
use anyhow::{Context, Error};
use std::str::FromStr;
use crate::mongodb::{MongoDB, MAPS_COLLECTION, TOKENS_COLLECTION, USERS_COLLECTION};

mod amazon;
mod types;
mod mongodb;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let mut mongo = MongoDB::connect().await?;
    let amazon = setup().await?;

    let maps = amazon.db_client.scan().table_name(MAPS_TABLE_NAME)
        .send().await?.items.context("Failed to find maps")?.into_iter()
        .map(|item| serde_dynamo::from_item(item))
        .collect::<Result<Vec<BeatMap>, serde_dynamo::Error>>()?;
    let users = amazon.db_client.scan().table_name(USERS_TABLE_NAME)
        .send().await?.items.context("Failed to find users")?.into_iter()
        .map(|item| serde_dynamo::from_item(item))
        .collect::<Result<Vec<User>, serde_dynamo::Error>>()?;

    for map in maps {
        mongo.upload(map, MAPS_COLLECTION).await?;
    }

    for user in users {
        mongo.upload(user, USERS_COLLECTION).await?;
    }
    Ok(())
}