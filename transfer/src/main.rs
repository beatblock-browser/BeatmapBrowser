use std::fs;
use std::str::FromStr;
use anyhow::Error;
use uuid::Uuid;
use crate::amazon::{setup, MAPS_TABLE_NAME, USERS_TABLE_NAME};
use crate::surreal::connect;
use crate::types::{AccountLink, BeatMap, SurrealBeatMap, SurrealUser, User};

mod types;
mod amazon;
mod surreal;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let surreal = connect().await?;
    let amazon = setup().await?;
    
    let maps: Vec<SurrealBeatMap> = surreal.select("beatmaps").await?;
    for map in maps {
        amazon.upload(MAPS_TABLE_NAME, &BeatMap {
            song: map.song,
            artist: map.artist,
            charter: map.charter,
            charter_uid: Uuid::from_str(map.charter_uid.unwrap().as_str())?,
            difficulties: map.difficulties,
            description: map.description,
            artist_list: map.artist_list,
            image: map.image.is_some(),
            upvotes: map.upvotes,
            upload_date: map.upload_date,
            update_date: map.update_date,
            id: map.id.unwrap().into(),
        }).await?;
        amazon.upload_object(fs::read(map.download)?, format!("{}.zip", map.id.unwrap().id).as_str()).await?;
        if let Some(image) = map.image {
            amazon.upload_object(fs::read(image)?, format!("{}.png", map.id.unwrap().id).as_str()).await?;
        }
    }
    let users: Vec<SurrealUser> = surreal.select("users").await?;
    for user in users {
        let mut new_user = User {
            maps: user.maps.iter().map(|x| x.into()).collect(),
            downloaded: user.downloaded.iter().map(|x| x.into()).collect(),
            upvoted: user.upvoted.iter().map(|x| x.into()).collect(),
            id: user.id.unwrap().into(),
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