use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

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