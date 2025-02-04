use crate::api::upvote::unvote_for_map;
use crate::api::APIError;
use crate::util::database::{AccountLink, User, UserID};
use crate::util::mongo::{MAPS_COLLECTION, TOKENS_COLLECTION, USERS_COLLECTION};
use crate::util::warp::Replyable;
use crate::util::{data, get_user_from_link};
use anyhow::Error;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use firebase_auth::FirebaseUser;
use mongodb::bson::doc;
use rand::rngs::OsRng;
use rand::RngCore;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::env;
use warp::{Rejection, Reply};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordTokenRequest {
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: String,
    pub verified: bool,
}

pub async fn discord_signin(code: String) -> Result<impl Reply, Rejection> {
    let account = get_user_from_link(AccountLink::Discord(
        get_discord_user(code).await?.id.parse().unwrap(),
    ))
    .await?;
    Ok(account.id.to_string().reply())
}

pub async fn discord_sync(user: User, code: String) -> Result<impl Reply, Rejection> {
    let other = get_user_from_link(AccountLink::Discord(
        get_discord_user(code).await?.id.parse().unwrap(),
    ))
    .await?;
    merge(user, other).await?;
    Ok("Ok".reply())
}

pub async fn google_signin(code: String) -> Result<impl Reply, Rejection> {
    let user: FirebaseUser = data()
        .await
        .auth
        .verify(&code)
        .map_err(|err| APIError::AuthError(err.to_string()))?;
    let account = get_user_from_link(AccountLink::Google(user.user_id)).await?;
    Ok(get_token(account.id)
        .await
        .map_err(APIError::database_error)?
        .reply())
}

pub async fn google_sync(user: User, code: String) -> Result<impl Reply, Rejection> {
    let firebase_user: FirebaseUser = data()
        .await
        .auth
        .verify(&code)
        .map_err(|err| APIError::AuthError(err.to_string()))?;
    let other = get_user_from_link(AccountLink::Google(firebase_user.user_id)).await?;
    merge(user, other).await?;
    Ok("Ok".reply())
}

pub async fn merge(mut first: User, second: User) -> Result<(), APIError> {
    for map in second.maps {
        data()
            .await
            .database
            .update(MAPS_COLLECTION, doc!{ "id": map }, doc! { "charter_uid", first.id })
            .await
            .map_err(APIError::database_error)?;
    }
    first.downloaded.extend(second.downloaded);
    for unvoting in first.upvoted.clone().iter().filter(|map| second.upvoted.contains(map)) {
        let unvoting = data().await.database.query_one(MAPS_COLLECTION, doc! { "id", unvoting })
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
        .remove(USERS_COLLECTION, doc! { "id": second.id })
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
        return Err(APIError::AuthError("Your account's email isn't verified.".to_string()).into());
    }
    Ok(user)
}

pub async fn get_token(user: UserID) -> Result<String, Error> {
    if let Some(token) = data()
        .await
        .database
        .query_one::<UserToken>(TOKENS_COLLECTION, doc! { "id": user.to_string() })
        .await?
    {
        return Ok(token.token);
    }

    let mut rng = OsRng; // Uses the operating system's randomness
    let mut buffer = [0u8; 32]; // 32 bytes = 256 bits
    rng.fill_bytes(&mut buffer);

    // Encode the random bytes as a Base64 string
    let token = BASE64_STANDARD.encode(&buffer);
    data()
        .await
        .database
        .upload(
            TOKENS_COLLECTION,
            &UserToken {
                id: user,
                token: token.clone(),
            }
        )
        .await?;
    Ok(token)
}

async fn get_response<T: for<'a> Deserialize<'a>>(response: Response) -> Result<T, APIError> {
    let bytes = response.bytes().await.map_err(APIError::database_error)?;
    let string = String::from_utf8_lossy(&bytes).into_owned();
    let deserialized = serde_json::from_str(&string).map_err(|e| {
        println!("{}", string);
        APIError::database_error(e)
    })?;
    Ok(deserialized)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserToken {
    pub id: UserID,
    #[serde(rename = "user_token")]
    token: String,
}
