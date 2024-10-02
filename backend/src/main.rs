mod parsing;
mod body;
mod database;
mod login;
mod search;
mod upload;

use crate::body::EitherBody;
use crate::database::connect;
use crate::search::search_database;
use crate::upload::upload;
use anyhow::Error;
use firebase_auth::FirebaseAuth;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::http::response::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_staticfile::Static;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::path::Path;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let site = Static::new(Path::new("backend/site/"));
    let db = connect().await?;

    let firebase_auth = FirebaseAuth::new("beatblockbrowser").await;

    let addr: SocketAddr = std::env::args().nth(1).unwrap().parse().unwrap();

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to create TCP listener");
    eprintln!("Server running on https://{}/", addr);
    loop {
        let (stream, _) = listener
            .accept()
            .await
            .expect("Failed to accept TCP connection");

        let site = site.clone();
        let db = db.clone();
        let auth = firebase_auth.clone();
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    TokioIo::new(stream),
                    service_fn(move |req| handle_request(req, site.clone(), db.clone(), auth.clone())),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn build_request(data: (StatusCode, String)) -> Result<Response<EitherBody>, hyper::http::Error> {
    Builder::new().status(data.0).body(Full::new(Bytes::from(data.1)).into())
}

async fn handle_request(request: Request<hyper::body::Incoming>, site: Static, db: Surreal<Client>, auth: FirebaseAuth) -> Result<Response<EitherBody>, Error> {
    Ok(match (request.method(), request.uri().path()) {
        (&Method::GET, "/api/search") => build_request(match search_database(request.uri().query().unwrap_or(""), db).await {
                Ok(maps) => (StatusCode::default(), serde_json::to_string(&maps).expect("Failed to serialize maps")),
                Err(error) => {
                    println!("Error: {:?}", error);
                    (error.get_code(), error.to_string())
                }
            })?,
        (&Method::POST, "/api/upload") => build_request(match upload(request, db, auth).await {
                Ok(_) => (StatusCode::OK, "Success".to_string()),
                Err(error) => {
                    println!("Error: {:?}", error);
                    (error.get_code(), error.to_string())
                }
            })?,
        /*(&Method::POST, "/api/login") => build_request(match login(request.uri().query().unwrap_or(""), db, auth).await {
            Ok(_) => (StatusCode::OK, "Success".to_string()),
            Err(error) => {
                println!("Error: {:?}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
            }
        })?,*/
        // Default to static files
        _ => site.serve(request).await.expect("Failed to serve static file").map(|body| body.into()),
    })
}