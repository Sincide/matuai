use crate::config::Config;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

pub fn run_from_colors(cfg: &Config, seeds: &[String], mode: &str, dry_run: bool) -> Result<()> {
    for seed in seeds {
        let mut args = cfg.matugen.args_color.clone();
        args.push(seed.to_string());
        args.extend(cfg.matugen.extra_args.clone());
        run_matugen(args, mode, dry_run)?;
    }
    Ok(())
}

pub fn run_from_image(cfg: &Config, image: &Path, mode: &str, dry_run: bool) -> Result<()> {
    let mut args = cfg.matugen.args_image.clone();
    args.push(image.to_string_lossy().to_string());
    args.extend(cfg.matugen.extra_args.clone());
    run_matugen(args, mode, dry_run)
}

fn run_matugen(args: Vec<String>, mode: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Matugen would run with args: {:?} mode:{}", args, mode);
        return Ok(());
    }
    let status = Command::new("matugen")
        .env("MATUGEN_MODE", mode)
        .args(&args)
        .status()
        .map_err(|e| anyhow!("failed to spawn matugen: {e}"))?;
    if !status.success() {
        return Err(anyhow!("matugen exited with status {status}"));
    }
    Ok(())
}
