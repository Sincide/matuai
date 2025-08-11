use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use std::path::Path;
use anyhow::Result;
use regex::Regex;

pub fn validate_hex(hex: &str) -> bool {
    let re = Regex::new(r"^#[0-9A-Fa-f]{6}$").unwrap();
    re.is_match(hex)
}

pub fn average_luminance(path: &Path) -> Result<f32> {
    let img = image::open(path)?;
    let img = downscale(img);
    let mut total = 0f32;
    for pixel in img.pixels() {
        let [r, g, b, _] = pixel.2 .0;
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        // simple luminance
        total += 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
    Ok(total / (img.width() * img.height()) as f32)
}

fn downscale(img: DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let max_dim = w.max(h);
    if max_dim <= 64 {
        img
    } else {
        let scale = 64.0 / max_dim as f32;
        let nw = (w as f32 * scale) as u32;
        let nh = (h as f32 * scale) as u32;
        img.resize_exact(nw, nh, FilterType::Triangle)
    }
}

pub fn decide_mode(lum: f32) -> &'static str {
    if lum < 0.5 { "dark" } else { "light" }
}

pub fn hash_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

use std::path::PathBuf;

pub fn expand_tilde(p: &str) -> PathBuf {
    if let Some(stripped) = p.strip_prefix("~") {
        if let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf()) {
            return home.join(stripped.trim_start_matches('/'));
        }
    }
    PathBuf::from(p)
}
