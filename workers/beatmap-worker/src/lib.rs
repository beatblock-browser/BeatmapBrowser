use worker::{event, Context, Env, Method, Request, Response, Result};
use uuid::Uuid;

use crate::util::auth::optional_auth;
use crate::util::db::DatabaseBackend;
use crate::util::roles::is_admin;

mod api;
mod util;
mod parsing;

fn with_cors(result: Result<Response>, origin: &str) -> Result<Response> {
    match result {
        Ok(mut resp) => {
            add_cors(&mut resp, origin);
            Ok(resp)
        }
        Err(err) => {
            let mut resp = Response::from_json(&serde_json::json!({
                "error": err.to_string()
            }))?
            .with_status(500);
            add_cors(&mut resp, origin);
            Ok(resp)
        }
    }
}

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).ok();

    let path = req.path();
    let method = req.method().clone();

    // Determine Origin for CORS reflection (allow prod + localhost dev)
    let origin = req.headers().get("Origin").ok().flatten().unwrap_or_default();

    // Handle CORS preflight
    if method == Method::Options {
        let mut resp = Response::empty()?
            .with_status(204);
        add_cors(&mut resp, &origin);
        return Ok(resp);
    }

    match (method, path.as_str()) {
        (Method::Post, "/api/search") => with_cors(api::handle_search(req, env).await, &origin),
        (Method::Post, "/api/upvote") => with_cors(api::upvote::upvote(req, env).await, &origin),
        (Method::Post, "/api/unvote") => with_cors(api::upvote::unvote(req, env).await, &origin),
        (Method::Post, "/api/usersongs") => with_cors(api::usersongs::usersongs(req, env).await, &origin),
        (Method::Post, "/api/delete") => with_cors(api::delete::delete(req, env).await, &origin),
        (Method::Post, "/api/upload") => with_cors(api::upload::upload(req, env).await, &origin),
        (Method::Post, "/api/signin") => with_cors(api::signin::signin(req, env).await, &origin),
        (Method::Post, "/api/moderation/set") => with_cors(api::moderation::set_moderator(req, env).await, &origin),
        (Method::Post, "/api/moderation/restore") => with_cors(api::moderation::restore_map(req, env).await, &origin),
        (Method::Post, "/api/moderation/status") => with_cors(api::moderation::map_status(req, env).await, &origin),
        (Method::Post, "/api/check-duplicate") => with_cors(api::handle_check_duplicate(req, env).await, &origin),
        (Method::Get, "/api/ping") => with_cors(api::handle_ping(req, env).await, &origin),
        (Method::Get, "/api/me") => with_cors(api::signin::me(req, env).await, &origin),
        _ => {
            // Match dynamic route for /api/map/{map_id}
            if let Some(map_id) = path.strip_prefix("/api/map/") {
                return with_cors(api::handle_get_map(req, env, map_id.to_string()).await, &origin);
            }
            // Serve assets from R2 via worker for dev/local access
            if let Some(id) = path.strip_prefix("/thumbs/") {
                if let Some(id) = id.strip_suffix(".png") {
                    // check soft delete first; only serve if not deleted or requester is mod/admin
                    let db = DatabaseBackend::from_env(&env)?;
                    let is_deleted = db.is_map_deleted(id).await.unwrap_or(false);
                    if is_deleted {
                        let mut allowed = false;
                        if let Some(claims) = optional_auth(&req, &env) {
                            if let Ok(uid) = Uuid::parse_str(&claims.sub) {
                                if is_admin(&uid) || db.is_user_moderator(&claims.sub).await.unwrap_or(false) {
                                    allowed = true;
                                }
                            }
                        }
                        if !allowed {
                            let mut resp = json_response(404, &serde_json::json!({ "error": "Not Found" }))?;
                            add_cors(&mut resp, &origin);
                            return Ok(resp);
                        }
                    }
                    return with_cors(util::r2::serve_thumb(&env, id).await, &origin);
                }
            }
            if let Some(id) = path.strip_prefix("/maps/") {
                if let Some(id) = id.strip_suffix(".zip") {
                    // check soft delete first; only serve if not deleted or requester is mod/admin
                    let db = DatabaseBackend::from_env(&env)?;
                    let is_deleted = db.is_map_deleted(id).await.unwrap_or(false);
                    if is_deleted {
                        let mut allowed = false;
                        if let Some(claims) = optional_auth(&req, &env) {
                            if let Ok(uid) = uuid::Uuid::parse_str(&claims.sub) {
                                if is_admin(&uid) || db.is_user_moderator(&claims.sub).await.unwrap_or(false) {
                                    allowed = true;
                                }
                            }
                        }
                        if !allowed {
                            let mut resp = json_response(404, &serde_json::json!({ "error": "Not Found" }))?;
                            add_cors(&mut resp, &origin);
                            return Ok(resp);
                        }
                    }
                    return with_cors(util::r2::serve_zip(&env, id).await, &origin);
                }
            }
            let mut resp = json_response(404, &serde_json::json!({
                "error": "Not Found"
            }))?;
            add_cors(&mut resp, &origin);
            Ok(resp)
        }
    }
}

pub(crate) fn json_response<T: serde::Serialize>(status: u16, value: &T) -> Result<Response> {
    Ok(Response::from_json(value)?.with_status(status))
}

fn add_cors(resp: &mut Response, origin: &str) {
    // Reflect prod or localhost dev origins for credentialed requests
    let allowed_prod = "https://beatblockbrowser.me";
    let allowed_dev = "http://localhost:3000";
    let allow = if origin == allowed_prod || origin == allowed_dev { origin } else { allowed_prod };
    let headers = resp.headers_mut();
    let _ = headers.set("Access-Control-Allow-Origin", allow);
    let _ = headers.set("Access-Control-Allow-Credentials", "true");
    let _ = headers.set("Access-Control-Allow-Headers", "Content-Type, Authorization");
    let _ = headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
    let _ = headers.set("Access-Control-Max-Age", "86400");
    let _ = headers.append("Vary", "Origin");
}
