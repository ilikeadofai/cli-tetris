//! Persist high score under ~/.cli-tetris-hiscore (no extra deps).

use std::fs;
use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".cli-tetris-hiscore"))
}

pub fn load() -> u64 {
    let Some(p) = path() else {
        return 0;
    };
    fs::read_to_string(p)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

/// If `score` beats the stored high score, save it. Returns the best of the two.
pub fn update_if_better(score: u64) -> u64 {
    let best = load();
    if score > best {
        if let Some(p) = path() {
            let _ = fs::write(p, score.to_string());
        }
        score
    } else {
        best
    }
}
