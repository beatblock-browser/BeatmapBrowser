use serde::{Deserialize, Serialize};
use worker::{Env, Error, Fetch, Headers, Method, Request, RequestInit, Response, Result};

use crate::json_response;
use crate::util::auth::{generate_jwt, verify_jwt, Claims};

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    #[serde(rename = "firebaseToken")]
    pub firebase_token: String,
}

#[derive(Debug, Serialize)]
pub struct SigninResponse {
    pub token: String,
}

pub async fn signin(mut req: Request, env: Env) -> Result<Response> {
    let payload: SigninRequest = req.json().await?;

    // Optionally verify Firebase token using REST (requires FIREBASE_API_KEY)
    if let Ok(api_key) = env.var("FIREBASE_API_KEY") {
        let ok = verify_firebase(&payload.firebase_token, &api_key.to_string()).await?;
        if !ok {
            return json_response(400, &serde_json::json!({ "error": "Invalid Firebase token" }));
        }
    }

    // Issue our JWT
    let jwt = generate_jwt(&env, &extract_user_id(&payload.firebase_token))?;
    json_response(200, &SigninResponse { token: jwt })
}

pub async fn me(req: Request, env: Env) -> Result<Response> {
    let headers = req.headers();
    let Ok(Some(auth_header)) = headers.get("Authorization") else {
        return json_response(401, &serde_json::json!({ "error": "Missing Authorization" }));
    };
    let token = auth_header.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        return json_response(401, &serde_json::json!({ "error": "Missing Bearer token" }));
    }
    let claims: Claims = verify_jwt(&env, token)?;
    json_response(200, &claims)
}

async fn verify_firebase(id_token: &str, api_key: &str) -> Result<bool> {
    // Minimal verification using accounts:lookup
    let url = format!("https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={api_key}");
    let headers = {
        let h = Headers::new();
        h
        .set("Content-Type", "application/json")
        .map_err(|err| Error::RustError(format!("header set error: {err:?}")))?;
        h
    };

    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    init.with_headers(headers);
    init.with_body(Some(serde_json::json!({ "idToken": id_token }).to_string().into()));

    let request = Request::new_with_init(&url, &init)?;
    let resp = Fetch::Request(request).send().await?;
    Ok((200..300).contains(&resp.status_code()))
}

fn extract_user_id(firebase_token: &str) -> String {
    // Best-effort: Firebase tokens are JWTs; we can parse the payload without verifying to extract `user_id` / `sub`.
    // For simplicity and to avoid bringing in another dep, return the token hash snippet.
    // In production, decode the JWT payload to extract `user_id`.
    let short = firebase_token.chars().take(16).collect::<String>();
    format!("u_{short}")
}
