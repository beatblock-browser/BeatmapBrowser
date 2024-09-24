use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

#[derive(Debug, Serialize, Deserialize)]
pub struct BeatMap {
    song: String,
    artist: String,
    charter: String,
    difficulty: f32,
    description: String,
    artistList: String,
    image: Option<String>,
    upvoted_by: Vec<Thing>,
    download: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Upvoted {
    #[serde(rename = "in")]
    in_: Thing,
    out: Thing
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {

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