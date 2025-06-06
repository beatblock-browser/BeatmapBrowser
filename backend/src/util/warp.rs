use crate::api::APIError;
use crate::schema::signin::UserToken;
use crate::schema::{BeatMap, User};
use crate::util::auth::verify_jwt;
use crate::util::data;
use crate::util::mongo::{MAPS_COLLECTION, TOKENS_COLLECTION, USERS_COLLECTION};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use anyhow::anyhow;
use cookie::time::Duration;
use cookie::{Cookie, SameSite};
use mongodb::bson::doc;
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use warp::http::header::AUTHORIZATION;
use warp::http::HeaderValue;
use warp::hyper::header::SET_COOKIE;
use warp::reject::Reject;
use warp::{reject, reply, Filter, Rejection, Reply};

pub fn extract_identifier() -> impl Filter<Extract = (UniqueIdentifier,), Error = Infallible> + Copy {
    warp::addr::remote()
        .map(|addr: Option<SocketAddr>| match addr.unwrap_or("0.0.0.0:0".parse().unwrap()) {
            SocketAddr::V4(ip) => UniqueIdentifier::Ipv4(ip.ip().clone()),
            SocketAddr::V6(ip) => UniqueIdentifier::Ipv6(ip.ip().clone()),
        })
}

pub fn check_ratelimit(action: SiteAction) -> impl Filter<Extract = ((),), Error = Rejection> + Copy {
    extract_identifier().and_then(move |identifier: UniqueIdentifier| async move {
        data().await.ratelimiter.lock().unwrap().check_limited(action, &identifier).map_err(reject::custom)
    })
}

pub async fn handle_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(api_error) = err.find::<APIError>() {
        match api_error {
            APIError::Ratelimited() => {},
            err => println!("Error: {:?}", err)
        }
        let response = reply::json(&serde_json::json!({
            "error": api_error.to_string()
        }));
        return Ok(reply::with_status(response, api_error.get_code()));
    }
    println!("Denied connection: {:?}", err);
    Err(err)
}

pub async fn get_user(token: String) -> Result<User, APIError> {
    let user_id: UserToken = data().await.database.query_one(TOKENS_COLLECTION, doc! { "user_token": token })
        .await
        .map_err(APIError::database_error)?
        .ok_or(APIError::AuthError(anyhow!("Invalid token!")))?;
    data().await.database.query_one(USERS_COLLECTION, doc! { "id": user_id.id.to_string() })
        .await
        .map_err(APIError::database_error)?
        .ok_or(APIError::AuthError(anyhow!("Invalid token!")))
}

pub async fn get_map(id: String) -> Result<BeatMap, APIError> {
    data().await.database.query_one(MAPS_COLLECTION, doc! { "id": id })
        .await
        .map_err(APIError::database_error)?
        .ok_or(APIError::AuthError(anyhow!("Invalid map!")))
}

pub trait Replyable {
    fn reply(self) -> impl Reply;
}

impl<T: Serialize> Replyable for T {
    fn reply(self) -> impl Reply {
        reply::json(&self)
    }
}

impl Reject for APIError {}

// Extracts the JWT from the Authorization header and verifies it
pub fn with_auth() -> impl Filter<Extract = (User,), Error = Rejection> + Clone {
    warp::header::<String>(AUTHORIZATION.as_str()).and_then(|auth_header: String| async move {
        if !auth_header.starts_with("Bearer ") {
            return Err(reject::custom(APIError::AuthError(anyhow!("Invalid authorization header format"))));
        }
        let token = auth_header.trim_start_matches("Bearer ").trim();
        match verify_jwt(token) {
            Ok(claims) => {
                let user = data().await.database.query::<User>(USERS_COLLECTION, doc! { "id": claims.sub})
                    .await
                    .map_err(APIError::database_error)?
                    .into_iter().next()
                    .ok_or_else(|| APIError::AuthError(anyhow!("User not found")))?;
                Ok(user)
            },
            Err(e) => Err(reject::custom(APIError::AuthError(e))),
        }
    })
}

// Helper function to create the cookie string
pub fn create_session_cookie(name: &str, value: &str) -> String {
    Cookie::build((name, value)) // Use tuple for name & value
        .path("/") // Available for the whole site
        .http_only(true) // Prevent JS access (important!)
        .secure(true) // Only send over HTTPS (important!)
        .same_site(SameSite::Lax) // Good balance for CSRF protection
        // Set expiration (e.g., 1 hour from now)
        .max_age(Duration::hours(1))
        // Or set an absolute expiry time:
        // .expires(OffsetDateTime::now_utc() + Duration::hours(1))
        .build() // Build the Cookie struct
        .to_string() // Convert to the string format for the header
}

// Helper function to create a JWT response with cookie
pub fn create_jwt_response(user: User) -> Result<impl Reply, Rejection> {
    use crate::util::auth::generate_jwt;

    let token = generate_jwt(&user.id.to_string())
        .map_err(|err| reject::custom(APIError::AuthError(err)))?;

    let cookie_string = create_session_cookie("jwt", &token);
    let cookie_header_value = HeaderValue::from_str(&cookie_string)
        .map_err(|err| <APIError as Into<Rejection>>::into(APIError::AuthError(err.into())))?;

    Ok(reply::with_header(user.reply(), SET_COOKIE, cookie_header_value))
}