use std::env;
use crate::api::APIError;
use crate::util::ratelimiter::UniqueIdentifier;
use crate::SiteData;
use anyhow::Error;
use hyper::body::Incoming;
use hyper::Request;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use reqwest::header::HeaderMap;
use crate::util::get_user;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordAuth {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordTokenRequest {
    pub access_token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: String,
    pub verified: bool,
}

pub async fn discord_signin(
    request: Request<Incoming>,
    _identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    let Ok(arguments) =
        serde_urlencoded::from_str::<DiscordAuth>(request.uri().query().unwrap_or(""))
    else {
        return Err(APIError::QueryError(Error::msg(
            "Invalid search arguments!",
        )));
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/x-www-form-urlencoded".parse().unwrap());
    let secret = env::var("CLIENT_SECRET").unwrap();
    let params = [
        ("client_id", "1298420686087262269"),
        ("client_secret", secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", &*arguments.code),
        ("redirect_uri", "https://beatblockbrowser.me/api/discordauth"),
        ("scope", "identify"),
    ];
    let token = Client::builder().default_headers(headers.clone()).build().map_err(APIError::database_error)?
        .post("https://discord.com/api/v10/oauth2/token").form(&params).send().await
        .map_err(APIError::database_error)?;
    let token: DiscordTokenRequest = get_response(token).await?;

    headers.insert("Authorization", format!("Bearer {}", token.access_token).parse().unwrap());
    let user = Client::builder().default_headers(headers).build().map_err(APIError::database_error)?
        .get("https://discord.com/api/users/@me").send().await.map_err(APIError::database_error)?;
    let user: DiscordUser = get_response(user).await?;
    if !user.verified {
        return Err(APIError::AuthError("Your account's email isn't verified.".to_string()));
    }
    let account = get_user(false, user.id, &data.db).await?;
    
    Ok(account.id.unwrap().to_string())
}

async fn get_response<T: for<'a> Deserialize<'a>>(response: Response) -> Result<T, APIError> {
    let bytes = response.bytes().await.map_err(APIError::database_error)?;
    let string = String::from_utf8_lossy(&bytes).into_owned();
    let deserialized = serde_json::from_str(&string).map_err(APIError::database_error)?;
    Ok(deserialized)
}