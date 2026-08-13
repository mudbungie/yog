//! What a running child produces, and the pump that produces it: the spawn
//! error, the output [`Chunk`] every stream shape emits, the [`ExitInfo`] that
//! terminates one, and the reader-thread byte pump behind them.
//!
//! This is the vocabulary the whole module speaks — [`super::stream`],
//! [`super::streamed`], [`super::detach`] and [`super::exec`] all hand these
//! values back, and `ops.jsonl` (§8.2) collapses an [`ExitInfo`] through the
//! one [`ExitInfo::shell_code`] mapping. [`super`] keeps the [`Cli`](super::Cli)
//! handle itself: which binary, under what prefix, with what standing env.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

/// One reader-thread read, sized to a page.
const READ_BUF: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("failed to spawn {binary}: {source}")]
    Spawn {
        binary: PathBuf,
        source: std::io::Error,
    },
    /// The caller-supplied working directory is not there (bl-6191). Its own
    /// variant because it is not a fact about the program at all — see
    /// [`CliError::spawn`].
    #[error("work directory does not exist: {}", dir.display())]
    WorkDirMissing { dir: PathBuf },
    /// The caller-supplied working directory exists but is not a directory — a
    /// file, a socket. Separate from [`WorkDirMissing`](Self::WorkDirMissing)
    /// because "does not exist" would be a *second* lie about a path that is
    /// plainly there, and the fix the operator needs is a different one.
    #[error("work directory is not a directory: {}", dir.display())]
    WorkDirNotADir { dir: PathBuf },
    #[error("child {0} stream was not captured")]
    Stdio(&'static str),
}

impl CliError {
    /// The one mapping from a failed fork to the fault the operator can act on
    /// (bl-6191). `std::process` reports an unusable `current_dir` as ENOENT
    /// **against the program path** — the child fails between fork and exec,
    /// long after the argv is fixed — so a spawn into a directory that does not
    /// exist reads "failed to spawn `<yog binary>`: No such file or directory".
    /// The operator typed a bad directory and is told their binary is missing.
    ///
    /// So every spawn shape ([`run_streaming`](super::Cli::run_streaming),
    /// [`spawn_detached`](super::Cli::spawn_detached),
    /// [`exec_in_world`](super::Cli::exec_in_world)) routes its failure through
    /// here, and here asks the cwd [`work_dir_fault`]'s question before it
    /// repeats what the OS said. Not gated on the error's kind: if the requested
    /// cwd is not a directory then the fork could not have succeeded for any
    /// other reason, whatever errno the caller was handed.
    pub(super) fn spawn(binary: &Path, cwd: Option<&Path>, source: std::io::Error) -> Self {
        cwd.and_then(work_dir_fault).unwrap_or(Self::Spawn {
            binary: binary.to_path_buf(),
            source,
        })
    }
}

/// The spawn boundary's one question about a caller-supplied working directory:
/// is it a directory that exists? `Some(fault)` is the refusal, worded as the
/// operator will read it — absent, or present but not a directory.
///
/// Public because the §11 birth-config block's work-directory field pre-flights
/// **this** question before Enter fires anything
/// ([`crate::actions::work_dir_refusal`] reads it for the form): one reading of
/// "lawful cwd" and one sentence for it, so the field's red flag and a forced
/// spawn failure cannot disagree.
pub fn work_dir_fault(dir: &Path) -> Option<CliError> {
    if dir.is_dir() {
        return None;
    }
    let dir = dir.to_path_buf();
    Some(if dir.exists() {
        CliError::WorkDirNotADir { dir }
    } else {
        CliError::WorkDirMissing { dir }
    })
}

/// One piece of output from a running `lernie` subprocess. The final
/// chunk in any stream is always `Exited`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chunk {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Exited(ExitInfo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitInfo {
    Code(i32),
    Signal(i32),
    Unknown,
}

impl ExitInfo {
    /// Collapse to the shell-convention process exit integer: a plain code
    /// passes through, a terminating signal is `128 + signum`, an unobservable
    /// status is `-1`. The single home for the mapping — `yog exec`'s faithful
    /// exit propagation (§8.4) and the `ops.jsonl` `exit` field
    /// ([`crate::actions`], §8.2) both collapse an [`ExitInfo`] through here.
    pub fn shell_code(self) -> i32 {
        match self {
            ExitInfo::Code(c) => c,
            ExitInfo::Signal(s) => 128 + s,
            ExitInfo::Unknown => -1,
        }
    }
}

pub(super) fn pump<R: Read>(mut reader: R, tx: Sender<Chunk>, wrap: fn(Vec<u8>) -> Chunk) {
    let mut buf = [0u8; READ_BUF];
    while pump_step(&mut reader, &tx, &mut buf, wrap) {}
}

pub(super) fn pump_step<R: Read>(
    reader: &mut R,
    tx: &Sender<Chunk>,
    buf: &mut [u8],
    wrap: fn(Vec<u8>) -> Chunk,
) -> bool {
    let Ok(n @ 1..) = reader.read(buf) else {
        return false; // a 0-length read (EOF) or error ends the pump
    };
    let bytes = buf.get(..n).unwrap_or_default().to_vec(); // n <= buf.len() ⇒ Some
    tx.send(wrap(bytes)).is_ok()
}
