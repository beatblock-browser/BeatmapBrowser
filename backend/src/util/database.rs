use crate::parsing::LevelVariant;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BeatMap {
    pub song: String,
    pub artist: String,
    pub charter: String,
    pub charter_uid: Option<String>,
    pub difficulties: Vec<LevelVariant>,
    pub description: String,
    pub artist_list: String,
    pub image: Option<String>,
    pub download: String,
    pub upvotes: u64,
    pub upload_date: DateTime<Utc>,
    pub update_date: DateTime<Utc>,
    pub id: Option<Thing>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct User {
    pub maps: Vec<Thing>,
    pub downloaded: Vec<Thing>,
    pub upvoted: Vec<Thing>,
    pub id: Option<Thing>,
    pub discord_id: Option<u64>,
    pub google_id: Option<String>
}

pub struct UserToken {
    pub user: Thing,
    pub token: String
}

pub async fn connect() -> surrealdb::Result<Surreal<Client>> {
    // Connect to the server
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    // Select a specific namespace / database
    db.use_ns("beatblock").use_db("beatblock").await?;

    db.query("DEFINE ANALYZER ascii TOKENIZERS blank FILTERS ascii, lowercase;")
        .await
        .unwrap();
    db.query("DEFINE INDEX song_name ON TABLE beatmaps FIELDS song SEARCH ANALYZER ascii;")
        .await
        .unwrap();
    db.query("DEFINE INDEX artist_name ON TABLE beatmaps FIELDS artist SEARCH ANALYZER ascii;")
        .await
        .unwrap();
    db.query("DEFINE INDEX charter_name ON TABLE beatmaps FIELDS charter SEARCH ANALYZER ascii;")
        .await
        .unwrap();
    Ok(db)
}
