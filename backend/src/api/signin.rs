use crate::api::upvote::unvote_for_map;
use crate::api::APIError;
use crate::schema::signin::{DiscordTokenRequest, DiscordUser};
use crate::schema::{AccountLink, User};
use crate::util::mongo::{MAPS_COLLECTION, USERS_COLLECTION};
use crate::util::warp::{create_jwt_response, Replyable};
use crate::util::{data, get_user_from_link};
use anyhow::{anyhow, Error};
use firebase_auth::FirebaseUser;
use mongodb::bson::doc;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde::Deserialize;
use std::env;
use actix_web::{post, Responder};

#[post("/api/discordauth")]
pub async fn discord_signin(code: String) -> impl Responder {
    let account = get_user_from_link(AccountLink::Discord(
        get_discord_user(code).await?.id.parse().unwrap(),
    ))
    .await?;
    create_jwt_response(account)
}

#[post("/api/discordsync")]
pub async fn discord_sync(user: User, code: String) -> impl Responder {
    let other = get_user_from_link(AccountLink::Discord(
        get_discord_user(code).await?.id.parse().unwrap(),
    ))
    .await?;
    merge(user, other).await?;
    Ok("Ok")
}

#[post("/api/googleauth")]
pub async fn google_signin(code: String) -> impl Responder {
    let user: FirebaseUser = data()
        .await
        .auth
        .verify(&code)
        .map_err(|err| APIError::AuthError(anyhow!(err.to_string())))?;
    let account = get_user_from_link(AccountLink::Google(user.user_id)).await?;
    create_jwt_response(account)
}

#[post("/api/googlesync")]
pub async fn google_sync(user: User, code: String) -> impl Responder {
    let firebase_user: FirebaseUser = data()
        .await
        .auth
        .verify(&code)
        .map_err(|err| APIError::AuthError(anyhow!(err.to_string())))?;
    let other = get_user_from_link(AccountLink::Google(firebase_user.user_id)).await?;
    merge(user, other).await?;
    Ok("Ok")
}

pub async fn merge(mut first: User, second: User) -> Result<(), APIError> {
    for map in second.maps {
        data()
            .await
            .database
            .update(MAPS_COLLECTION, doc!{ "id": map.to_string() }, doc! { "charter_uid": first.id.to_string() })
            .await
            .map_err(APIError::database_error)?;
    }
    first.downloaded.extend(second.downloaded);
    for unvoting in first.upvoted.clone().iter().filter(|map| second.upvoted.contains(map)) {
        let unvoting = data().await.database.query_one(MAPS_COLLECTION, doc! { "id": unvoting.to_string() })
            .await.map_err(APIError::database_error)?
            .ok_or(APIError::DatabaseError(Error::msg("Failed to find map while merging!")))?;
        unvote_for_map(&unvoting, &mut first).await?;
    }
    first.upvoted.extend(second.upvoted);
    data()
        .await
        .database
        .upload(USERS_COLLECTION, &first)
        .await
        .map_err(APIError::database_error)?;
    data()
        .await
        .database
        .remove(USERS_COLLECTION, doc! { "id": second.id.to_string() })
        .await
        .map_err(APIError::database_error)?;
    Ok(())
}

pub async fn get_discord_user(code: String) -> Result<DiscordUser, APIError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    let secret = env::var("CLIENT_SECRET").unwrap();
    let params = [
        ("client_id", "1298420686087262269"),
        ("client_secret", secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", &*code),
        (
            "redirect_uri",
            "https://beatblockbrowser.me/api/discordauth",
        ),
        ("scope", "identify"),
    ];
    let token = Client::builder()
        .default_headers(headers.clone())
        .build()
        .map_err(APIError::database_error)?
        .post("https://discord.com/api/v10/oauth2/token")
        .form(&params)
        .send()
        .await
        .map_err(APIError::database_error)?;
    let token: DiscordTokenRequest = get_response(token).await?;

    headers.insert(
        "Authorization",
        format!("Bearer {}", token.access_token).parse().unwrap(),
    );
    let user = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(APIError::database_error)?
        .get("https://discord.com/api/users/@me")
        .send()
        .await
        .map_err(APIError::database_error)?;
    let user: DiscordUser = get_response(user).await?;
    if !user.verified {
        return Err(APIError::AuthError(anyhow!("Your account's email isn't verified.")).into());
    }
    Ok(user)
}

async fn get_response<T: for<'a> Deserialize<'a>>(response: Response) -> Result<T, APIError> {
    let bytes = response.bytes().await.map_err(APIError::database_error)?;
    let string = String::from_utf8_lossy(&bytes).into_owned();
    let deserialized = serde_json::from_str(&string).map_err(APIError::database_error)?;
    Ok(deserialized)
}
