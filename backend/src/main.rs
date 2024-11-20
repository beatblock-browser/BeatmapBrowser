mod api;
mod discord;
mod parsing;
mod util;

use crate::api::account_data::account_data;
use crate::api::delete::delete;
use crate::api::discord_signin::discord_signin;
use crate::api::downloaded::{download, remove};
use crate::api::search::search_database;
use crate::api::upload::upload;
use crate::api::upvote::{unvote, upvote};
use crate::api::usersongs::usersongs;
use crate::api::APIError;
use crate::discord::run_bot;
use crate::util::body::EitherBody;
use crate::util::database::connect;
use crate::util::ratelimiter::{Ratelimiter, UniqueIdentifier};
use anyhow::Error;
use firebase_auth::FirebaseAuth;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::http::response::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_staticfile::Static;
use hyper_util::rt::{TokioIo, TokioTimer};
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting version {}", env!("CARGO_PKG_VERSION"));

    let addr: SocketAddr = std::env::args().nth(1).unwrap().parse().unwrap();

    println!("Exists: {} for {}", fs::metadata("site/").is_ok(), std::env::args().nth(2).unwrap());
    let data = SiteData {
        site: Static::new(std::env::args().nth(2).unwrap()),
        db: connect().await?,
        auth: FirebaseAuth::new("beatblockbrowser").await,
        ratelimiter: Arc::new(Mutex::new(Ratelimiter::new())),
    };

    let _ = tokio::spawn(run_bot(data.db.clone(), data.ratelimiter.clone()));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to create TCP listener");
    eprintln!("Server running on http://{}/", addr);
    loop {
        match handle_connection(&listener, data.clone()).await {
            Ok(()) => {}
            Err(err) => println!("Error serving connection: {err:?}"),
        }
    }
}

async fn handle_request(
    request: Request<hyper::body::Incoming>,
    ip: SocketAddr,
    data: SiteData,
) -> Result<Response<EitherBody>, Error> {
    let identifier = match ip {
        SocketAddr::V4(ip) => UniqueIdentifier::Ipv4(ip.ip().clone()),
        SocketAddr::V6(ip) => UniqueIdentifier::Ipv6(ip.ip().clone()),
    };
    let request_path = request.uri().path().to_string();
    let method = match (request.method(), &*request_path) {
        (&Method::GET, "/api/search") => search_database(request, identifier, &data).await,
        (&Method::GET, "/api/usersongs") => usersongs(request, identifier, &data).await,
        (&Method::GET, "/api/discordauth") => discord_signin(request, identifier, &data).await,
        (&Method::POST, "/api/upvote") => upvote(request, identifier, &data).await,
        (&Method::POST, "/api/unvote") => unvote(request, identifier, &data).await,
        (&Method::POST, "/api/download") => download(request, identifier, &data).await,
        (&Method::POST, "/api/remove") => remove(request, identifier, &data).await,
        (&Method::POST, "/api/account_data") => account_data(request, identifier, &data).await,
        (&Method::POST, "/api/upload") => upload(request, identifier, &data).await,
        (&Method::POST, "/api/delete") => delete(request, identifier, &data).await,
        _ => {
            return match data
                .site
                .serve(request)
                .await {
                Ok(file) => Ok(file.map(|body| body.into())),
                Err(error) => {
                    println!("Error serving static file: {error}");
                    Builder::new().status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from("Site is likely overwhelmed, pleaase try again later")).into())
                        .map_err(Error::new)
                }
            };
        }
    };

    Ok(match method {
        Ok(query) => Builder::new()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from(format!("{query}"))).into()),
        Err(error) => {
            if let APIError::Ratelimited() = error {} else {
                println!("Error with {}: {:?}", request_path, error);
            }
            build_request((error.get_code(), error.to_string()))
        }
    }?)
}

async fn handle_connection(listener: &TcpListener, data: SiteData) -> Result<(), Error> {
    let (stream, ip) = listener
        .accept()
        .await
        .expect("Failed to accept TCP connection");

    tokio::spawn(async move {
        if let Err(err) = http1::Builder::new()
            .timer(TokioTimer::new())
            .serve_connection(
                TokioIo::new(stream),
                service_fn(move |req| handle_request(req, ip, data.clone())),
            )
            .await
        {
            eprintln!("Error serving connection: {:?}", err);
        }
    });
    Ok(())
}

fn build_request(data: (StatusCode, String)) -> Result<Response<EitherBody>, hyper::http::Error> {
    Builder::new()
        .status(data.0)
        .body(Full::new(Bytes::from(data.1)).into())
}

#[derive(Clone)]
pub struct SiteData {
    site: Static,
    db: Surreal<Client>,
    auth: FirebaseAuth,
    ratelimiter: Arc<Mutex<Ratelimiter>>,
}
