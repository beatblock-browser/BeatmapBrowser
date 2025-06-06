mod api;
mod discord;
mod parsing;
mod util;
mod schema;

use std::{env, fs};
use crate::api::delete::delete;
use crate::api::downloaded::{download, remove};
use crate::api::search::search;
use crate::api::signin::{discord_signin, discord_sync, google_signin, google_sync};
use crate::api::upload::upload;
use crate::api::upvote::{unvote, upvote};
use crate::api::usersongs::usersongs;
use crate::discord::run_bot;
use crate::schema::User;
use crate::util::mongo::MongoDB;
use crate::util::ratelimiter::{Ratelimiter, SiteAction};
use crate::util::warp::{check_ratelimit, extract_identifier, handle_error, with_auth, Replyable};
use firebase_auth::FirebaseAuth;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use anyhow::Error;
use warp::path::param;
use warp::{get, multipart, path, post, Filter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Starting version {}", env!("CARGO_PKG_VERSION"));

    let _ = tokio::spawn(async {
        if let Err(error) = run_bot().await {
            eprintln!("Discord bot task encountered an error: {}", error);
        }
    });

    let limit = |action, in_path: &'static str| path("api").and(path(in_path)).and(check_ratelimit(action)).untuple_one();
    let limit_param = |action, path| limit(action, path).and(get()).and(param::<String>());
    let auth = |action, path| limit(action, path).and(post()).and(with_auth());
    let auth_json_delete = |action, path| limit(action, path).and(post()).and(with_auth()).and(warp::body::json::<schema::delete::DeleteRequest>());
    let auth_json_downloaded = |action, path| limit(action, path).and(post()).and(with_auth()).and(warp::body::json::<schema::downloaded::DownloadedRequest>());
    let auth_json_upvote = |action, path| limit(action, path).and(post()).and(with_auth()).and(warp::body::json::<schema::upvote::UpvoteRequest>());

    // Define each endpoint as its own boxed filter
    let account_data_route = auth(SiteAction::UpvoteList, "account_data")
        .map(|user: User| user.reply())
        .boxed();

    let delete_route = auth_json_delete(SiteAction::Search, "delete")
        .and_then(|user, req: schema::delete::DeleteRequest| async move {
            let map = util::warp::get_map(req.map_id.to_string()).await.map_err(warp::reject::custom)?;
            delete(req, user, map).await
        })
        .boxed();

    let download_route = auth_json_downloaded(SiteAction::Download, "download")
        .and_then(|user, req: schema::downloaded::DownloadedRequest| async move {
            let map = util::warp::get_map(req.map_id.to_string()).await.map_err(warp::reject::custom)?;
            download(req, user, map).await
        })
        .boxed();

    let remove_route = auth_json_downloaded(SiteAction::Download, "remove")
        .and_then(|user, req: schema::downloaded::DownloadedRequest| async move {
            let map = util::warp::get_map(req.map_id.to_string()).await.map_err(warp::reject::custom)?;
            remove(req, user, map).await
        })
        .boxed();

    let search_route = limit(SiteAction::Search, "search")
        .and(post())
        .and(warp::body::json())
        .and_then(search)
        .boxed();

    let upvote_route = auth_json_upvote(SiteAction::Search, "upvote")
        .and_then(|user, req: schema::upvote::UpvoteRequest| async move {
            let map = util::warp::get_map(req.map_id.to_string()).await.map_err(warp::reject::custom)?;
            upvote(req, user, map).await
        })
        .boxed();

    let unvote_route = auth_json_upvote(SiteAction::Search, "unvote")
        .and_then(|user, req: schema::upvote::UpvoteRequest| async move {
            let map = util::warp::get_map(req.map_id.to_string()).await.map_err(warp::reject::custom)?;
            unvote(req, user, map).await
        })
        .boxed();

    let upload_route = limit(SiteAction::Search, "upload")
        .and(post())
        .and(extract_identifier())
        .and(multipart::form())
        .and_then(upload)
        .boxed();

    let usersongs_route = limit(SiteAction::Search, "usersongs")
        .and(post())
        .and(warp::body::json())
        .and_then(usersongs)
        .boxed();

    let discordauth_route = limit_param(SiteAction::UpvoteList, "discordauth")
        .and_then(discord_signin)
        .boxed();

    let discordsync_route = auth(SiteAction::UpvoteList, "discordsync")
        .and(param())
        .and_then(discord_sync)
        .boxed();

    let googleauth_route = limit_param(SiteAction::UpvoteList, "googleauth")
        .and_then(google_signin)
        .boxed();

    let googlesync_route = auth(SiteAction::UpvoteList, "googlesync")
        .and(param())
        .and_then(google_sync)
        .boxed();

    let static_route = warp::fs::dir(env::args().nth(2).unwrap()).boxed();

    let routes = account_data_route
        .or(delete_route)
        .or(download_route)
        .or(remove_route)
        .or(search_route)
        .or(upvote_route)
        .or(unvote_route)
        .or(upload_route)
        .or(usersongs_route)
        .or(discordauth_route)
        .or(discordsync_route)
        .or(googleauth_route)
        .or(googlesync_route)
        .or(static_route)
        .recover(handle_error);

    warp::serve(routes)
        .run(std::env::args().nth(1).unwrap().parse::<SocketAddr>().unwrap())
        .await;
    Ok(())
}

#[derive(Clone)]
pub struct SiteData {
    auth: FirebaseAuth,
    database: MongoDB,
    ratelimiter: Arc<Mutex<Ratelimiter>>,
}
