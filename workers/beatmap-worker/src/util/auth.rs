use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use worker::{Env, Error, Request, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn secret(env: &Env) -> Result<String> {
    Ok(env.var("JWT_SECRET")?.to_string())
}

pub fn generate_jwt(env: &Env, user_id: &str) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::hours(1)).timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret(env)?.as_bytes()),
    )
    .map_err(|e| Error::RustError(format!("jwt encode error: {e}")))?;
    Ok(token)
}

pub fn verify_jwt(env: &Env, token: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret(env)?.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| Error::RustError(format!("jwt decode error: {e}")))?;
    Ok(data.claims)
}

pub fn require_auth(req: &Request, env: &Env) -> Result<Claims> {
    let headers = req.headers();
    let Ok(Some(authorization)) = headers.get("Authorization") else {
        return Err(Error::RustError("missing authorization".into()));
    };
    let token = authorization.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        return Err(Error::RustError("missing bearer token".into()));
    }
    verify_jwt(env, token)
}
