pub mod config;
pub mod llava;
pub mod matugen;
pub mod reload;
pub mod utils;
pub mod watch;

use anyhow::Result;
use std::path::PathBuf;
use crate::llava::Palette;

pub fn apply_once(image: PathBuf, mode: Option<String>, force: bool) -> Result<()> {
    let cfg = config::Config::load()?;
    let desired_mode = mode.unwrap_or_else(|| cfg.behavior.mode_default.clone());
    let actual_mode = if desired_mode == "auto" {
        let lum = utils::average_luminance(&image)?;
        utils::decide_mode(lum).to_string()
    } else {
        desired_mode
    };

    let cache_dir = utils::expand_tilde(&cfg.behavior.cache_dir);
    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join("cache.json");
    let mut cache: serde_json::Map<String, serde_json::Value> = if cache_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&cache_path)?)?
    } else {
        serde_json::Map::new()
    };
    let hash = utils::hash_file(&image)?;
    let mut palette: Option<Palette> = if !force {
        cache
            .get(&hash)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    } else {
        None
    };
    if palette.is_none() {
        palette = llava::analyze_image(&cfg, &image).ok();
        if let Some(p) = &palette {
            cache.insert(hash.clone(), serde_json::to_value(p)?);
            std::fs::write(&cache_path, serde_json::to_string_pretty(&cache)?)?;
        }
    }
    let palette = palette.unwrap_or_default();

    if !palette.is_empty() {
        let colors: Vec<String> = palette.values().cloned().collect();
        matugen::run_from_colors(&cfg, &colors, &actual_mode, false)?;
    } else {
        matugen::run_from_image(&cfg, &image, &actual_mode, false)?;
    }
    reload::run_hooks(&cfg, false);
    Ok(())
}
