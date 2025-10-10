use uuid::Uuid;
use worker::{Env, FormEntry, Request, Response, Result};

use crate::json_response;
use crate::util::auth::require_auth;
use crate::util::db::{Database, DatabaseBackend, NewMap, UpdateMap, DIFFICULTIES_TABLE};
use crate::util::r2::{generate_urls, upload_thumbnail, upload_zip};
use crate::util::image::make_thumbnail_png;
use crate::parsing::{validate_and_extract};

// No JSON payload accepted; only multipart/form-data with a 'beatmap' file is supported.

pub async fn upload(mut req: Request, env: Env) -> Result<Response> {
    let db: DatabaseBackend = DatabaseBackend::from_env(&env)?;
    let claims = require_auth(&req, &env)?;

    // Hard cap on compressed upload size via Content-Length (best-effort)
    const MAX_COMPRESSED: u64 = 200_000_000; // 200 MB
    if let Some(len_str) = req.headers().get("content-length")? {
        if let Ok(len) = len_str.parse::<u64>() {
            if len > MAX_COMPRESSED {
                return json_response(413, &serde_json::json!({ "error": "Payload too large" }));
            }
        }
    }

    // Simple per-user rate limiting using recent successful uploads in D1 (last 10 minutes)
    // Limits spikes and saves CPU. Consider Durable Object for stronger limits.
    let charter_uid = Uuid::parse_str(&claims.sub)
        .map_err(|_| worker::Error::RustError("Invalid user id in token".into()))?;
    {
        let since = (chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339();
        let sql = "SELECT COUNT(1) as cnt FROM maps m WHERE m.charter_uid = ?1 AND m.upload_date >= ?2";
        let d1 = env.d1("beatblockbrowser")?;
        let stmt = d1.prepare(sql);
        let res = stmt
            .bind(&[worker::wasm_bindgen::JsValue::from_str(&charter_uid.to_string()), worker::wasm_bindgen::JsValue::from_str(&since)])?
            .all()
            .await?;
        let rows = res.results::<std::collections::BTreeMap<String, serde_json::Value>>()
            .unwrap_or_default();
        let recent = rows.first()
            .and_then(|r| r.get("cnt"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if recent >= 3 {
            return json_response(429, &serde_json::json!({ "error": "Rate limit exceeded. Try again later." }));
        }
    }

    let ct = req
        .headers()
        .get("content-type")?
        .unwrap_or_default();

    // NOTE: We intentionally do NOT call the Durable Object limiter here, so that
    // failed validation attempts do not count against rate limits.

    // Expect multipart/form-data with file: beatmap (required). Metadata will be extracted from archive.
    if ct.to_lowercase().contains("multipart/form-data") {
        let form = req.form_data().await?;
        let mut zip_bytes = match form.get("beatmap") {
            Some(FormEntry::File(f)) => f.bytes().await?.to_vec(),
            _ => {
                return json_response(400, &serde_json::json!({ "error": "Missing required file: beatmap" }));
            }
        };

        // Create new map document id
        let id = Uuid::new_v4();

        // Validate and extract metadata from archive (verify-only). On failure, return a clear 400.
        let details = match validate_and_extract(&mut zip_bytes[..]) {
            Ok(d) => d,
            Err(msg) => {
                return json_response(400, &serde_json::json!({
                    "error": msg
                }));
            }
        };

        // Determine if this is an update to an existing map by identity
        let existing = db
            .find_map_by_identity(
                &details.level_data.song_name,
                &details.level_data.artist,
                &details.level_data.charter,
                charter_uid,
            )
            .await?;

        let target_id = existing.unwrap_or(id);

        // Upload sanitized ZIP to R2 (overwrites if exists)
        upload_zip(&env, &target_id, &zip_bytes, None).await?;
        let had_thumb = if let Some(img) = details.image.as_ref() {
            let png = make_thumbnail_png(img, &details.level_data.bg_data)
                .map_err(|e| worker::Error::RustError(format!("thumbnail encode: {e}")))?;
            upload_thumbnail(&env, &target_id, &png, Some("image/png")).await?;
            true
        } else { false };

        // If new, create; if existing, update metadata
        if existing.is_none() {
            let new_map = NewMap {
                id: target_id,
                song: &details.level_data.song_name,
                artist: &details.level_data.artist,
                charter: &details.level_data.charter,
                charter_uid,
                description: &details.level_data.description,
                artist_list: &details.level_data.artist_list,
                image: had_thumb,
            };
            db.create_map(&new_map).await?;
        } else {
            let updated = UpdateMap {
                id: target_id,
                song: &details.level_data.song_name,
                artist: &details.level_data.artist,
                charter: &details.level_data.charter,
                description: &details.level_data.description,
                artist_list: &details.level_data.artist_list,
                image: if had_thumb { Some(true) } else { None },
            };
            db.update_map_metadata(&updated).await?;
        }

        // Persist parsed difficulties if present
        let mut diffs_resp: Vec<crate::api::LevelVariant> = Vec::new();
        if !details.level_data.variants.is_empty() {
            let d1 = env.d1("beatblockbrowser")?;
            // Replace difficulties entirely for target map
            let del_stmt = d1.prepare(&format!(
                "DELETE FROM {DIFFICULTIES_TABLE} WHERE map_id = ?1"
            ));
            del_stmt
                .bind(&[worker::wasm_bindgen::JsValue::from_str(&target_id.to_string())])?
                .run()
                .await
                .map_err(|e| worker::Error::RustError(format!("d1 delete difficulties: {e}")))?;
            for v in &details.level_data.variants {
                // Map parsing::LevelVariant -> api::LevelVariant for response
                diffs_resp.push(crate::api::LevelVariant { display: v.display.clone(), difficulty: v.difficulty });
                let stmt = d1.prepare(&format!(
                    "INSERT INTO {DIFFICULTIES_TABLE}(map_id, display, difficulty) VALUES (?1, ?2, ?3)"
                ));
                stmt.bind(&[
                    worker::wasm_bindgen::JsValue::from_str(&target_id.to_string()),
                    worker::wasm_bindgen::JsValue::from_str(&v.display),
                    worker::wasm_bindgen::JsValue::from_f64(v.difficulty),
                ])?.run().await.map_err(|e| worker::Error::RustError(format!("d1 insert difficulty: {e}")))?;
            }
        }

        // Durable Object rate-limit: per user+ip (implemented in JS DO `Limiter`).
        // Consume AFTER a successful upload so failed attempts don't count.
        if let Ok(ns) = env.durable_object("LIMITER") {
            let ip = req.headers().get("CF-Connecting-IP").ok().flatten().unwrap_or_else(|| "unknown".to_string());
            let key = format!("{}:{}", charter_uid, ip);
            if let Ok(id) = ns.id_from_name(&key) {
                if let Ok(stub) = id.get_stub() {
                    let mut init = worker::RequestInit::new();
                    init.with_method(worker::Method::Post);
                    let do_req = worker::Request::new_with_init("https://limiter/consume?rate=1&burst=3", &init)?;
                    let resp = stub.fetch_with_request(do_req).await?;
                    if resp.status_code() == 429 {
                        // If limiter rejects here, we already have a successful upload; return 200 anyway
                        // to avoid confusing clients, as the upload has been processed.
                    }
                }
            }
        }

        let urls = generate_urls(&env, &target_id)?;
        return json_response(200, &serde_json::json!({
            "id": target_id,
            "song": details.level_data.song_name,
            "artist": details.level_data.artist,
            "charter": details.level_data.charter,
            "charter_uid": charter_uid,
            "difficulties": diffs_resp,
            "download_url": urls.download_url,
            "thumbnail_url": urls.thumbnail_url,
            "updated": existing.is_some(),
        }));
    }

    json_response(400, &serde_json::json!({ "error": "Expected multipart/form-data" }))
}

