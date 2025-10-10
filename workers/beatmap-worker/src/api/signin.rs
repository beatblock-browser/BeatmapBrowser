use serde::{Deserialize, Serialize};
use worker::{Env, Error, Fetch, Headers, Method, Request, RequestInit, Response, Result};
use uuid::Uuid;
use base64::Engine;

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
    let mut uid: Option<String> = None;
    if let Ok(api_key) = env.var("FIREBASE_API_KEY") {
        let (ok, fetched_uid) = verify_firebase_and_uid(&payload.firebase_token, &api_key.to_string()).await?;
        if !ok {
            return json_response(400, &serde_json::json!({ "error": "Invalid Firebase token" }));
        }
        uid = fetched_uid;
    }

    if uid.is_none() {
        // Best-effort local decode of Firebase JWT to read 'sub'
        uid = decode_sub_from_jwt(&payload.firebase_token);
    }

    let Some(uid_str) = uid else {
        return json_response(400, &serde_json::json!({ "error": "Missing uid in Firebase token" }));
    };

    // Derive stable UUID v5 from UID
    // Fixed namespace for this application (random, but constant)
    let ns = Uuid::parse_str("38c8d7b9-585a-4c54-8d5a-6d3b9b2d1a11")
        .map_err(|e| Error::RustError(format!("invalid namespace uuid: {e}")))?;
    let user_uuid = Uuid::new_v5(&ns, uid_str.as_bytes());

    // Issue our JWT with sub = UUID string
    let jwt = generate_jwt(&env, &user_uuid.to_string())?;
    // Also set as HttpOnly session cookie
    let mut resp = crate::json_response(200, &SigninResponse { token: jwt.clone() })?;
    let cookie = format!(
        "session={}; Path=/; HttpOnly; Secure; SameSite=None; Max-Age={}",
        jwt,
        60 * 60 * 12 // 12 hours to align with JWT exp
    );
    resp.headers_mut().set("Set-Cookie", &cookie).ok();
    Ok(resp)
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
    let claims: Claims = match verify_jwt(&env, token) {
        Ok(c) => c,
        Err(e) => {
            return json_response(401, &serde_json::json!({
                "error": e.to_string()
            }));
        }
    };
    json_response(200, &claims)
}

async fn verify_firebase_and_uid(id_token: &str, api_key: &str) -> Result<(bool, Option<String>)> {
    // Use accounts:lookup to validate and fetch users[0].localId (UID)
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
    let mut resp = Fetch::Request(request).send().await?;
    if !(200..300).contains(&resp.status_code()) {
        return Ok((false, None));
    }
    let body_text = resp.text().await.map_err(|e| Error::RustError(format!("firebase text error: {e}")))?;
    let body: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|e| Error::RustError(format!("firebase json error: {e}")))?;
    let uid = body
        .get("users")
        .and_then(|u| u.as_array())
        .and_then(|arr| arr.first())
        .and_then(|u| u.get("localId"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok((true, uid))
}

fn decode_sub_from_jwt(id_token: &str) -> Option<String> {
    // Decode base64url payload (second segment)
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 { return None; }
    let payload = parts[1];
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload.as_bytes()).ok()?;
    let val: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    val.get("sub").and_then(|v| v.as_str()).map(|s| s.to_string())
}

// removed extract_user_id; replaced with decode_sub_from_jwt + uuid v5 derivation
