//! **Which file yog itself is** — asked once per process, never re-asked
//! (bl-f558). One fact with one home, which is the whole of this module.
//!
//! Every self-multiplexed resolution ([`resolve`](super::resolve)), every
//! world tool shim (`world::tools`) and the `$EDITOR` re-entry §9.3 spawns
//! (`config_edit::branch::edit`) name yog's own executable. Until this existed
//! each of them called `std::env::current_exe()` at the moment it needed the
//! answer — several representations of one fact, which is the drift the house
//! rule forbids, and the drift is not hypothetical:
//!
//! **A live engine outlives its own inode.** Installing, updating or rebuilding
//! yog replaces the file at yog's pathname — the install shape is `cp new
//! yog.next && mv -f yog.next yog`, atomic by `rename(2)` — while the running
//! process keeps executing the unlinked image. From that instant Linux's
//! `/proc/self/exe`, which is the whole of `current_exe()` there, reads back
//! `<path> (deleted)`: a procfs *annotation*, not a path, naming a file that
//! does not exist and cannot be exec'd. A resolution taken after the replace
//! therefore yields an impossible program, and anything durable written from it
//! — the §8.6 `tool-control` shim is the sharp case, since the start flow
//! re-resolves it on every Start — fails closed on every later use, long after
//! the replace is forgotten.
//!
//! **Two answers, and both are here rather than at the call sites.**
//!
//! 1. **The reading is taken once, at the first ask** ([`self_exe`]), which
//!    every face performs during boot — `main.rs` converges the world tool
//!    roster before eframe, and a `yog <ns> …` re-entry resolves before it
//!    dispatches. So the process holds the pathname it was *born* from for its
//!    whole life, and an install that replaces that pathname leaves every
//!    derived artifact naming the file the operator just installed. That is the
//!    honest answer as well as the stable one: the shim must name yog's install
//!    path, and the install path is exactly what did not change.
//! 2. **A reading that names no file is not a reading** ([`usable`]). This is a
//!    question about the filesystem, never about the spelling — no `(deleted)`
//!    suffix is stripped and none is matched, because a binary genuinely
//!    installed at a path ending in ` (deleted)` is a real target and a binary
//!    merely unlinked is not, and only `stat` can tell those apart. It covers
//!    the platforms whose `current_exe()` reports the original path with no
//!    annotation at all (macOS `_NSGetExecutablePath`) by the same one test.
//!
//! What is left after both is a pathname yog was born from that has since been
//! deleted outright rather than replaced — a *stale* target, honestly named,
//! which the next install heals. Nothing here can do better: yog cannot invent
//! a path for a binary that is not anywhere.

use std::path::PathBuf;
use std::sync::OnceLock;

/// The process's one reading, memoized at the first ask (see the module doc for
/// why it is taken once rather than per call).
static SELF_EXE: OnceLock<Option<PathBuf>> = OnceLock::new();

/// The judgement over one reading, pure and separable: a path yog's own
/// executable can be exec'd from, or nothing. `is_file` is the whole test —
/// it rejects the `<path> (deleted)` procfs annotation for the same reason it
/// rejects a binary that was unlinked without replacement, which is that
/// neither of them is there.
pub(crate) fn usable(reading: Option<PathBuf>) -> Option<PathBuf> {
    reading.filter(|path| path.is_file())
}

/// yog's own executable, as a fact of THIS process: the first [`usable`]
/// reading, held for the process's life. `None` when yog was born from a file
/// that is already gone — every caller then falls back to what it would do
/// without a self-exe at all, and no caller may persist a substitute.
pub(crate) fn self_exe() -> Option<PathBuf> {
    SELF_EXE
        .get_or_init(|| usable(std::env::current_exe().ok()))
        .clone()
}
