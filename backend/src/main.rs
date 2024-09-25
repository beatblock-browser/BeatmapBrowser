mod database;
mod body;
mod search;

use crate::body::EitherBody;
use crate::database::connect;
use crate::search::search_database;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_staticfile::Static;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use firebase_auth::FirebaseAuth;
use multipart::client::Multipart;
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

async fn handle_request(request: Request<hyper::body::Incoming>, site: Static, db: Surreal<Client>, auth: FirebaseAuth) -> Result<Response<EitherBody>, Infallible> {
    Ok(match request.uri().path() {
        "/api/search" => {
            let maps = match search_database(request.uri().query().unwrap_or(""), db).await {
                Ok(maps) => maps,
                Err(error) => {
                    println!("Error: {:?}", error);
                    return Ok(Response::builder().status(error.get_code())
                        .body(Full::new(Bytes::from(error.to_string())).into()).expect("Failed to make error response"))
                }
            };

            Response::new(Full::new(Bytes::from(serde_json::to_string(&maps).expect("Failed to serialize maps"))).into())
        },
        "/api/upload" => {
            let bytes = request.into_body().collect().await.unwrap().to_bytes();
            Multipart::from_request(request)
            let uploaded = String::from_utf8_lossy(&bytes);

            println!("{}", uploaded);
            Response::new(Full::new(Bytes::from("Hello World!")).into())
        },
        // Default to static files
        _ => site.serve(request).await.expect("Failed to serve static file").map(|body| body.into()),
    })
}