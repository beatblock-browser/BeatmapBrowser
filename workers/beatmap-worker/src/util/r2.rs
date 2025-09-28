use uuid::Uuid;
use worker::{Env, Result};

#[derive(Debug, serde::Serialize)]
pub struct UploadResult {
    pub download_url: String,
    pub thumbnail_url: String,
}

pub fn generate_urls(env: &Env, map_id: &Uuid) -> Result<UploadResult> {
    // R2 public base URL, e.g., https://example.r2.cloudflarestorage.com/bucket
    let base = env.var("R2_PUBLIC_URL").map(|v| v.to_string()).unwrap_or_else(|_| "https://r2.example.invalid/bucket".to_string());
    Ok(UploadResult {
        download_url: format!("{base}/maps/{map_id}.zip"),
        thumbnail_url: format!("{base}/thumbs/{map_id}.png"),
    })
}
