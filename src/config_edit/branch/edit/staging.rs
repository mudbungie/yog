//! The staged half of a config-branch edit (DESIGN §9.3 step 1, §5.2 step 5):
//! where a drafted file waits for lernie's `$EDITOR` callback to collect it,
//! and how a dir the callback never reached is swept.
//!
//! Split off [`super`] at §12's pre-split band on the seam that module's own
//! numbered flow draws — steps 1 and 5 are a scratch-dir lifecycle with no
//! subprocess in them, while steps 2–4 are an argv, an environment and a spawn.
//! The `<nonce>/` dir is the only thing the two halves share, and it crosses as
//! a path.

/// Staging dirs untouched for longer than this are swept at startup (§5.2) —
/// the bound is [`crate::scratch::STALE_SECS`], one home for both halves of
/// that sentence.
use crate::scratch::STALE_SECS;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A drafted config file: a checkout-relative path and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftFile {
    pub rel_path: String,
    pub bytes: Vec<u8>,
}

/// A collision-safe staging nonce: `<pid>-<counter>`. No clock / randomness
/// (per the task) — the pid scopes it to this process and the monotonic
/// counter to this call, so concurrent edits never share a dir.
pub fn next_nonce() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{n}", std::process::id())
}

/// Write the drafted `files` into `<staging_root>/<nonce>/` (creating parent
/// dirs) and return that staging dir. An empty `files` still creates the
/// dir — the shim then copies nothing and lernie declines the empty commit.
pub fn stage_files(
    staging_root: &Path,
    nonce: &str,
    files: &[DraftFile],
) -> std::io::Result<PathBuf> {
    let dir = staging_root.join(nonce);
    std::fs::create_dir_all(&dir)?;
    for f in files {
        let dest = dir.join(&f.rel_path);
        // A path joined under `dir` always has a parent (at worst `dir`
        // itself); the fallback keeps the staging write panic-free.
        std::fs::create_dir_all(dest.parent().unwrap_or(&dir))?;
        std::fs::write(&dest, &f.bytes)?;
    }
    Ok(dir)
}

/// Pure decision (clock-injected): the staging dirs whose mtime is more than
/// 24 h before `now_secs`. `now_secs` and each mtime are unix seconds, so
/// every arm is deterministic under test.
pub fn stale_staging(now_secs: i64, dirs: &[(PathBuf, i64)]) -> Vec<PathBuf> {
    dirs.iter()
        .filter(|(_, mtime)| now_secs - mtime > STALE_SECS)
        .map(|(p, _)| p.clone())
        .collect()
}

/// Sweep `<stage_root>/*`: best-effort delete every `<nonce>/` dir untouched
/// for over 24 h (§5.2 startup sweep). A missing root is a no-op; the wall
/// clock is the caller's (main.rs), keeping the decision ([`stale_staging`])
/// pure. Returns the dirs decided stale.
pub fn sweep_staging(stage_root: &Path, now_secs: i64) -> Vec<PathBuf> {
    let stale = stale_staging(now_secs, &staging_dirs(stage_root));
    for dir in &stale {
        let _ = std::fs::remove_dir_all(dir);
    }
    stale
}

/// Enumerate `<stage_root>/*` sub-dirs paired with their mtime (unix secs).
/// A missing root, or any entry that cannot be stat'd (a dangling symlink, a
/// racing unlink), contributes nothing — enumeration is best-effort.
fn staging_dirs(stage_root: &Path) -> Vec<(PathBuf, i64)> {
    use std::os::unix::fs::MetadataExt;
    let Ok(entries) = std::fs::read_dir(stage_root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata()
            && meta.is_dir()
        {
            out.push((entry.path(), meta.mtime()));
        }
    }
    out
}

#[cfg(test)]
mod tests;
