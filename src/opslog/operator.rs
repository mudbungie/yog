//! The operator's own two lines in the trail (DESIGN §4.2 as amended, §7.3,
//! §11): the **ack** that quiets every failure-derived alarm, and the **clear**
//! that starts a fresh trail.
//!
//! The complaint both answer is one complaint: a failure banner clears only
//! when a *newer successful op of the same origin* lands (§7.3), so an operator
//! who reads the error and decides not to retry stares at it forever; and the
//! §11 chip's ⚠ count derives from an append-only file, so it can never reach
//! zero either. Neither had a dismissal, and the log had no verb that ends a
//! trail.
//!
//! **Dismissing is an action, so it is an ops line.** It is not a stored flag,
//! a second state home, or a per-surface bit: `ops.jsonl` stays the single
//! source of truth, and the ack is a **global seen-watermark** derived from it
//! — every failure-derived alarm considers only the rows *after* the newest ack
//! ([`since_ack`]). One line shape, no per-origin variants: dismissing from
//! anywhere means "I have seen what is on screen now". A **new** failure
//! afterwards lands after the watermark and re-alarms, which is the whole point
//! of watermarking rather than deleting.
//!
//! **Ack quiets alarms; it never hides history.** The expanded trail renders
//! every row it always did — the acked failures and the ack line itself — and
//! the chip's `N ops` still counts the whole tail, because that number names
//! the rows the expansion shows and an ack removes none of them.
//!
//! Both lines are ordinary completed [`YOG_STEP`] rows
//! ([`OpEntry::step_done`]), exit `0`, [`Origin::World`] — yog's own doing, on
//! no operator surface, so neither can banner anywhere (§7.3) and neither reads
//! as a failure. They obey every rule the other line shapes obey: built through
//! [`build_line`], so the ≤[`CAP`](super::CAP)/PIPE_BUF atomicity bound holds
//! for them too.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::rows::OpRow;
use super::{FILENAME, OpEntry, Origin, YOG_STEP, build_line};

/// `argv[1]` of the **ack** line: the operator has seen every alarm on screen.
/// The whole vocabulary of the watermark — [`since_ack`] recognises a row by
/// this and nothing else.
pub(crate) const ACK_STEP: &str = "ack-failures";

/// `argv[1]` of the **clear** line: the first row of the trail it begins.
pub(crate) const CLEAR_STEP: &str = "clear-trail";

/// The dismiss control's label, one home for two seats (the §7.3 banner and the
/// §11 ops pane both offer it, and a control the operator meets twice must not
/// be spelled two ways).
pub(crate) const ACK_LABEL: &str = "Dismiss";

/// What the dismiss control says on hover (bl-68ac: every control explains
/// itself). It states the two things the operator cannot see from the button —
/// that the quiet is not permanent, and that nothing is thrown away.
pub(crate) const ACK_HOVER: &str = "Mark every failure and drift now on screen as seen. \
The banners and the activity chip go quiet; the trail keeps every row, and the \
next failure raises them again. Typed, it is `/ack`.";

/// The clear verb's label (§11 ops pane).
pub(crate) const CLEAR_LABEL: &str = "Clear trail";

/// What the clear verb says on hover — it names the destruction outright,
/// because this is the one gesture in yog that discards durable history.
pub(crate) const CLEAR_HOVER: &str = "Start a fresh trail: discard every row of \
ops.jsonl and log this clear as the new trail's first row. The discarded rows \
are not kept anywhere else. Typed, it is `/clear-trail`.";

/// Append the **ack** line (§4.2): the operator's seen-watermark over the trail.
/// `ts` is the caller's clock stamp, as for every other line — this module
/// reads no clock.
pub fn ack(state_root: &Path, ts: &str) -> io::Result<()> {
    super::append(state_root, &entry(ts, state_root, ACK_STEP))
}

/// **Clear the trail** (§4.2 as amended, §11): truncate `ops.jsonl` and log the
/// clear itself as the new trail's first row, so the discard is an action with
/// a record like every other action.
///
/// Written through an `O_APPEND` handle that is truncated with `set_len`, never
/// a positioned write: the clear line therefore lands at whatever the end of
/// the file is *at write time*, so a concurrent instance's append between the
/// truncate and the write is preserved rather than overwritten. The reader is
/// stateless (it re-reads the whole file, §4.2), so a shrinking file needs no
/// handling of its own.
pub fn clear(state_root: &Path, ts: &str) -> io::Result<()> {
    fs::create_dir_all(state_root)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(state_root.join(FILENAME))?;
    file.set_len(0)?;
    file.write_all(&build_line(&entry(ts, state_root, CLEAR_STEP)))?;
    Ok(())
}

/// The shared shape of both lines: a completed `["yog-step", <step>]` row in
/// the state root, attributed to [`Origin::World`].
fn entry(ts: &str, state_root: &Path, step: &str) -> OpEntry {
    OpEntry::step_done(
        ts.to_owned(),
        step,
        state_root.to_string_lossy().into_owned(),
        Origin::World,
    )
}

/// The rows an alarm may consider: everything **after** the newest ack line, or
/// all of them when the operator has acknowledged nothing. The one derivation
/// of the watermark — [`AppModel::last_failure`](crate::AppModel::last_failure)
/// and the §11 chip's counts ([`super::activity`]) both read it, so a banner
/// and a chip can never disagree about what has been seen.
///
/// Slicing off a *prefix* cannot change what the rows that remain mean: §6's
/// retirement looks only at rows *later* than the one it judges
/// ([`super::outcomes`]), so an outcome computed over the suffix is the outcome
/// it had over the whole tail. `pub(crate)` — a borrowed slice is an internal
/// accessor, never a boundary type.
pub(crate) fn since_ack(rows: &[OpRow]) -> &[OpRow] {
    let after = rows.iter().rposition(is_ack).map_or(0, |i| i + 1);
    rows.get(after..).unwrap_or_default()
}

/// Whether `row` is an ack line — its leading two argv tokens, the same
/// two-token verb reading §6's retirement key uses.
fn is_ack(row: &OpRow) -> bool {
    row.argv.split(' ').take(2).eq([YOG_STEP, ACK_STEP])
}

#[cfg(test)]
mod tests;
