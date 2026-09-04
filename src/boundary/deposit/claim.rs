//! The claim lifecycle (§8.5, bl-d1f1): a consumer takes a deposit by rename
//! and **holds it under an OS file lock** for as long as it lives, so that a
//! crash between claim and reply is tellable from work in flight — the kernel
//! releases the lock the instant the claimant dies, and nothing else can. The
//! sweep that reads that fact is [`crate::boundary::consume::sweep`]; the
//! contract it enforces — a lost reply is *in doubt*, and the recovery is a
//! read, never a resend — is stated at the module above and DESIGN §8.5.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::{CLAIMED_DIR, EXT, gestures_dir, list};

/// The taken deposits — answered audit and in-flight work alike: the listing
/// over `claimed/`. [`crate::boundary::consume::sweep`] walks this to find
/// the crash debris.
pub fn claimed(state_root: &Path) -> Vec<(String, PathBuf)> {
    list(&gestures_dir(state_root).join(CLAIMED_DIR))
}

/// A held claim: the claimed file's path plus the OS lock that marks its
/// claimant alive. The lock is taken **before** the rename — locks follow the
/// inode, so a file in `claimed/` is locked from before it ever appears there
/// and a sweeper can never mistake live work for debris.
#[derive(Debug)]
pub struct Claim {
    path: PathBuf,
    lock: fs::File,
}

impl Drop for Claim {
    /// Release the claim explicitly (bl-98ce, module doc): a close only
    /// releases when the LAST descriptor onto this description goes, and a
    /// concurrently forked child owns one of those until its `exec`.
    fn drop(&mut self) {
        let _ = self.lock.unlock();
    }
}

impl Claim {
    /// Where the claimed gesture's bytes sit, this consumer's to answer.
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

/// Claim one pending deposit: lock it, then rename it into `claimed/`. Only
/// the lock's winner renames, so the rename stays the mutual exclusion it
/// always was and the loser of either race gets the error and moves on.
pub fn claim(state_root: &Path, id: &str) -> io::Result<Claim> {
    let dir = gestures_dir(state_root);
    let claimed_dir = dir.join(CLAIMED_DIR);
    fs::create_dir_all(&claimed_dir)?;
    let from = dir.join(format!("{id}{EXT}"));
    let file = fs::File::open(&from)?;
    if file.try_lock().is_err() {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            format!("gesture {id:?} is already claimed"),
        ));
    }
    let to = claimed_dir.join(format!("{id}{EXT}"));
    fs::rename(&from, &to)?;
    Ok(Claim {
        path: to,
        lock: file,
    })
}

/// True when nobody holds `path`'s claim lock: the claimant is gone, however
/// it went. The probe's own lock is dropped by [`fs::File::unlock`] before it
/// returns, never by the close (module doc) — a probe whose descriptor a fork
/// copied would otherwise leave the file reading as claimed by nobody.
pub fn unheld(path: &Path) -> bool {
    let Ok(file) = fs::File::open(path) else {
        return false;
    };
    if file.try_lock().is_err() {
        return false;
    }
    let _ = file.unlock();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::deposit::deposit;
    use serde_json::json;
    use tempfile::tempdir;

    /// bl-98ce: the release is the `unlock`, so a **copy of the claimant's
    /// descriptor** — which is exactly what a `fork` on any other thread hands
    /// a child — cannot hold the claim open past the drop. `try_clone` is that
    /// copy, in-process and deterministic; released by close instead, this
    /// beat reads the dropped claim as work still in flight.
    #[test]
    fn a_dropped_claim_releases_while_a_copy_of_its_descriptor_still_lives() {
        let root = tempdir().unwrap();
        deposit(root.path(), "g-1", &json!({"op": "ack"})).unwrap();
        let held = claim(root.path(), "g-1").unwrap();
        let path = held.path();
        let forked = held.lock.try_clone().unwrap();
        drop(held);
        assert!(unheld(&path), "the drop released the lock, not the close");
        drop(forked);
    }
}
