use crate::config::Config;
use crate::utils::validate_hex;
use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use image::imageops::FilterType;
use image::GenericImageView;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;

const PROMPT: &str = r##"System: You are a wallpaper-to-UI theming assistant. Pick exactly one accent color suitable as a UI seed color.
User: Analyze the attached image. Return ONLY valid JSON in this exact schema:
{"seed_hex":"#RRGGBB"}
Rules:
- seed_hex must be a 7-character hex like "#82A3FF".
- Choose a color that works as an accent for both text and UI elements.
- Avoid extremely low-contrast grayish colors and neon extremes.
- No extra text. Only the JSON object."##;

#[derive(Deserialize)]
struct LlavaResp {
    response: String,
}

pub fn analyze_image(cfg: &Config, path: &Path) -> Result<String> {
    let img = image::open(path)?;
    let img = downscale(img);
    let mut cursor = Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageOutputFormat::Png)?;
    let b64 = BASE64.encode(cursor.into_inner());

    let client = Client::builder()
        .timeout(Duration::from_millis(cfg.llava.timeout_ms))
        .build()?;

    let payload = serde_json::json!({
        "model": cfg.llava.model,
        "prompt": PROMPT,
        "images": [b64],
        "stream": false,
        "options": {"temperature": cfg.llava.temperature}
    });

    for attempt in 0..2 {
        let resp = client
            .post(format!("{}/api/generate", cfg.llava.endpoint))
            .json(&payload)
            .send();
        let resp = match resp {
            Ok(r) => r,
            Err(e) => return Err(anyhow!("llava request failed: {e}")),
        };
        let text = resp.text()?;
        // attempt to parse seed directly
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(seed) = v.get("seed_hex").and_then(|v| v.as_str()) {
                if validate_hex(seed) {
                    return Ok(seed.to_string());
                }
            }
        }
        let parsed: Result<LlavaResp, _> = serde_json::from_str(&text);
        if let Ok(parsed) = parsed {
            if validate_hex(&parsed.response) {
                return Ok(parsed.response);
            }
            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&parsed.response) {
                if let Some(seed) = j.get("seed_hex").and_then(|v| v.as_str()) {
                    if validate_hex(seed) {
                        return Ok(seed.to_string());
                    }
                }
            }
        }
        if attempt == 0 {
            continue;
        } else {
            break;
        }
    }
    Err(anyhow!("llava returned unusable data"))
}

fn downscale(img: image::DynamicImage) -> image::DynamicImage {
    let (w, h) = img.dimensions();
    let max = w.max(h);
    if max <= 1024 {
        img
    } else {
        let scale = 1024f32 / max as f32;
        let nw = (w as f32 * scale) as u32;
        let nh = (h as f32 * scale) as u32;
        img.resize(nw, nh, FilterType::Triangle)
    }
}
