use anyhow::Error;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

// Define claims structure for JWT payload
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // Subject (user ID)
    pub exp: u64,       // Expiration time
    pub iat: u64,       // Issued at
}

#[cfg(debug_assertions)]
fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret_for_development".to_string())
}

#[cfg(not(debug_assertions))]
fn jwt_secret() -> String {
    env::var("JWT_SECRET").expect("No JWT secret provided!")
}

// Generate a new JWT token for a user
pub fn generate_jwt(user_id: &str) -> Result<String, Error> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let expiry = now + 3600;
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiry,
        iat: now,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes())
    )?;
    
    Ok(token)
}

// Verify and decode a JWT token
pub fn verify_jwt(token: &str) -> Result<Claims, Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default()
    )?;
    
    Ok(token_data.claims)
}
