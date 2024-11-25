use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use anyhow::Error;
use surrealdb::sql::Thing;
use uuid::Uuid;
use crate::amazon::{setup, MAPS_TABLE_NAME, USERS_TABLE_NAME};
use crate::surreal::connect;
use crate::types::{AccountLink, BeatMap, SurrealBeatMap, SurrealUser, User};

mod types;
mod amazon;
mod surreal;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    println!("Setting up");
    let surreal = connect().await?;
    let amazon = setup().await?;

    let mut new_ids = HashMap::new();
    let maps: Vec<SurrealBeatMap> = surreal.select("beatmaps").await?;
    println!("Ready");
    for map in maps {
        let new_id = map.id.as_ref().unwrap().id.to_string();
        let new_id = Uuid::from_str(&new_id[3..new_id.len()-3])?;
        let charter_uid = new_ids.entry(map.charter_uid.as_ref().unwrap().split(":").last().unwrap().to_string())
            .or_insert(Uuid::new_v4()).clone();

        println!("Uploading {} for {}", new_id, charter_uid);
        amazon.upload(MAPS_TABLE_NAME, &BeatMap {
            song: map.song,
            artist: map.artist,
            charter: map.charter,
            charter_uid,
            difficulties: map.difficulties,
            description: map.description,
            artist_list: map.artist_list,
            image: map.image.is_some(),
            upvotes: map.upvotes,
            upload_date: map.upload_date,
            update_date: map.update_date,
            id: new_id.clone(),
        }).await?;
        amazon.upload_object(fs::read(map.download)?, format!("{}.zip", new_id.clone()).as_str()).await?;
        if let Some(image) = map.image {
            amazon.upload_object(fs::read(image)?, format!("{}.png", new_id.clone()).as_str()).await?;
        }
        println!("Uploaded {}", map.id.as_ref().unwrap().id);
    }
    let users: Vec<SurrealUser> = surreal.select("users").await?;
    for user in users {
        let new_id = user.id.as_ref().unwrap().id.to_string();
        let id = new_ids.entry(new_id[3..new_id.len()-3].to_string()).or_insert(Uuid::new_v4()).clone();
        let mut new_user = User {
            maps: user.maps.iter().map(to_uuid).collect(),
            downloaded: user.downloaded.iter().map(to_uuid).collect(),
            upvoted: user.upvoted.iter().map(to_uuid).collect(),
            id,
            links: vec![],
        };
        if let Some(google) = user.google_id {
            new_user.links.push(AccountLink::Google(google));
        }

        if let Some(discord) = user.discord_id {
            new_user.links.push(AccountLink::Discord(discord));
        }
        amazon.upload(USERS_TABLE_NAME, &new_user).await?;
    }
    Ok(())
}

pub fn to_uuid(thing: &Thing) -> Uuid {
    Uuid::from_str(thing.id.to_string().as_str()).unwrap()
}