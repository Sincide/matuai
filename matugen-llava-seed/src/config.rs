use serde::Deserialize;
use std::fs;
use directories::ProjectDirs;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub llava: LlavaConfig,
    pub matugen: MatugenConfig,
    pub behavior: BehaviorConfig,
    pub reload: ReloadConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LlavaConfig {
    pub endpoint: String,
    pub model: String,
    pub temperature: f32,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatugenConfig {
    pub args_color: Vec<String>,
    pub args_image: Vec<String>,
    pub extra_args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BehaviorConfig {
    pub mode_default: String,
    pub vibrancy_clamp: Option<f32>,
    pub cache_dir: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReloadConfig {
    pub enable_hyprland: bool,
    pub enable_waybar: bool,
    pub enable_mako: bool,
    pub enable_kitty: bool,
    pub enable_fish: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llava: LlavaConfig::default(),
            matugen: MatugenConfig::default(),
            behavior: BehaviorConfig::default(),
            reload: ReloadConfig::default(),
        }
    }
}

impl Default for LlavaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:11434".into(),
            model: "llava:latest".into(),
            temperature: 0.1,
            timeout_ms: 3500,
        }
    }
}

impl Default for MatugenConfig {
    fn default() -> Self {
        Self {
            args_color: vec!["color".into()],
            args_image: vec!["image".into()],
            extra_args: vec![],
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            mode_default: "auto".into(),
            vibrancy_clamp: Some(0.85),
            cache_dir: "~/.cache/matugen-llava-seed".into(),
            log_level: "info".into(),
        }
    }
}

impl Default for ReloadConfig {
    fn default() -> Self {
        Self {
            enable_hyprland: true,
            enable_waybar: true,
            enable_mako: true,
            enable_kitty: true,
            enable_fish: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let proj = ProjectDirs::from("", "", "matugen-llava-seed")
            .context("unable to locate config directory")?;
        let path = proj.config_dir().join("config.toml");
        let cfg = if path.exists() {
            let text = fs::read_to_string(&path)?;
            toml::from_str(&text)?
        } else {
            Config::default()
        };
        Ok(cfg)
    }
}
