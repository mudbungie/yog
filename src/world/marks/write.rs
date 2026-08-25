//! **Pointing a space at a branch** — the write half of §16.3, split from the
//! read at §12's budget on the seam the module's own doc draws: above is where
//! an agent's balls space *is* and what branch it currently tracks, here is the
//! one act that changes it.
//!
//! It stays balls' own file, balls' own key and balls' own precedence — yog
//! authors a layer-2 `config.toml` in full because a space is one agent's and
//! its only balls config is the branch, so a merge-preserving edit would be
//! machinery for a key nothing else ever writes. The write is logged to
//! `ops.jsonl` (§4.2) and the branch handed back is **re-read**: what landed,
//! never an echo of what was asked.

use std::io;
use std::path::Path;

use super::{Space, config_file, lawful};
use crate::opslog::{self, OpEntry};

/// Point a workspace's own space at `branch` (§16.3): write `tasks_branch` into
/// balls' layer-2 config for that space, log the write to `ops.jsonl` (§4.2, the
/// mutation-logging discipline), and hand back the branch **re-read** — what
/// landed, never an echo of what was asked.
pub fn apply(space: &Space, state_root: &Path, ts: &str, branch: &str) -> io::Result<String> {
    let path = config_file(&space.config);
    let outcome = if lawful(branch) {
        write_branch(&path, branch)
    } else {
        Err(io::Error::other(REFUSAL))
    };
    log_op(state_root, ts, &path, branch, &outcome)?;
    outcome?;
    Ok(space.branch())
}

/// The refusal an unlawful branch earns, said once — the grammar states it
/// before dispatch and [`apply`] states it again at the write, so a typed line
/// and a forced call cannot word the same fact differently.
pub const REFUSAL: &str =
    "name a store branch: one word, no quotes, and not balls' own landing branch (balls/config)";

/// Write `tasks_branch = "<branch>"` as the space's whole layer-2 config. The
/// file is yog's to author in full: a space is one agent's, and its only balls
/// config is the branch — so a merge-preserving edit would be machinery for a
/// key nothing else ever writes.
fn write_branch(path: &Path, branch: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, body(branch))
}

/// The config body, said once — the write emits it and [`parse_branch`] reads
/// it back. One key, quoted; [`lawful`] has already refused anything a quote
/// would have to escape.
pub fn body(branch: &str) -> String {
    format!("tasks_branch = \"{branch}\"\n")
}

/// Append the write's outcome to `ops.jsonl` (§4.2). A file write is not a
/// spawn, so it rides the §4.2 non-spawn step shape the start flow's own
/// `yog-step` rows use — the path is the subject, the exit says whether it
/// landed.
fn log_op(
    state_root: &Path,
    ts: &str,
    path: &Path,
    branch: &str,
    outcome: &io::Result<()>,
) -> io::Result<()> {
    let (exit, stderr) = match outcome {
        Ok(()) => (0, String::new()),
        Err(e) => (-3, e.to_string()),
    };
    opslog::append(
        state_root,
        &OpEntry {
            ts: ts.to_owned(),
            argv: vec!["yog-step".to_owned(), "marks".to_owned(), branch.to_owned()],
            cwd: path.display().to_string(),
            exit,
            stdout: String::new(),
            stderr,
            // The §16.3 knob's own pane states this outcome in place (§7.3,
            // bl-48f8), so no banner elsewhere repeats it.
            origin: crate::opslog::Origin::World,
        },
    )
}
