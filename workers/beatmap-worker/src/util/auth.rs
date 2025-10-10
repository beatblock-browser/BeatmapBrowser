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
        // Extend expiry to 12 hours to reduce frequent refresh prompts
        exp: (now + Duration::hours(12)).timestamp(),
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
    // Prefer Authorization header for backward compatibility
    if let Ok(Some(authorization)) = headers.get("Authorization") {
        let token = authorization.strip_prefix("Bearer ").unwrap_or("");
        if token.is_empty() {
            return Err(Error::RustError("missing bearer token".into()));
        }
        return verify_jwt(env, token);
    }

    // Fallback to HttpOnly cookie session: Cookie: session=<jwt>
    if let Ok(Some(cookie_header)) = headers.get("Cookie") {
        for part in cookie_header.split(';') {
            let trimmed = part.trim();
            if let Some(v) = trimmed.strip_prefix("session=") {
                let token = v.trim();
                if !token.is_empty() {
                    return verify_jwt(env, token);
                }
            }
        }
    }
    Err(Error::RustError("missing authorization".into()))
}

pub fn optional_auth(req: &Request, env: &Env) -> Option<Claims> {
    let headers = req.headers();
    if let Ok(Some(authorization)) = headers.get("Authorization") {
        let token = authorization.strip_prefix("Bearer ").unwrap_or("");
        if !token.is_empty() {
            if let Ok(claims) = verify_jwt(env, token) {
                return Some(claims);
            }
        }
    }
    if let Ok(Some(cookie_header)) = headers.get("Cookie") {
        for part in cookie_header.split(';') {
            let trimmed = part.trim();
            if let Some(v) = trimmed.strip_prefix("session=") {
                let token = v.trim();
                if !token.is_empty() {
                    if let Ok(claims) = verify_jwt(env, token) {
                        return Some(claims);
                    }
                }
            }
        }
    }
    None
}
