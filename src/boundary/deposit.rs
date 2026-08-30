//! The headless transport (§8.5): a gesture is a **create-only file** in the
//! yog-watched `<state_root>/gestures/` inbox — the litany deposit discipline
//! (I4) applied to yog itself. Delivery is `rename(2)`; temps are `.`-prefixed
//! dotfiles the consumer never lists; the audit is the deposit plus the
//! `ops.jsonl` rows the dispatch writes (§4.2).
//!
//! Lifecycle: `<id>.json` (deposited) → `claimed/<id>.json` (taken — the
//! atomic rename is the claim, so two consumers on one world never double-run
//! a gesture) → `replies/<id>.json` (answered). A deposit with no reply is
//! simply not yet converged (I0): the next consumer pass takes it, whichever
//! yog process that is. A crash between claim and reply leaves the claimed
//! file as debris naming exactly what was in flight — re-deposit to re-run.
//!
//! **The id is claimed, never guessed** ([`mint`], bl-aa9f). A depositor's id
//! is also its reply key, so two depositors holding one id is two callers
//! reading one answer. No local guess is unique across a shared world — a
//! clock second and a pid repeat freely across process namespaces — so the
//! world's own filesystem is the arbiter: the depositor **reserves its reply
//! slot** with an exclusive create, and the reservation it wins *is* its id.

use serde_json::Value;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The inbox directory under the yog state root (§4.2's sibling).
pub const GESTURES_DIR: &str = "gestures";
/// Where a taken deposit moves — the claim, by rename.
const CLAIMED_DIR: &str = "claimed";
/// Where answers land, keyed by the deposit's id.
const REPLIES_DIR: &str = "replies";
/// The one listed extension; anything else in the inbox is ignored.
const EXT: &str = ".json";

/// The inbox root: `<state_root>/gestures/`.
pub fn gestures_dir(state_root: &Path) -> PathBuf {
    state_root.join(GESTURES_DIR)
}

/// The reply file a deposit `id` earns: `<state_root>/gestures/replies/<id>.json`.
pub fn reply_path(state_root: &Path, id: &str) -> PathBuf {
    gestures_dir(state_root)
        .join(REPLIES_DIR)
        .join(format!("{id}{EXT}"))
}

/// A depositor-chosen id's validity: non-empty, no path separators, no leading
/// dot (temps are dotfiles), no embedded extension games. The id is the file
/// name and the reply key — nothing more.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Claim a deposit id no other process writing this world can hold, by
/// reserving its reply slot: `<seed>-<n>`, `n` rising until the exclusive
/// create of `replies/<id>.json` wins. The reservation is an unparseable empty
/// file, which [`read_reply`] already reads as *not yet* — and it is never
/// removed, so an id retired by a claim or an answer is still spent and can
/// never be handed out twice (bl-aa9f). `seed` only keeps the names legible
/// and time-ordered; uniqueness is the filesystem's, not the seed's.
pub fn mint(state_root: &Path, seed: &str) -> io::Result<String> {
    if !valid_id(seed) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid gesture id seed {seed:?}"),
        ));
    }
    let dir = gestures_dir(state_root).join(REPLIES_DIR);
    fs::create_dir_all(&dir)?;
    let mut n: u64 = 0;
    loop {
        let id = format!("{seed}-{n}");
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(format!("{id}{EXT}")))
        {
            Ok(_) => return Ok(id),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => n = n.saturating_add(1),
            Err(e) => return Err(e),
        }
    }
}

/// Deposit one gesture envelope, create-only: dotfile temp, then `rename` to
/// `<id>.json`. An already-present id refuses — a deposit is never replayed
/// in place, and two depositors minting one id is the error it looks like.
pub fn deposit(state_root: &Path, id: &str, gesture: &Value) -> io::Result<PathBuf> {
    if !valid_id(id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid gesture id {id:?}"),
        ));
    }
    let dir = gestures_dir(state_root);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{id}{EXT}"));
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("gesture {id:?} already deposited"),
        ));
    }
    let tmp = dir.join(format!(".{id}{EXT}.tmp"));
    fs::write(&tmp, gesture.to_string().as_bytes())?;
    fs::rename(&tmp, &path)?;
    Ok(path)
}

/// The waiting deposits: every non-dot `*.json` directly in the inbox, as
/// `(id, path)`, name-ordered so two consumers walk one order (I9). A missing
/// inbox is the empty set — the general path with no inputs.
pub fn pending(state_root: &Path) -> Vec<(String, PathBuf)> {
    let dir = gestures_dir(state_root);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(EXT)?;
            if !valid_id(id) {
                return None;
            }
            Some((id.to_owned(), e.path()))
        })
        .filter(|(_, p)| p.is_file())
        .collect();
    out.sort();
    out
}

/// Claim one pending deposit by renaming it into `claimed/`. The rename is the
/// mutual exclusion: the loser of a race gets the error and moves on. Returns
/// the claimed path, whose content is now this consumer's to answer.
pub fn claim(state_root: &Path, id: &str) -> io::Result<PathBuf> {
    let dir = gestures_dir(state_root);
    let claimed_dir = dir.join(CLAIMED_DIR);
    fs::create_dir_all(&claimed_dir)?;
    let from = dir.join(format!("{id}{EXT}"));
    let to = claimed_dir.join(format!("{id}{EXT}"));
    fs::rename(&from, &to)?;
    Ok(to)
}

/// Write the reply for `id`, atomically (dotfile temp + rename): the reply's
/// existence is the done marker a waiting depositor polls for.
pub fn write_reply(state_root: &Path, id: &str, reply: &Value) -> io::Result<()> {
    let dir = gestures_dir(state_root).join(REPLIES_DIR);
    fs::create_dir_all(&dir)?;
    let tmp = dir.join(format!(".{id}{EXT}.tmp"));
    fs::write(&tmp, reply.to_string().as_bytes())?;
    fs::rename(&tmp, dir.join(format!("{id}{EXT}")))
}

/// Read back the reply for `id`, if answered: the depositor's poll. `None`
/// until the reply file exists and parses whole (the atomic rename makes a
/// torn read impossible; a parse failure is treated as not-yet).
pub fn read_reply(state_root: &Path, id: &str) -> Option<Value> {
    let bytes = fs::read(reply_path(state_root, id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests;
