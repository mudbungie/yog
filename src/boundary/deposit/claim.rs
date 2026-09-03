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
    _lock: fs::File,
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
        _lock: file,
    })
}

/// True when nobody holds `path`'s claim lock: the claimant is gone, however
/// it went. The probe's own lock is released on return; the only later writer
/// of a debris reply slot is another sweep writing the same sentence, so the
/// momentary hold is enough.
pub fn unheld(path: &Path) -> bool {
    fs::File::open(path).is_ok_and(|f| f.try_lock().is_ok())
}
