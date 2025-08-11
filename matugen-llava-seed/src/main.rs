use clap::{Parser, Subcommand};
use std::path::PathBuf;
use matugen_llava_seed::{apply_once, config::Config, llava, utils, reload, watch};
use anyhow::Result;

#[derive(Parser)]
#[command(version, about="LLaVA seed picker for Matugen")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
    #[arg(long, global=true)]
    dry_run: bool,
    #[arg(long, global=true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Analyze { #[arg(long)] image: PathBuf, #[arg(long)] mode: Option<String> },
    Apply { #[arg(long)] image: PathBuf, #[arg(long)] mode: Option<String>, #[arg(long)] force: bool },
    Watch { #[arg(long)] dir: PathBuf, #[arg(long, value_delimiter= ',')] pattern: Option<Vec<String>>, #[arg(long, default_value_t=400)] debounce_ms: u64, #[arg(long)] mode: Option<String>, #[arg(long)] force: bool },
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.verbose {
        tracing_subscriber::fmt().with_env_filter("debug").init();
    } else {
        tracing_subscriber::fmt().with_env_filter("info").init();
    }
    match cli.cmd {
        Commands::Analyze { image, mode } => analyze_cmd(image, mode),
        Commands::Apply { image, mode, force } => apply_cmd(image, mode, force, cli.dry_run),
        Commands::Watch { dir, pattern, debounce_ms, mode, force } => watch::watch_dir(dir, pattern.unwrap_or_else(|| vec!["*.jpg".into(), "*.png".into()]), debounce_ms, mode, force),
        Commands::Validate => validate_cmd(),
    }
}

fn analyze_cmd(image: PathBuf, _mode: Option<String>) -> Result<()> {
    let cfg = Config::load()?;
    let palette = llava::analyze_image(&cfg, &image)?;
    println!("{}", serde_json::to_string_pretty(&palette)?);
    Ok(())
}

fn apply_cmd(image: PathBuf, mode: Option<String>, force: bool, dry: bool) -> Result<()> {
    if dry {
        let cfg = Config::load()?;
        let desired_mode = mode.unwrap_or_else(|| cfg.behavior.mode_default.clone());
        let lum = utils::average_luminance(&image)?;
        let actual_mode = if desired_mode == "auto" { utils::decide_mode(lum).to_string() } else { desired_mode };
        let palette = llava::analyze_image(&cfg, &image).unwrap_or_default();
        let valid = !palette.is_empty() && palette.values().all(|c| utils::validate_hex(c));
        println!(
            "dry-run: palette={:?} mode={} matugen from colors?{}",
            palette,
            actual_mode,
            valid
        );
        println!("Matugen extra args {:?}", cfg.matugen.extra_args);
        reload::run_hooks(&cfg, true);
        return Ok(());
    }
    apply_once(image, mode, force)
}

fn validate_cmd() -> Result<()> {
    let required = ["matugen", "kitty", "waybar", "makoctl", "hyprctl", "fish"];
    for cmd in required { if which::which(cmd).is_err() { println!("missing executable: {}", cmd); } }
    Ok(())
}
