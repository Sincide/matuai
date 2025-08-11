use crate::config::Config;
use directories::BaseDirs;
use std::process::Command;

pub fn run_hooks(cfg: &Config, dry_run: bool) {
    if dry_run {
        println!("Would reload: hyprland?{} waybar?{} mako?{} kitty?{} fish?{}",
            cfg.reload.enable_hyprland,
            cfg.reload.enable_waybar,
            cfg.reload.enable_mako,
            cfg.reload.enable_kitty,
            cfg.reload.enable_fish);
        return;
    }
    if cfg.reload.enable_hyprland {
        let _ = Command::new("hyprctl").arg("reload").status();
    }
    if cfg.reload.enable_waybar {
        let pids = Command::new("pidof").arg("waybar").output();
        if let Ok(out) = pids {
            for pid in String::from_utf8_lossy(&out.stdout).split_whitespace() {
                let _ = Command::new("kill").arg("-USR2").arg(pid).status();
            }
        }
    }
    if cfg.reload.enable_mako {
        let _ = Command::new("makoctl").arg("reload").status();
    }
    if cfg.reload.enable_kitty {
        let _ = Command::new("kitty").args(["@", "set-colors", "-a", "~/.config/kitty/theme.conf"]).status();
    }
    if cfg.reload.enable_fish {
        if let Some(base) = BaseDirs::new() {
            let p = base.home_dir().join(".config/fish/conf.d/00-theme-vars.fish");
            if p.exists() {
                let _ = Command::new("fish").args(["-c", &format!("source {}", p.display())]).status();
            }
        }
    }
}
