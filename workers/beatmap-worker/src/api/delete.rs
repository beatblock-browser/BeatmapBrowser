use serde::Deserialize;
use worker::{Env, Request, Response, Result, Error};

use crate::json_response;
use crate::util::db::{Database, DatabaseBackend};
use crate::util::auth::require_auth;

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub map_id: String,
}

pub async fn delete(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let payload: DeleteRequest = req.json().await?;
    let claims = require_auth(&req, &env)?;

    // Permission: allow if requester is the charter of the map
    if let Some(map) = db.get_map_by_id(uuid::Uuid::parse_str(&payload.map_id).map_err(|e| Error::RustError(e.to_string()))?).await? {
        let requester = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|e| Error::RustError(format!("invalid user id in token: {e}")))?;
        if map.charter_uid != requester {
            return json_response(400, &serde_json::json!({
                "error": "You do not have permission to perform this action"
            }));
        }
        db.delete_map(&payload.map_id).await?;
        return json_response(200, &serde_json::json!({ "status": "ok" }));
    }

    json_response(404, &serde_json::json!({ "error": "Beatmap not found" }))
}
