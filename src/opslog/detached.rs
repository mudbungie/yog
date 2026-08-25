//! The detached driver's captured stderr (DESIGN §8.1, §13.3, §4.2, §5.2).
//!
//! A detached `lernie prompt`'s status is never observed — the row is written at
//! launch and never rewritten — so its ops line carries the
//! [`DETACHED_EXIT`] sentinel and an empty `stderr` (§8.1), and says nothing but
//! that the handoff happened (bl-afa9: a spawn that never happened writes a
//! synthetic-failure line instead, so this sentinel means one thing). That made a driver
//! which **dies right after launch** — a version-skew refusal, a missing model
//! config — indistinguishable from a clean launch: exit `-2`, nothing rendered,
//! a prompt that visibly "does nothing". So the child's stderr is routed to a
//! per-spawn **sink file** under the yog state root, and the row's `stderr` is
//! folded in from that file **at read time** ([`fold`]) rather than copied into
//! `ops.jsonl`. The file is the authority; the row is a projection, so the fact
//! is never stored twice and a still-running driver's later output surfaces on
//! the next sweep without rewriting a durable line.
//!
//! A folded capture makes the row a rendered failure
//! ([`OpRow::failed`](super::rows::OpRow::failed) reads `DETACHED_EXIT`
//! that way), which is what stirs the §6/§11 activity surface and the §7.3
//! ichor-red banner — no new signal, the existing machinery fed a fact it was
//! previously denied.
//!
//! **Whether to fold at all is not this file's question** (bl-b95e). This file
//! is a *transport*: it names the sink and reads its tail, and says nothing
//! about what the tail means. For two rulings it was folded into every `-2`
//! row, which made the meaning "the driver said anything at all" — and
//! lernie's contract makes this sink an **operator-notice** channel as much as
//! a dying one (declines, superseded landings, accepted-crash-class launch
//! notes, a §6 budget stop, all printed on paths that return `Ok(())`). bl-1296
//! answered with a phrase table over those sentences; bl-b95e deleted the table
//! and moved the decision to where §13.3 already puts it for `driver.log` — the
//! **state** the launch produced ([`super::launch::stillborn`]), asked by the
//! caller ([`crate::app::derive`]'s ops refresh) before it folds. Content is
//! diagnosis. That also dissolves what no table could reach: the sink is
//! append-only for the driver's whole life and this fold re-reads its tail every
//! sweep, so one unrecognized line held its origin's newest row red for every
//! later pass, however many turns the driver went on to run.
//!
//! **The join key is computed, not stored.** The sink's name derives from facts
//! the ops line already carries — its `ts` and its workspace argument — so the
//! row needs no extra field to find its file, and [`sink`] is the single home of
//! the naming: the spawn side creates that path, [`fold`] reads it back.

use super::{DETACHED_EXIT, OpEntry};
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// The sink directory under the yog state root, beside `ops.jsonl` (§5.2).
const DIR: &str = "detached";

/// The sink leaf's extension.
const EXT: &str = "err";

/// How many trailing bytes of a sink the projection folds in. A driver may
/// chatter for hours; the tail is where a death lands, and the bound keeps the
/// sweep's read cost flat regardless of how long the loop has run.
const TAIL: u64 = 4096;

/// Sink-name stand-in for a workspace path with no file name (`/`, `..`).
const UNNAMED: &str = "workspace";

/// The per-spawn stderr sink for a detached `lernie prompt`:
/// `<state_root>/detached/<ts>-<workspace leaf>.err`. Both sides of the fold go
/// through here — the spawn hands this path to
/// [`spawn_detached`](crate::cli_outbound::Cli::spawn_detached), [`fold`] reads
/// it back — so the naming has one home. `ts` (unix seconds) plus the workspace
/// leaf separates every spawn the operator can actually make: one fire per
/// workspace per second.
pub fn sink(state_root: &Path, ts: &str, workspace: &Path) -> PathBuf {
    let leaf = workspace
        .file_name()
        .map_or_else(|| UNNAMED.to_owned(), |n| n.to_string_lossy().into_owned());
    state_root.join(DIR).join(format!("{ts}-{leaf}.{EXT}"))
}

/// `entry` with its detached child's captured stderr folded in — the read-time
/// projection the ops sweep applies to a line whose launch produced nothing
/// (§4.2, §7.2; the gate is [`super::launch::stillborn`], asked by the caller).
///
/// Only a [`DETACHED_EXIT`] line whose own `stderr` is empty is folded, which
/// since bl-afa9 is every `-2` line yog writes: a spawn that never launched is a
/// synthetic-failure line now, not a detached one. The empty-`stderr` guard
/// stays because `ops.jsonl` is append-only — a pre-bl-afa9 line that stored a
/// spawn error is a durable fact, and a sink must never clobber a stored one.
/// Everything else — piped verbs, synthetic failures — rides back untouched.
pub fn fold(state_root: &Path, entry: &OpEntry) -> OpEntry {
    if entry.exit != DETACHED_EXIT || !entry.stderr.is_empty() {
        return entry.clone();
    }
    // `lernie prompt … <workspace> <goal>` behind the resolved binary: the goal
    // is always LAST (`clip_goal` trims exactly it, so the rest survives the
    // log intact) and the workspace rides just before it. Reading from the tail
    // holds across flag growth (`--name`, bl-08f2) and across the append-only
    // log's older flagless lines alike.
    let Some(workspace) = entry.argv.iter().rev().nth(1) else {
        return entry.clone();
    };
    OpEntry {
        stderr: captured(&sink(state_root, &entry.ts, Path::new(workspace))),
        ..entry.clone()
    }
}

/// The tail of a capture file as text: at most [`TAIL`] bytes, starting at a
/// line boundary when the head was clipped (a half-line is noise, not a cause).
/// An absent or unreadable sink — the overwhelmingly common case, a clean
/// launch that never wrote — yields the empty string, leaving the row a clean
/// launch.
///
/// `pub(crate)` since bl-55d8: this is the crate's **one** bound on how much of
/// a captured stderr yog ever reads back, and the §7.3 no-response wound reads
/// a step's own `stderr.log` (lernie ARCH §2.3) through it. Nothing about the
/// bound is detached-spawn-specific — a driver that chatters for hours and an
/// adapter that retried a hundred times both die at the tail — and a second
/// spelling of "how far back do we read" is how the two would drift.
pub(crate) fn captured(path: &Path) -> String {
    let Ok((bytes, clipped)) = read_tail(path) else {
        return String::new();
    };
    let text = String::from_utf8_lossy(&bytes);
    match clipped.then(|| text.split_once('\n')).flatten() {
        Some((_partial, rest)) => rest.to_owned(),
        None => text.into_owned(),
    }
}

/// The last [`TAIL`] bytes of `path` plus whether the head was clipped. Seeks
/// rather than reading the whole file: a long-lived driver's sink is unbounded,
/// and this runs per detached row on every sweep.
fn read_tail(path: &Path) -> io::Result<(Vec<u8>, bool)> {
    let mut file = fs::File::open(path)?;
    let from = file.metadata()?.len().saturating_sub(TAIL);
    file.seek(SeekFrom::Start(from))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok((buf, from > 0))
}

#[cfg(test)]
mod tests;
