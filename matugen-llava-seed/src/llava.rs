use crate::config::Config;
use crate::utils::validate_hex;
use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use image::imageops::FilterType;
use image::GenericImageView;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;

const PROMPT: &str = r##"System: You are a wallpaper-to-UI theming assistant.
User: Analyze the attached image and respond ONLY with a JSON object in this exact schema:
{
  "primary_hex":"#RRGGBB",
  "secondary_hex":"#RRGGBB",
  "tertiary_hex":"#RRGGBB",
  "accent1_hex":"#RRGGBB",
  "accent2_hex":"#RRGGBB",
  "neutral1_hex":"#RRGGBB",
  "neutral2_hex":"#RRGGBB"
}
Rules:
- Every value must be a 7-character hex string like "#82A3FF".
- Colors should be suitable for UI theming with good contrast.
- Avoid neon extremes and near-grayscale tones.
- Return only the JSON object, no extra text."##;

pub type Palette = BTreeMap<String, String>;

#[derive(Deserialize)]
struct LlavaResp {
    response: String,
}

/// Attempt to parse a palette JSON string into a map of color values.
/// Only keys that contain valid 7-character hex colors are included.
pub fn parse_palette(text: &str) -> Option<Palette> {
    let parsed: serde_json::Value = serde_json::from_str(text).ok()?;
    let obj = parsed.as_object()?;
    let keys = [
        "primary_hex",
        "secondary_hex",
        "tertiary_hex",
        "accent1_hex",
        "accent2_hex",
        "neutral1_hex",
        "neutral2_hex",
    ];
    let mut out = BTreeMap::new();
    for key in keys.iter() {
        if let Some(val) = obj.get(*key).and_then(|v| v.as_str()) {
            if validate_hex(val) {
                out.insert((*key).to_string(), val.to_string());
            }
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

pub fn analyze_image(cfg: &Config, path: &Path) -> Result<Palette> {
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

        if let Some(palette) = parse_palette(&text) {
            return Ok(palette);
        }

        let parsed: Result<LlavaResp, _> = serde_json::from_str(&text);
        if let Ok(parsed) = parsed {
            if let Some(palette) = parse_palette(&parsed.response) {
                return Ok(palette);
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
