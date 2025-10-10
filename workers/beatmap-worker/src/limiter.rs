use js_sys::Date;
use worker::{durable_object, Env, Method, Request, Response, Result, State};

#[durable_object]
pub struct Limiter {
    state: State,
    _env: Env,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct BucketState {
    tokens: f64,
    last_ts: f64, // seconds epoch
}

impl Limiter {
    fn now_seconds() -> f64 {
        Date::now() / 1000.0
    }
}

impl worker::DurableObject for Limiter {
    fn new(state: State, env: Env) -> Self {
        Self { state, _env: env }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        if req.method() != Method::Post {
            return Response::error("Method Not Allowed", 405);
        }
        let url = req.url()?;
        let params = url.search_params();
        let rate: f64 = params.get("rate").and_then(|s| s.parse().ok()).unwrap_or(1.0); // tokens per minute
        let burst: f64 = params.get("burst").and_then(|s| s.parse().ok()).unwrap_or(3.0);

        let storage = self.state.storage();
        let mut st: BucketState = storage.get("bucket").await?.unwrap_or_default();

        // Refill based on elapsed time
        let now = Self::now_seconds();
        if st.last_ts == 0.0 { st.last_ts = now; }
        let elapsed_min = ((now - st.last_ts).max(0.0)) / 60.0;
        st.tokens = (st.tokens + elapsed_min * rate).min(burst);

        let allowed = if st.tokens >= 1.0 { st.tokens -= 1.0; true } else { false };
        st.last_ts = now;
        // Persist with a TTL to auto-cleanup idle objects (e.g., 1 day)
        storage.put_with_ttl("bucket", &st, 86400).await?;

        if allowed {
            Response::ok("ok")
        } else {
            Response::error("Too Many Requests", 429)
        }
    }
}
