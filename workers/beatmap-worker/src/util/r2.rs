use uuid::Uuid;
use worker::{Bucket, Env, HttpMetadata, Response, Result};

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

fn maps_bucket(env: &Env) -> Result<Bucket> {
    // Binding configured in wrangler.toml [[r2_buckets]] binding = "MAPS"
    env.bucket("MAPS")
}

pub async fn upload_zip(env: &Env, map_id: &Uuid, bytes: &[u8], content_type: Option<&str>) -> Result<()> {
    let bucket = maps_bucket(env)?;
    let key = format!("maps/{map_id}.zip");
    let mut put = bucket.put(&key, bytes.to_vec());
    let ct = content_type.unwrap_or("application/zip");
    put = put.http_metadata(HttpMetadata {
        content_type: Some(ct.to_string()),
        cache_control: Some("public, max-age=31536000, immutable".to_string()),
        ..Default::default()
    });
    put.execute().await?;
    Ok(())
}

pub async fn upload_thumbnail(env: &Env, map_id: &Uuid, bytes: &[u8], content_type: Option<&str>) -> Result<()> {
    let bucket = maps_bucket(env)?;
    let key = format!("thumbs/{map_id}.png");
    let mut put = bucket.put(&key, bytes.to_vec());
    let ct = content_type.unwrap_or("image/png");
    put = put.http_metadata(HttpMetadata {
        content_type: Some(ct.to_string()),
        cache_control: Some("public, max-age=31536000, immutable".to_string()),
        ..Default::default()
    });
    put.execute().await?;
    Ok(())
}

pub async fn delete_zip(env: &Env, map_id: &Uuid) -> Result<()> {
    let bucket = maps_bucket(env)?;
    let key = format!("maps/{map_id}.zip");
    bucket.delete(&key).await?;
    Ok(())
}

pub async fn delete_thumbnail(env: &Env, map_id: &Uuid) -> Result<()> {
    let bucket = maps_bucket(env)?;
    let key = format!("thumbs/{map_id}.png");
    bucket.delete(&key).await?;
    Ok(())
}

pub async fn serve_thumb(env: &Env, id: &str) -> Result<Response> {
    let bucket = maps_bucket(env)?;
    let key = format!("thumbs/{}.png", id);
    if let Some(obj) = bucket.get(&key).execute().await? {
        if let Some(ob) = obj.body() {
            let body = ob.bytes().await.unwrap_or_default();
            let mut resp = Response::from_bytes(body)?;
            let headers = resp.headers_mut();
            let _ = headers.set("Content-Type", "image/png");
            let _ = headers.set("Cache-Control", "public, max-age=31536000, immutable");
            return Ok(resp);
        }
        // No body available
        return Response::error("Not Found", 404);
    } else {
        Response::error("Not Found", 404)
    }
}

pub async fn serve_zip(env: &Env, id: &str) -> Result<Response> {
    let bucket = maps_bucket(env)?;
    let key = format!("maps/{}.zip", id);
    if let Some(obj) = bucket.get(&key).execute().await? {
        if let Some(ob) = obj.body() {
            let body = ob.bytes().await.unwrap_or_default();
            let mut resp = Response::from_bytes(body)?;
            let headers = resp.headers_mut();
            let _ = headers.set("Content-Type", "application/zip");
            let _ = headers.set("Cache-Control", "public, max-age=31536000, immutable");
            return Ok(resp);
        }
        return Response::error("Not Found", 404);
    } else {
        Response::error("Not Found", 404)
    }
}
