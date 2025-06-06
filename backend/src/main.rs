mod api;
mod discord;
mod parsing;
mod util;
mod schema;

use crate::discord::run_bot;
use crate::util::mongo::MongoDB;
use crate::util::ratelimiter::{Ratelimiter, SiteAction};
use crate::util::warp::{check_ratelimit, with_auth, Replyable};
use anyhow::Error;
use firebase_auth::FirebaseAuth;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::env;
use std::future::Future;
use std::sync::{Arc, Mutex};
use warp::path::param;
use warp::{get, path, post, Filter, Reply};

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

    fn json<T: DeserializeOwned + Send>(action: SiteAction, path: &str) -> impl Filter + use<'_, T> {
        warp::path("api").and(warp::path(path)).and(check_ratelimit(action)).untuple_one()
            .and(post()).and(warp::body::json::<T>())
    }

    fn auth_json<T: DeserializeOwned + Send>(action: SiteAction, path: &str) -> impl Filter + use<'_, T> {
        json(action, path).and(with_auth())
    }

    let account_data_route = auth(SiteAction::UpvoteList, "account_data")
        .map(|user: User| user.reply());

    let delete_route = auth_json::<DeleteRequest>(SiteAction::Search, "delete")
        .and_then(delete);

    let download_route = auth_json::<DownloadedRequest>(SiteAction::Download, "download")
        .and_then(download);

    let remove_route = auth_json::<DownloadedRequest>(SiteAction::Download, "remove")
        .and_then(remove);

    let search_route = json::<SearchRequest>(SiteAction::Search, "search")
        .and_then(search);

    let upvote_route = auth_json::<UpvoteRequest>(SiteAction::Search, "upvote")
        .and_then(upvote);

    let unvote_route = auth_json::<UpvoteRequest>(SiteAction::Search, "unvote")
        .and_then(unvote);

    let upload_route = limit(SiteAction::Search, "upload")
        .and(post())
        .and(extract_identifier())
        .and(multipart::form())
        .and_then(upload);

    let usersongs_route = json::<UsersongsRequest>(SiteAction::Search, "usersongs")
        .and_then(usersongs);

    let discordauth_route = limit_param(SiteAction::UpvoteList, "discordauth")
        .and_then(discord_signin);

    let discordsync_route = auth(SiteAction::UpvoteList, "discordsync")
        .and(param())
        .and_then(discord_sync);

    let googleauth_route = limit_param(SiteAction::UpvoteList, "googleauth")
        .and_then(google_signin);

    let googlesync_route = auth(SiteAction::UpvoteList, "googlesync")
        .and(param())
        .and_then(google_sync);

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
        .run(env::args().nth(1).unwrap().parse::<SocketAddr>()?)
        .await;
    Ok(())
}

#[derive(Clone)]
pub struct SiteData {
    auth: FirebaseAuth,
    database: MongoDB,
    ratelimiter: Arc<Mutex<Ratelimiter>>,
}
