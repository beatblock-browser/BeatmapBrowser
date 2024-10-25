mod parsing;
mod body;
mod database;
mod search;
mod upload;
mod discord;
mod ratelimiter;

use std::env;
use crate::body::EitherBody;
use crate::database::connect;
use crate::discord::run_bot;
use crate::ratelimiter::{Ratelimiter, SiteAction, UniqueIdentifier};
use crate::search::search_database;
use crate::upload::upload;
use anyhow::Error;
use firebase_auth::FirebaseAuth;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::http::response::Builder;
use hyper::server::conn::{http1, http2};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_staticfile::Static;
use hyper_util::rt::{TokioIo, TokioTimer};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, LockResult, Mutex};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = std::env::args().nth(1).unwrap().parse().unwrap();

    let site = Static::new(Path::new("site/"));
    let db = connect().await?;

    let firebase_auth = FirebaseAuth::new("beatblockbrowser").await;

    let data = SiteData {
        site,
        db,
        auth: firebase_auth,
        ratelimiter: Arc::new(Mutex::new(Ratelimiter::new())),
    };

    let _ = tokio::spawn(run_bot(data.db.clone(), data.ratelimiter.clone()));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to create TCP listener");
    eprintln!("Server running on https://{}/", addr);
    loop {
        match handle_connection(&listener, data.clone()).await {
            Ok(()) => {}
            Err(err) => println!("Error serving connection: {err:?}")
        }
    }
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
                service_fn(move |req| {
                    let data = data.clone();
                    handle_request(req, ip, data)
                }),
            )
            .await
        {
            eprintln!("Error serving connection: {:?}", err);
        }
    });
    Ok(())
}

fn build_request(data: (StatusCode, String)) -> Result<Response<EitherBody>, hyper::http::Error> {
    Builder::new().status(data.0)
        .body(Full::new(Bytes::from(data.1)).into())
}

#[derive(Clone)]
pub struct SiteData {
    site: Static,
    db: Surreal<Client>,
    auth: FirebaseAuth,
    ratelimiter: Arc<Mutex<Ratelimiter>>,
}

async fn handle_request(request: Request<hyper::body::Incoming>, ip: SocketAddr, data: SiteData) -> Result<Response<EitherBody>, Error> {
    Ok(match (request.method(), request.uri().path()) {
        (&Method::GET, "/api/search") => {
            if data.ratelimiter.lock().ignore_poison().check_limited(SiteAction::Search, &UniqueIdentifier::Ip(ip)) {
                build_request((StatusCode::TOO_MANY_REQUESTS, "Ratelimited".to_string()))?
            } else {
                build_request(match search_database(request.uri().query().unwrap_or(""), data.db).await {
                    Ok(maps) => (StatusCode::default(), serde_json::to_string(&maps).expect("Failed to serialize maps")),
                    Err(error) => {
                        println!("Search Error: {:?}", error);
                        (error.get_code(), error.to_string())
                    }
                })?
            }
        }
        (&Method::POST, "/api/upload") => match upload(request, ip, &data).await {
            Ok(query) => Builder::new().status(StatusCode::OK).body(Full::new(Bytes::from(format!("{query}"))).into()),
            Err(error) => {
                println!("Upload Error: {:?}", error);
                build_request((error.get_code(), error.to_string()))
            }
        }?,
        // Default to static files
        _ => {
            data.site.serve(request).await.expect("Failed to serve static file").map(|body| body.into())
        }
    })
}

pub trait LockResultExt {
    type Guard;

    /// Returns the lock guard even if the mutex is [poisoned].
    ///
    /// [poisoned]: https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html#poisoning
    fn ignore_poison(self) -> Self::Guard;
}

impl<Guard> LockResultExt for LockResult<Guard> {
    type Guard = Guard;

    fn ignore_poison(self) -> Guard {
        self.unwrap_or_else(|e| e.into_inner())
    }
}