//! `ops.jsonl`: the durable action-outcome log (DESIGN §4.2, §15 Y15).
//!
//! yog is a pure renderer of disk except for two owned files; this is one of
//! them. Every **attempted** yog-initiated CLI action appends one JSON line to
//! `<yog_state_root>/ops.jsonl` — `{ts, argv, cwd, exit, stdout, stderr}` — so
//! both instances tail one shared history instead of holding gate/close output
//! RAM-only (§4.2 as amended: "closes the one durability leak"). A *completed*
//! run logs its real outcome; a spawn or non-spawn **step failure** logs a
//! synthetic line ([`OpEntry::synthetic_failure`] / [`OpEntry::step_failure`])
//! so no error class is un-logged — the §7.3 failed-action row depends on it.
//!
//! **Atomicity by size cap.** The line *including its newline* is hard-capped
//! at [`CAP`] = 4096 bytes (PIPE_BUF): a write that size or under is a single
//! atomic `O_APPEND` on Linux, so two instances never interleave.
//! [`build_line`] is the pure capper — it truncates `stdout`, then `stderr`,
//! keeping heads and stamping `"truncated":true`; the fixed fields
//! (`ts`/`argv`/`cwd`/`exit`) never truncate, so a pathological argv is the one
//! case a line may exceed the cap (structurally unavoidable; §4.2 is silent
//! there — we stay strict-otherwise).
//!
//! **Append-only, with exactly one operator-initiated exception.** No line is
//! ever rewritten — a row is a projection of what was true when it was written,
//! and the read-time folds below are how it gains detail. The exception is
//! [`clear`] ([`operator`], bl-c417): the operator asking for a fresh trail
//! truncates the file and logs *that* as the new trail's first row. Durability
//! (§4.2) is the promise that yog never loses an outcome **silently**; a
//! discard the operator asked for, which leaves its own record behind, loses
//! nothing silently.
//!
//! **No clock here.** `ts` is a data field, stamped upstream from the caller's
//! clock; nothing in this module reads wall-clock time, keeping [`build_line`]
//! and the tail parser pure and deterministic.
//!
//! **One field is not stored here.** A detached spawn's `stderr` is captured to
//! a per-spawn sink file and folded into the row at read time
//! ([`detached`]) — the log records the launch, the sink records what the
//! launched process said.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// The hard cap on a serialized line *including* its trailing newline, in
/// bytes. 4096 = PIPE_BUF: at or under it an `O_APPEND` write is atomic.
pub const CAP: usize = 4096;

/// The log's leaf name under the yog state root (§4.2).
const FILENAME: &str = "ops.jsonl";

/// How many `ops.jsonl` lines the trail carries (§4.2, §11 accessory) — **one
/// bound, named once**. The derivation tails this many into the snapshot, and
/// the §11 activity accessory asks [`Query::Ops`](crate::boundary::Query::Ops)
/// for this many, so the pane and the fold behind it cannot disagree about how
/// much trail there is. It lives here rather than beside either reader because
/// it is a fact about the log.
pub const OPS_TAIL: usize = 256;

/// `exit` sentinel for a piped verb whose status was unobservable
/// (`ExitInfo::Unknown`, [`crate::cli_outbound::ExitInfo::shell_code`]): the
/// process **ran** — not a rendered failure ([`rows::OpRow::failed`]).
pub const PIPED_UNOBSERVED: i32 = -1;

/// `exit` sentinel for the detached `litany prompt` (§8.1, §13.3): the row is
/// written the moment the child launches, while its status arrives arbitrarily
/// later (the reaper thread takes it and discards it — bl-3016 — since
/// `ops.jsonl` is append-only and never rewritten), so `-2` records **the
/// handoff itself**: "launched detached; exit deliberately unobserved". It says
/// that and only that (bl-afa9) — a spawn that never launched is a
/// [`SYNTHETIC_EXIT`] line like every other never-launched spawn, so this
/// sentinel can no longer stand for two opposite facts. The one thing that can
/// still make a `-2` row [`rows::OpRow::failed`] is text the *launched* child
/// wrote on stderr, folded in from its sink at read time ([`detached`]).
pub const DETACHED_EXIT: i32 = -2;

/// `exit` sentinel for a **synthetic failure line** (§4.2 as amended): an
/// attempted action that produced no process status — a spawn that never
/// launched (piped or detached), or a non-spawn yog-step failure
/// ([`OpEntry::synthetic_failure`] / [`OpEntry::step_failure`]). The failure
/// text always rides in `stderr`; which of the two it is reads off `argv[0]`
/// ([`exit::ExitKind`]), the [`YOG_STEP`] pseudo-binary naming the stepwise one.
pub const SYNTHETIC_EXIT: i32 = -3;

/// `exit` sentinel for a **drift line** (§7.2 instrumentation): not an attempted
/// action at all, but an observation yog made about *its own* event stream — a
/// sweep or the watch backend finding a change nobody announced. It rides
/// `ops.jsonl` because that is where yog's durable, two-instance-shared trail
/// already lives, and is deliberately **not** a [`rows::OpRow::failed`] row: a
/// drift is an alarm about the watcher, not a failed operator action, so it must
/// not hijack the §7.3 failure banner. It carries its own count on the §11
/// activity chip instead ([`live::Activity`]) — a query over the tail, not a
/// stored counter.
pub const DRIFT_EXIT: i32 = -4;

/// `argv[0]` of a drift line (§7.2). The drift *kind* rides as `argv[1]` — e.g.
/// `["yog-drift","unannounced"]` — and the roots it names ride in `stderr`, one
/// per line, so the §11 accessory's existing expand-a-row affordance shows the
/// attribution with no new surface.
pub const YOG_DRIFT: &str = "yog-drift";

/// `argv[0]` of a non-spawn step-failure line (§4.2): the logical step name
/// rides as `argv[1]`, e.g. `["yog-step","mint"]` — an error class with no real
/// binary still gets a rendered ops row (the §7.3 failed-action row depends on
/// every error class having one).
pub const YOG_STEP: &str = "yog-step";

/// `argv[0]` of a **capability-answer** line (§8.6): the operator's answer to a
/// held tool invocation, or a per-conversation floor raised or lowered. These
/// rows are at once the audit and the memory the capability control folds on
/// read (`["yog-control","answer",<tool-use-id>,<verdict>]`,
/// `["yog-control","floor",<conversation-id>,"raise"|"lower"]`) — the alignment
/// monitor's own pattern, so answering needs no fourth durable artifact and I2
/// holds at three. Written by the boundary's answer actions (bl-765d, bl-94b4);
/// read by [`crate::control::judge`], which is the whole reason the grammar has
/// one home rather than two.
pub const YOG_CONTROL: &str = "yog-control";

/// **The record itself** — [`OpEntry`] and its synthetic constructors; its own
/// file per §12's budget.
pub mod entry;
pub use entry::OpEntry;

/// The pure ≤[`CAP`] line serializer and the caller-side argv clip (§4.2). Split
/// out of this file per §12's line-budget discipline.
pub mod line;
use line::parse_line;
pub use line::{build_line, clip_goal};

/// Append `entry`'s capped line to `<state_root>/ops.jsonl` via `O_APPEND`,
/// creating the state dir if absent. Atomic against a concurrent instance by
/// the [`CAP`] size bound.
pub fn append(state_root: &Path, entry: &OpEntry) -> io::Result<()> {
    fs::create_dir_all(state_root)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(state_root.join(FILENAME))?;
    file.write_all(&build_line(entry))?;
    Ok(())
}

/// The last `max` parseable entries, oldest-first (newest-last). A missing file
/// or unreadable bytes yield an empty view; each line parses forgivingly — a
/// corrupt or mid-write-torn line is skipped, never an error.
pub fn tail(state_root: &Path, max: usize) -> Vec<OpEntry> {
    let Ok(bytes) = fs::read(state_root.join(FILENAME)) else {
        return Vec::new();
    };
    let mut entries: Vec<OpEntry> = bytes
        .split(|&b| b == b'\n')
        .filter_map(|line| std::str::from_utf8(line).ok())
        .filter_map(parse_line)
        .collect();
    let overflow = entries.len().saturating_sub(max);
    entries.drain(..overflow);
    entries
}

/// View-models over the log (§4.2, §7.3, §11), split out per §12's line budget:
/// `rows` = the expandable [`OpRow`] and the [`SurfaceFailure`] a surface holds;
/// `live` = §6's retirement projection over a tail of rows ([`OpOutcome`]: which
/// failures are still live — the log keeps every failure, prominence is derived)
/// + [`Activity`].
pub mod live;
pub mod rows;
pub use live::{Activity, OpOutcome, activity, outcomes};
pub use rows::{OpRow, SurfaceFailure};

/// The one reading of the `exit` field (§4.2's sentinels) and the one home of
/// its wording: [`exit::ExitKind`] plus the `OpRow` half that asks it
/// everything — `failed`, `drift`, `exit_label`.
pub mod exit;

/// The §7.3 attribution — which surface an attempted action came from, and the
/// one thing that lets a banner tell its own failures from someone else's.
pub mod origin;
pub use origin::Origin;

/// The detached driver's captured stderr (§8.1, §13.3): the per-spawn sink file
/// and the read-time fold that projects its tail into the row.
pub mod detached;

/// **What a detached launch produced** (§8.1, §13.3, bl-b95e) — the state a
/// `-2` row's failure is derived from, which decides whether the sink above is
/// read at all. It replaced the marker table (`opslog::notice`, bl-1296) that
/// tried to tell a dying line from a benign one by its words.
pub mod launch;

/// The operator's own two lines (§4.2 as amended, bl-c417): the **ack** — a
/// global seen-watermark that quiets every failure-derived alarm without
/// hiding a row — and the **clear**, the one gesture that ends a trail, which
/// logs itself as the next trail's first row.
pub mod operator;
pub(crate) use operator::since_ack;
pub use operator::{ack, clear};

#[cfg(test)]
mod tests;
