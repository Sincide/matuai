use crate::apply_once;
use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use glob::Pattern;

pub fn watch_dir(dir: PathBuf, pattern: Vec<String>, debounce_ms: u64, mode: Option<String>, force: bool) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;
    let patterns: Vec<Pattern> = pattern.into_iter().filter_map(|p| Pattern::new(&p).ok()).collect();
    loop {
        let event = match rx.recv()? {
            Ok(ev) => ev,
            Err(e) => { eprintln!("watch error: {}", e); continue; }
        };
        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
            std::thread::sleep(Duration::from_millis(debounce_ms));
            if let Some(path) = event.paths.first() {
                let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if patterns.iter().any(|pat| pat.matches(fname)) {
                    let _ = apply_once(path.clone(), mode.clone(), force);
                }
            }
        }
    }
}
