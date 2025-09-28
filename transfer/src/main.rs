use crate::amazon::{setup, MAPS_TABLE_NAME, USERS_TABLE_NAME};
use crate::types::{BeatMap, User};
use anyhow::{Context, Error};
use std::env;
use crate::d1::D1;

mod amazon;
mod types;
mod d1;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let d1_sqlite_path = env::var("D1_SQLITE_PATH")
        .context("D1_SQLITE_PATH must be set to the local D1 SQLite file path")?;
    let mut d1 = D1::connect(&d1_sqlite_path)?;

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
        d1.upsert_map(&map)?;
    }

    for user in users {
        d1.upsert_user(&user)?;
    }
    Ok(())
}