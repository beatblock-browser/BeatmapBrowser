mod api;
//mod discord;
mod parsing;
mod util;
mod schema;

use crate::api::search::search;
//use crate::discord::run_bot;
use crate::util::mongo::MongoDB;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use anyhow::Error;
use firebase_auth::FirebaseAuth;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use crate::api::{map_data, APIError};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Starting version {}", env!("CARGO_PKG_VERSION"));
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    /*let _ = tokio::spawn(async {
        if let Err(error) = run_bot().await {
            eprintln!("Discord bot task encountered an error: {}", error);
        }
    });*/

    let auth = Arc::new(FirebaseAuth::new("beatblockbrowser").await);
    let database = Arc::new(MongoDB::connect().await?);

    let normal_request = GovernorConfigBuilder::default()
        .seconds_per_request(3)
        .burst_size(20)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(auth.clone()))
            .app_data(Data::new(database.clone()))
            .service(web::scope("/api")
                .service(search)
                .service(map_data)
                .wrap(Governor::new(&normal_request)))
            .service(Files::new("/", env::args().nth(2).unwrap())
                .index_file("index.html")
                .default_handler(web::route().to(spa_fallback)))
    })
        .bind(env::args().nth(1).unwrap().parse::<SocketAddr>()?)?
        .run()
        .await?;
    Ok(())
}

async fn spa_fallback() -> Result<NamedFile, APIError> {
    Ok(NamedFile::open(format!("{}/index.html", env::args().nth(2).unwrap()))
        .map_err(|err| APIError::IOError(err))?)
}