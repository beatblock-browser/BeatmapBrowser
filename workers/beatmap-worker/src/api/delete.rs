use serde::Deserialize;
use uuid::Uuid;
use worker::{Env, Request, Response, Result, Error};

use crate::json_response;
use crate::util::db::{Database, DatabaseBackend};
use crate::util::auth::require_auth;
use crate::util::roles::is_admin;

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub map_id: String,
}

pub async fn delete(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let payload: DeleteRequest = req.json().await?;
    let claims = require_auth(&req, &env)?;

    // Permission: allow if requester is a moderator OR admin
    if let Some(_map) = db.get_map_by_id(Uuid::parse_str(&payload.map_id).map_err(|e| Error::RustError(e.to_string()))?).await? {
        let requester = Uuid::parse_str(&claims.sub)
            .map_err(|e| Error::RustError(format!("invalid user id in token: {e}")))?;
        let is_admin_user = is_admin(&requester);
        let is_mod_user = if is_admin_user { true } else { db.is_user_moderator(&claims.sub).await.unwrap_or(false) };
        if !(is_mod_user || is_admin_user) {
            return json_response(403, &serde_json::json!({
                "error": "You do not have permission to perform this action"
            }));
        }
        // Soft-delete only; keep R2 assets so admins can restore
        db.set_map_deleted(&payload.map_id, true, Some(&claims.sub)).await?;
        return json_response(200, &serde_json::json!({ "status": "ok" }));
    }

    json_response(404, &serde_json::json!({ "error": "Beatmap not found" }))
}
