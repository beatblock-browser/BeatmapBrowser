use worker::{event, Context, Env, Method, Request, Response, Result};

mod api;
mod util;

fn with_cors(result: Result<Response>) -> Result<Response> {
    match result {
        Ok(mut resp) => {
            add_cors(&mut resp);
            Ok(resp)
        }
        Err(err) => {
            let mut resp = Response::from_json(&serde_json::json!({
                "error": err.to_string()
            }))?
            .with_status(500);
            add_cors(&mut resp);
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

    // Handle CORS preflight
    if method == Method::Options {
        let mut resp = Response::empty()?
            .with_status(204);
        add_cors(&mut resp);
        return Ok(resp);
    }

    match (method, path.as_str()) {
        (Method::Post, "/api/search") => with_cors(api::handle_search(req, env).await),
        (Method::Post, "/api/upvote") => with_cors(api::upvote::upvote(req, env).await),
        (Method::Post, "/api/unvote") => with_cors(api::upvote::unvote(req, env).await),
        (Method::Post, "/api/usersongs") => with_cors(api::usersongs::usersongs(req, env).await),
        (Method::Post, "/api/delete") => with_cors(api::delete::delete(req, env).await),
        (Method::Post, "/api/upload") => with_cors(api::upload::upload(req, env).await),
        (Method::Post, "/api/signin") => with_cors(api::signin::signin(req, env).await),
        (Method::Get, "/api/ping") => with_cors(api::handle_ping(req, env).await),
        (Method::Get, "/api/me") => with_cors(api::signin::me(req, env).await),
        _ => {
            // Match dynamic route for /api/map/{map_id}
            if let Some(map_id) = path.strip_prefix("/api/map/") {
                return with_cors(api::handle_get_map(req, env, map_id.to_string()).await);
            }
            let mut resp = json_response(404, &serde_json::json!({
                "error": "Not Found"
            }))?;
            add_cors(&mut resp);
            Ok(resp)
        }
    }
}

pub(crate) fn json_response<T: serde::Serialize>(status: u16, value: &T) -> Result<Response> {
    Ok(Response::from_json(value)?.with_status(status))
}

fn add_cors(resp: &mut Response) {
    let headers = resp.headers_mut();
    let _ = headers.set("Access-Control-Allow-Origin", "*");
    // Allow common headers plus wildcard to cover dev tool headers
    let _ = headers.set("Access-Control-Allow-Headers", "*, Content-Type, Authorization");
    let _ = headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
    let _ = headers.set("Access-Control-Max-Age", "86400");
    let _ = headers.append("Vary", "Origin");
}
