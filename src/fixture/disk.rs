//! **How a byte gets written** — the four primitives every writer above spends,
//! each answering a failure as a sentence naming its own path.
//!
//! **Nothing here panics.** A fixture that aborted mid-lay would leave a
//! half-world a harness then renders, so every effect is a `Result` and the
//! whole lay stops on the first one.
//!
//! The only `git` that runs is [`crate::git_env::git`] with both config homes
//! pointed at `/dev/null`: a fixture commit must not read the operator's
//! `core.hooksPath`, and a `GIT_DIR` inherited from a hook would land these
//! commits in the outer repo — the scrub `git_env` exists for.

use std::fs;
use std::path::Path;

/// Backdate `path` to `unix` seconds.
pub(super) fn stamp(path: &Path, unix: i64) -> Result<(), String> {
    let when = std::time::UNIX_EPOCH + std::time::Duration::from_secs(unix.unsigned_abs());
    let times = fs::FileTimes::new().set_modified(when);
    fs::File::options()
        .write(true)
        .open(path)
        .and_then(|f| f.set_times(times))
        .map_err(|e| format!("stamp {}: {e}", path.display()))
}

pub(super) fn mkdir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("create {}: {e}", path.display()))
}

pub(super) fn write(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        mkdir(parent)?;
    }
    fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

pub(super) fn display(path: &Path) -> String {
    path.display().to_string()
}

/// One `git`, config-free and optionally dated. `when` stamps both the author
/// and committer dates in git's raw format, which is what makes two runs of one
/// recipe produce the same commit graph.
pub(super) fn git(dir: &Path, args: &[&str], when: Option<i64>) -> Result<(), String> {
    let mut cmd = crate::git_env::git();
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null");
    cmd.env("GIT_CONFIG_SYSTEM", "/dev/null");
    cmd.env("GIT_AUTHOR_NAME", "yog fixture");
    cmd.env("GIT_AUTHOR_EMAIL", "fixture@yog.invalid");
    cmd.env("GIT_COMMITTER_NAME", "yog fixture");
    cmd.env("GIT_COMMITTER_EMAIL", "fixture@yog.invalid");
    if let Some(unix) = when {
        let stamp = format!("@{unix} +0000");
        cmd.env("GIT_AUTHOR_DATE", &stamp);
        cmd.env("GIT_COMMITTER_DATE", &stamp);
    }
    let status = crate::git_env::status(cmd.arg("-C").arg(dir).args(args))
        .map_err(|e| format!("git {args:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {args:?}: {status}"))
    }
}

#[cfg(test)]
mod tests;
