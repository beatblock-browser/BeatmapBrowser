use worker::{Env, Request, Response, Result};

use crate::json_response;
use crate::util::db::{Database, DatabaseBackend};
use crate::util::auth::require_auth;

pub async fn usersongs(req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let claims = require_auth(&req, &env)?;
    let results = db.usersongs(&claims.sub).await?;
    json_response(200, &serde_json::json!({ "results": results }))
}
