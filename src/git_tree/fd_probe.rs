//! Writer-fd probe for the branch-state classifier (ARCH §3.5, §4.4).
//!
//! The §3.5 completion signal is the writer closing the `response.json`
//! fd (`IN_CLOSE_WRITE`). A terminal event on disk is only authoritative
//! once that fd is closed: the harness holds ONE fd across every retry
//! attempt and the backoff sleeps between them (§4.4 "Fd held open for
//! the whole model call"), so a mid-retry `end` segment with a writer
//! still present is `in_flight`, not `stopped`. This module answers the
//! one question the classifier needs — "does a process still hold this
//! path open for *write*?" — by scanning `/proc/<pid>/fd/*`, filtered to
//! writers via `/proc/<pid>/fdinfo/<fd>` (a reader such as the UI's own
//! tail must never be mistaken for the harness).
//!
//! The scanner mirrors the harness's own `stop::discover` writer scan;
//! it is duplicated here (not shared via a crate dep) so the frontend
//! stays decoupled from the harness binary (ARCH §3.5 pluggability — a
//! frontend shares nothing but the filesystem and the CLI). Linux only;
//! `/proc` is the verified platform (§2.9).

use super::probe::{Probe, WriterProbe};
use std::path::{Path, PathBuf};

/// Production probe backed by `/proc`.
pub(super) struct ProcFsProbe {
    proc_root: PathBuf,
}

impl Default for ProcFsProbe {
    fn default() -> Self {
        Self {
            proc_root: PathBuf::from("/proc"),
        }
    }
}

impl ProcFsProbe {
    #[cfg(test)]
    fn with_root(proc_root: PathBuf) -> Self {
        Self { proc_root }
    }

    fn scan(&self, path: &Path) -> bool {
        // Canonicalize so the symlink-target compare is exact. A file
        // removed between close and scan canonicalizes to NotFound →
        // no writer.
        let Ok(target) = std::fs::canonicalize(path) else {
            return false;
        };
        let Ok(entries) = std::fs::read_dir(&self.proc_root) else {
            return false;
        };
        for entry in entries.flatten() {
            // Only numeric pid dirs; skip `/proc/acpi`, `/proc/self`, etc.
            if entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<u32>().ok())
                .is_none()
            {
                continue;
            }
            if pid_holds_writable(&entry.path(), &target) {
                return true;
            }
        }
        false
    }
}

impl WriterProbe for ProcFsProbe {
    // `/proc` is always present on Linux (DESIGN §10), so this backend is
    // never `Unknown`: the scan settles definitely to `Held` or `Free`. A
    // missing `proc_root` (only reachable in tests) reads as `Free`, the
    // historical "no writer" behavior preserved by the tri-state move.
    fn writer_state(&self, path: &Path) -> Probe {
        if self.scan(path) {
            Probe::Held
        } else {
            Probe::Free
        }
    }
}

/// Any fd under `<proc_pid>/fd/` resolving to `target` and opened for
/// write. Pids we cannot introspect (other uid, raced teardown) simply
/// do not match.
fn pid_holds_writable(proc_pid: &Path, target: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(proc_pid.join("fd")) else {
        return false;
    };
    for entry in entries.flatten() {
        // Canonicalize the fd's symlink target before comparing: `target`
        // is already canonicalized (`scan`), and on platforms where the
        // tempdir/prefix is itself a symlink (macOS `/tmp` → `/private/tmp`)
        // the raw `read_link` value would never equal it. Comparing two
        // canonical paths is exact everywhere; a vanished target (raced
        // teardown) fails to canonicalize and the entry is skipped.
        if !std::fs::read_link(entry.path())
            .and_then(std::fs::canonicalize)
            .is_ok_and(|link| link == target)
        {
            continue;
        }
        let Some(fd) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        if fdinfo_is_writable(proc_pid, fd) {
            return true;
        }
    }
    false
}

/// Parse `flags:` from `<proc_pid>/fdinfo/<fd>`. The low octal digit of
/// the access mode is `0` for `O_RDONLY`; anything else is a writer.
fn fdinfo_is_writable(proc_pid: &Path, fd: u32) -> bool {
    let Ok(contents) = std::fs::read_to_string(proc_pid.join("fdinfo").join(fd.to_string())) else {
        return false;
    };
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("flags:") {
            return rest.trim().chars().last().is_some_and(|c| c != '0');
        }
    }
    false
}

#[cfg(test)]
mod tests;
