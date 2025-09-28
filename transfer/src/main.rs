use crate::amazon::{setup, MAPS_TABLE_NAME, USERS_TABLE_NAME};
use crate::d1::D1;
use crate::types::{BeatMap, LevelVariant, MapID, User};
use anyhow::{Context, Error};
use serde::Serialize;
use serde_dynamo::from_item;
use serde_json::to_string;
use std::collections::{HashMap, HashSet};
use std::env;

mod amazon;
mod types;
mod d1;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let d1_sqlite_path = env::var("D1_SQLITE_PATH")
        .context("D1_SQLITE_PATH must be set to the local D1 SQLite file path")?;
    let mut d1 = D1::connect(&d1_sqlite_path)?;

    let amazon = setup().await?;

    // Begin transfer: deduplicate maps and normalize upvotes

    let maps = amazon.db_client.scan().table_name(MAPS_TABLE_NAME)
        .send().await?.items.context("Failed to find maps")?.into_iter()
        .map(from_item)
        .collect::<Result<Vec<BeatMap>, serde_dynamo::Error>>()?;
    let users = amazon.db_client.scan().table_name(USERS_TABLE_NAME)
        .send().await?.items.context("Failed to find users")?.into_iter()
        .map(from_item)
        .collect::<Result<Vec<User>, serde_dynamo::Error>>()?;

    // 1) Deduplicate maps by exact fingerprint (all content fields). Keep the latest by update_date.
    let (canonical_maps, id_to_canonical) = deduplicate_maps(maps)?;

    // 2) Normalize user upvotes to canonical IDs and compute canonical upvote counts.
    let (normalized_users, upvotes_by_canonical) = normalize_user_upvotes(users, &id_to_canonical);

    // 3) Set canonical maps' upvote counters based on normalized unique user upvotes.
    let mut canonical_maps_with_upvotes = canonical_maps;
    for map in canonical_maps_with_upvotes.values_mut() {
        let count = upvotes_by_canonical
            .get(&map.id)
            .map(|s| s.len() as u64)
            .unwrap_or(0);
        map.upvotes = count;
    }

    // 4) Write only canonical maps, then normalized users
    for map in canonical_maps_with_upvotes.values() {
        d1.upsert_map(map)?;
    }

    for user in normalized_users {
        d1.upsert_user(&user)?;
    }
    Ok(())
}

type CanonicalMaps = HashMap<String, BeatMap>;
type IdMap = HashMap<MapID, MapID>;
type UpvotesByCanonical = HashMap<MapID, HashSet<String>>;

#[derive(Serialize)]
struct MapFingerprint<'a> {
    song: &'a str,
    artist: &'a str,
    charter: &'a str,
    charter_uid: &'a str,
    description: &'a str,
    artist_list: &'a str,
    image: bool,
    difficulties: &'a [LevelVariant],
}

fn map_fingerprint(map: &BeatMap) -> Result<String, Error> {
    // Use a stable JSON representation of all content-defining fields.
    // Note: charter_uid and difficulties are included to ensure exact duplicates are 100% identical.
    let fp = MapFingerprint {
        song: &map.song,
        artist: &map.artist,
        charter: &map.charter,
        charter_uid: &map.charter_uid.to_string(),
        description: &map.description,
        artist_list: &map.artist_list,
        image: map.image,
        difficulties: &map.difficulties,
    };
    Ok(to_string(&fp)?)
}

fn deduplicate_maps(
    maps: Vec<BeatMap>,
) -> Result<(CanonicalMaps, IdMap), Error> {
    let mut latest_by_fp: CanonicalMaps = HashMap::new();
    let mut groups: HashMap<String, Vec<MapID>> = HashMap::new();

    for map in maps {
        let key = map_fingerprint(&map)?;
        let group = groups.entry(key.clone()).or_default();
        group.push(map.id);

        match latest_by_fp.get_mut(&key) {
            Some(existing) => {
                if map.update_date > existing.update_date {
                    // Replace with newer version
                    *existing = map;
                }
            }
            None => {
                latest_by_fp.insert(key, map);
            }
        }
    }

    // Build mapping from every map ID to its canonical (latest) map ID
    let mut id_to_canonical: IdMap = HashMap::new();
    for (key, ids) in groups {
        if let Some(canonical) = latest_by_fp.get(&key) {
            for id in ids {
                id_to_canonical.insert(id, canonical.id);
            }
        }
    }

    Ok((latest_by_fp, id_to_canonical))
}

fn normalize_user_upvotes(
    users: Vec<User>,
    id_to_canonical: &IdMap,
) -> (Vec<User>, UpvotesByCanonical) {
    let mut upvotes_by_canonical: UpvotesByCanonical = HashMap::new();
    let mut normalized_users: Vec<User> = Vec::with_capacity(users.len());

    for mut user in users {
        // Remap each upvoted ID -> canonical, ensure uniqueness per user
        let mut set: HashSet<MapID> = HashSet::new();
        for id in user.upvoted.into_iter() {
            let canonical = id_to_canonical.get(&id).copied().unwrap_or(id);
            set.insert(canonical);
            upvotes_by_canonical
                .entry(canonical)
                .or_default()
                .insert(user.id.to_string());
        }
        user.upvoted = set.into_iter().collect();
        normalized_users.push(user);
    }

    (normalized_users, upvotes_by_canonical)
}