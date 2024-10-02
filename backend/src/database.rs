use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

#[derive(Debug, Serialize, Deserialize)]
pub struct BeatMap {
    pub song: String,
    pub artist: String,
    pub charter: String,
    pub charter_uid: Option<String>,
    pub difficulty: f32,
    pub description: String,
    pub artist_list: String,
    pub image: Option<String>,
    pub download: String,
    pub upvotes: u64,
    pub upload_date: DateTime<Utc>,
    pub id: Thing
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    maps: Vec<Thing>,
    upvoted: Vec<Thing>,
    account_type: AccountType
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AccountType {
    Google(GoogleAccount)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleAccount {
    uid: String
}

#[derive(Debug, Deserialize)]
struct Record {
    #[allow(dead_code)]
    id: Thing,
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

    Ok(db)
}