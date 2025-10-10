use serde::Deserialize;
use serde::Serialize;
use std::io::{Cursor, Read};
use std::path::{Component, Path};
use zip::ZipArchive;

pub const MAX_SIZE: u32 = 200_000_000;
pub const MAX_FILES: usize = 2_000;
pub const MAX_FILE_SIZE: u64 = 25_000_000; // 25 MB per file cap

// Allows misspelling, just here to block exes and other malicious files
pub const EXTENSIONS: [&str; 12] = [
    "png", "jpg", "jpeg", "webp", "mp3", "bmp", "ogg", "oog", "wav", "json", "md", "txt",
];

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BackgroundData {
    #[serde(default)]
    pub image: String,
    // Optional channel remaps (RGB triplets) for color swapping
    #[serde(default)]
    pub red_channel: Option<[u8; 3]>,
    #[serde(default)]
    pub green_channel: Option<[u8; 3]>,
    #[serde(default)]
    pub blue_channel: Option<[u8; 3]>,
    #[serde(default)]
    pub magenta_channel: Option<[u8; 3]>,
    #[serde(default)]
    pub cyan_channel: Option<[u8; 3]>,
    #[serde(default)]
    pub yellow_channel: Option<[u8; 3]>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LevelMetadata {
    #[serde(default)]
    pub song_name: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub charter: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub artist_list: String,
    #[serde(default)]
    pub difficulty: Option<f64>,
    #[serde(default)]
    pub variants: Vec<LevelVariant>,
    #[serde(default)]
    pub bg_data: Option<BackgroundData>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LevelVariant {
    pub display: String,
    pub difficulty: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct LevelData {
    #[serde(default)]
    pub metadata: LevelMetadata,
}

#[derive(Debug, Clone)]
pub struct FileData {
    pub level_data: LevelMetadata,
    pub image: Option<Vec<u8>>, // raw image bytes if available
}

pub fn check_path(path: &Path) -> Result<(), String> {
    if path.components().any(|c| c == Component::ParentDir) {
        Err("File path contains directory traversal".into())
    } else {
        Ok(())
    }
}

fn is_legal_name(name: &str) -> Result<bool, String> {
    check_path(Path::new(name))?;
    Ok(
        name.ends_with('/')
            || name.ends_with('\\')
            || name
                .split('.')
                .next_back()
                .map(|ext| EXTENSIONS.contains(&ext))
                .unwrap_or(false),
    )
}

pub fn parse_archive(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Result<FileData, String> {
    fn read_file(archive: &mut ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<Vec<u8>> {
        if let Ok(mut f) = archive.by_name(name) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() { Some(buf) } else { None }
        } else { None }
    }

    fn base_dir_if_single(archive: &mut ZipArchive<Cursor<&[u8]>>) -> Option<String> {
        // Determine if all entries are within a single top-level directory like "name/..."
        let mut base: Option<String> = None;
        let count = archive.len();
        for i in 0..count {
            let Ok(f) = archive.by_index(i) else { continue };
            let name = f.name();
            // Normalize to forward slashes (zip uses '/'), skip empty names
            let n = name.trim_start_matches("./");
            // Directory entry at root like "dir/" is fine
            if let Some(pos) = n.find('/') {
                let top = &n[..pos];
                if top.is_empty() { return None; }
                match &base {
                    Some(b) => { if b != top { return None; } }
                    None => base = Some(top.to_string()),
                }
            } else {
                // A file at the root means not single-nested
                return None;
            }
        }
        base
    }

    fn read_with_base(archive: &mut ZipArchive<Cursor<&[u8]>>, name: &str, base: &Option<String>) -> Option<Vec<u8>> {
        if let Some(buf) = read_file(archive, name) { return Some(buf); }
        if let Some(b) = base {
            let joined = format!("{}/{}", b, name);
            return read_file(archive, &joined);
        }
        None
    }

    // Try level.json first, then manifest.json; if missing or empty, fallback to scanning other json files
    let base = base_dir_if_single(archive);
    let mut meta_json = if let Some(buf) = read_with_base(archive, "level.json", &base) {
        serde_json::from_slice::<LevelData>(&buf)
            .map(|d| d.metadata)
            .map_err(|e| e.to_string())?
    } else if let Some(buf) = read_with_base(archive, "manifest.json", &base) {
        match serde_json::from_slice::<LevelData>(&buf) {
            Ok(d) => d.metadata,
            Err(_) => serde_json::from_slice::<LevelMetadata>(&buf).map_err(|e| e.to_string())?,
        }
    } else {
        LevelMetadata::default()
    };

    // Normalize a key: lowercase and keep only [a-z0-9]
    fn norm_key(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>()
    }

    // Helper: recursively find first STRING by candidate keys (no array concatenation)
    // Also apply length caps to avoid swallowing large unrelated data
    fn find_string_by_keys(v: &serde_json::Value, keys: &[&str], max_len: usize) -> Option<String> {
        let keyset: std::collections::HashSet<String> = keys.iter().map(|k| norm_key(k)).collect();
        match v {
            serde_json::Value::String(s) => {
                let t = s.trim();
                if t.is_empty() { return None; }
                let clipped = if t.len() > max_len { &t[..max_len] } else { t };
                Some(clipped.to_string())
            }
            serde_json::Value::Object(map) => {
                // Prefer known containers first
                for container in ["metadata", "info", "song"] {
                    if let Some(inner) = map.get(container) {
                        if let Some(s) = find_string_by_keys(inner, keys, max_len) { return Some(s); }
                    }
                }
                // Then search direct keys
                for (k, vv) in map.iter() {
                    let kl = norm_key(k);
                    if keyset.contains(&kl) {
                        if let Some(s) = find_string_by_keys(vv, keys, max_len) { return Some(s); }
                    }
                }
                // Finally, shallow search children objects only (avoid arrays)
                for vv in map.values() {
                    if vv.is_object() {
                        if let Some(s) = find_string_by_keys(vv, keys, max_len) { return Some(s); }
                    }
                }
                None
            }
            _ => None,
        }
    }

    // If metadata is still empty, scan common json files and try to extract fields
    let needs_fill = meta_json.song_name.is_empty() && meta_json.artist.is_empty() && meta_json.charter.is_empty();
    if needs_fill {
        // Collect all json entries, prefer known names
        let mut json_entries: Vec<String> = Vec::new();
        let count = archive.len();
        for i in 0..count {
            if let Ok(f) = archive.by_index(i) {
                let name = f.name().to_string();
                if name.to_lowercase().ends_with(".json") { json_entries.push(name); }
            }
        }
        let prefer = ["info.json", "song.json", "beatmap.json", "metadata.json"]; 
        json_entries.sort_by_key(|n| {
            let nl = n.to_lowercase();
            prefer.iter().position(|p| nl.ends_with(p)).unwrap_or(999)
        });

        for jn in json_entries {
            let data = if let Some(buf) = read_with_base(archive, &jn, &base) { buf } else { continue };
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&data) {
                if meta_json.song_name.is_empty() {
                    if let Some(s) = find_string_by_keys(&val, &["song","song_name","songname","title","name","music","level_name","levelname"], 200) { meta_json.song_name = s; }
                }
                if meta_json.artist.is_empty() {
                    if let Some(s) = find_string_by_keys(&val, &["artist","artist_name","artistname","author","creator","mapper","composer"], 200) { meta_json.artist = s; }
                }
                if meta_json.charter.is_empty() {
                    if let Some(s) = find_string_by_keys(&val, &["charter","charter_name","chartername","chartauthor","mapper","author","creator","creatorname"], 200) { meta_json.charter = s; }
                }
                if meta_json.description.is_empty() {
                    if let Some(s) = find_string_by_keys(&val, &["description","desc"], 500) { meta_json.description = s; }
                }
                if meta_json.artist_list.is_empty() {
                    if let Some(s) = find_string_by_keys(&val, &["artist_list","artists"], 300) { meta_json.artist_list = s; }
                }

                // Attempt to extract difficulty variants
                if meta_json.variants.is_empty() {
                    // Walk the object looking for arrays under common keys
                    fn collect_variants(v: &serde_json::Value, out: &mut Vec<LevelVariant>) {
                        if let serde_json::Value::Object(map) = v {
                            for (k, vv) in map.iter() {
                                let kl = norm_key(k);
                                if ["difficulties","levels","variants"].contains(&kl.as_str()) {
                                    if let serde_json::Value::Array(arr) = vv {
                                        for item in arr {
                                            if let serde_json::Value::Object(o) = item {
                                                let mut display: Option<String> = None;
                                                let mut difficulty: Option<f64> = None;
                                                if let Some(s) = o.get("display").and_then(|x| x.as_str()) { display = Some(s.to_string()); }
                                                if let Some(s) = o.get("name").and_then(|x| x.as_str()) { if display.is_none() { display = Some(s.to_string()); } }
                                                if let Some(s) = o.get("title").and_then(|x| x.as_str()) { if display.is_none() { display = Some(s.to_string()); } }
                                                if let Some(d) = o.get("difficulty").and_then(|x| x.as_f64()) { difficulty = Some(d); }
                                                if let Some(d) = o.get("level").and_then(|x| x.as_f64()) { if difficulty.is_none() { difficulty = Some(d); } }
                                                if let Some(d) = o.get("value").and_then(|x| x.as_f64()) { if difficulty.is_none() { difficulty = Some(d); } }
                                                if let Some(disp) = display {
                                                    out.push(LevelVariant { display: disp, difficulty: difficulty.unwrap_or(0.0) });
                                                }
                                            } else if let serde_json::Value::String(s) = item {
                                                let disp = s.trim();
                                                if !disp.is_empty() { out.push(LevelVariant { display: disp.to_string(), difficulty: 0.0 }); }
                                            }
                                        }
                                    }
                                }
                                // Recurse into child objects
                                if vv.is_object() { collect_variants(vv, out); }
                            }
                        }
                    }
                    let mut vars: Vec<LevelVariant> = Vec::new();
                    collect_variants(&val, &mut vars);
                    // Keep unique displays and cap to reasonable number
                    if !vars.is_empty() {
                        let mut seen = std::collections::HashSet::new();
                        let mut unique: Vec<LevelVariant> = Vec::new();
                        for v in vars {
                            if seen.insert(v.display.clone()) { unique.push(v); }
                            if unique.len() >= 12 { break; }
                        }
                        meta_json.variants = unique;
                    }
                }
                // Stop early if we have basics
                if !meta_json.song_name.is_empty() || !meta_json.artist.is_empty() || !meta_json.charter.is_empty() { break; }
            }
        }
    }

    // Try to extract background image if referenced
    let mut image: Option<Vec<u8>> = None;
    if let Some(bg) = meta_json.bg_data.as_ref() {
        if !bg.image.is_empty() {
            if let Some(buf) = read_with_base(archive, &bg.image, &base) {
                image = Some(buf);
            }
        }
    }

    // If no image from metadata, pick the first image file in the archive
    if image.is_none() {
        let mut names: Vec<String> = Vec::new();
        let count = archive.len();
        for i in 0..count {
            if let Ok(f) = archive.by_index(i) {
                names.push(f.name().to_string());
            }
        }
        for n in names {
            let nl = n.to_lowercase();
            if nl.ends_with(".png") || nl.ends_with(".jpg") || nl.ends_with(".jpeg") || nl.ends_with(".webp") || nl.ends_with(".bmp") {
                if let Some(buf) = read_with_base(archive, &n, &base) {
                    image = Some(buf);
                    break;
                }
            }
        }
    }

    // Sanitize textual fields to avoid injection vectors
    fn sanitize_text(input: &str, max_len: usize) -> String {
        let mut out = String::with_capacity(input.len().min(max_len));
        for ch in input.chars() {
            // Allow basic printable characters except angle brackets and control chars
            if ch.is_control() { continue; }
            if matches!(ch, '<' | '>' ) { continue; }
            out.push(ch);
            if out.len() >= max_len { break; }
        }
        let trimmed = out.trim();
        // Collapse multiple whitespace
        let mut collapsed = String::with_capacity(trimmed.len());
        let mut prev_space = false;
        for ch in trimmed.chars() {
            let is_space = ch.is_whitespace();
            if is_space {
                if !prev_space { collapsed.push(' '); }
            } else {
                collapsed.push(ch);
            }
            prev_space = is_space;
        }
        collapsed
    }

    meta_json.song_name = sanitize_text(&meta_json.song_name, 200);
    meta_json.artist = sanitize_text(&meta_json.artist, 200);
    meta_json.charter = sanitize_text(&meta_json.charter, 200);
    meta_json.description = sanitize_text(&meta_json.description, 500);
    meta_json.artist_list = sanitize_text(&meta_json.artist_list, 300);

    if !meta_json.variants.is_empty() {
        for v in &mut meta_json.variants {
            v.display = sanitize_text(&v.display, 64);
        }
        // Drop empties after sanitize
        meta_json.variants.retain(|v| !v.display.is_empty());
    }

    Ok(FileData {
        level_data: meta_json,
        image,
    })
}

// Verify-only path: confirm the archive is already sanitized according to our policy
pub fn verify_zip_sanitized(bytes: &[u8]) -> Result<(), String> {
    if !bytes.starts_with(b"PK") {
        return Err("Unsupported archive type".into());
    }
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    let mut total_size: u64 = 0;
    let mut names: Vec<String> = Vec::new();
    let count = archive.len();
    for i in 0..count {
        if let Ok(f) = archive.by_index(i) {
            names.push(f.name().to_string());
        }
    }
    if names.len() > MAX_FILES {
        return Err(format!("Too many files in archive ({} > {})", names.len(), MAX_FILES));
    }
    for name in names {
        if !is_legal_name(&name)? { return Err(format!("Illegal file path: {}", name)); }
        let file = archive.by_name(&name).map_err(|e| e.to_string())?;
        let size = file.size();
        if size > MAX_FILE_SIZE {
            return Err(format!("File '{}' exceeds per-file size limit ({} > {})", name, size, MAX_FILE_SIZE));
        }
        if (total_size + size).max(size) > (MAX_SIZE as u64) * 2 {
            return Err("Uncompressed file size is too large".into());
        }
        total_size += size;
    }
    Ok(())
}

pub fn validate_and_extract(beatmap_data: &mut [u8]) -> Result<FileData, String> {
    // Quick signature check (ZIP magic)
    if !beatmap_data.starts_with(b"PK") {
        return Err("Unsupported archive type".into());
    }

    // Parse metadata before sanitization to find image path
    let cursor_before = Cursor::new(&beatmap_data[..]);
    let mut archive_before = ZipArchive::new(cursor_before).map_err(|e| e.to_string())?;
    let details = parse_archive(&mut archive_before)?;

    // Verify-only: ensure the client-submitted archive already complies. Do not repackage.
    // If you want server-side repackaging, call `check_and_sanitize_zip(beatmap_data)` instead.
    verify_zip_sanitized(&beatmap_data[..])?;

    Ok(details)
}
