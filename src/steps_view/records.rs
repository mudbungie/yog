//! **The two capture-log file names** — one home for each, because more than
//! one derivation reads them and two spellings of one file name drift.
//!
//! This file was the drill-in picker's row set until bl-7942: the five JSON
//! records litany contracts to write, the words each seat carries, and the
//! rule that a log earns a seat only when it has bytes. A picker is a face,
//! and the face is the seat crate's now — what a server keeps is the two
//! *names*, which the §7.3 wound and orphan derivations open directly.
//!
//! `pub(crate)` since bl-9b88: the §3.5 classifier reads a step's `stderr.log`
//! too — the out-of-band half of "why the latest model call failed"
//! ([`crate::git_tree`]'s `failure`) — and this module's whole claim is that
//! one file name has one spelling however many derivations open it.

/// The adapter subprocess's captured stderr, per step (litany ARCH §2.3). The
/// §7.3 wound reads its tail to say why a step produced nothing.
pub(crate) const STDERR_FILE: &str = "stderr.log";

/// Where litany binds a launched driver's stderr (ARCH §2.11; litany bl-55f9):
/// beside the step dirs, one per agent, append-only across launches. The
/// orphaned-mail predicate reads it for the same reason.
pub(crate) const DRIVER_LOG_FILE: &str = "driver.log";
