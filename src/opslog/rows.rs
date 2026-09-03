//! The ops-trail row view-model (DESIGN §4.2, §7.3, §11).
//!
//! [`OpRow`] carries the **full** `ops.jsonl` entry (argv, cwd, exit, stdout,
//! stderr) so the ops pane can expand a row to the whole record — "a trail that
//! hides *why* is not a trail" (§7.3). What that `exit` field *means* — is the
//! row a failure, a drift, and how does its exit read in words — is the one
//! classification in [`super::exit`], which carries `OpRow`'s other half; where
//! the row *stands* in the trail is [`super::standing`]'s, and both cross the
//! §8.5 boundary on the row itself.
//!
//! **`SurfaceFailure` stood here and is gone (bl-4d81).** It was the compact
//! argv-plus-stderr-tail a §7.3 banner painted, held by whichever surface the
//! op's `origin` named — and it had no production caller once the window left
//! (bl-7942). The row-level [`Standing`](super::Standing) answers it: a banner
//! is the rows standing `live`, grouped by `origin`, and every field it would
//! have projected is already on the row it projected from.

use super::{OpEntry, Origin};

/// How many trailing stderr lines the compact surface-failure view keeps: the
/// error's tail is where the cause lands; the ops pane expands to the full text.
const SURFACE_STDERR_LINES: usize = 3;

/// The one elision policy for [`OpRow::summary`] (bl-0bf9): a collapsed row is
/// a scan surface, one row per op — a prompt op's `argv` carries an arbitrary-
/// length, multi-line goal (a whole ball body is the ordinary case) that used
/// to flow into the list unwrapped, breaking the scan.
/// Past this many `char`s the summary is cut; this is the ONLY place `argv`
/// elides — the expansion (`cwd`/`exit`/`stdout`/`stderr`, §4.2) always carries
/// `argv` byte-exact.
///
/// **Where** it cuts is [`crate::elide::middle`]'s rule, not this file's
/// (bl-3aa1). The cut used to keep the head, which for an `argv` is the end
/// that does not distinguish it: every row opened
/// `litany <verb> … /home/<user>/.cache/…/data/yog/` — over half the row,
/// identical on every line — and the workspace leaf and agent id that told two
/// operations apart were exactly what fell off the end. A column of different
/// ops scanned as one repeated string.
const SUMMARY_ARGV_MAX: usize = 100;

/// One ops-pane row — the whole `ops.jsonl` entry, argv pre-joined for display.
/// Collapsed the pane shows `ts`/[`OpRow::summary`]/`exit`; expanded it shows
/// `cwd`, the byte-exact `argv`, `stdout`, and `stderr` (§11). No egui here —
/// the shell paints these.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpRow {
    pub ts: String,
    pub argv: String,
    pub cwd: String,
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
    /// The §7.3 attribution the durable line carried ([`Origin`]) — what lets a
    /// banner surface ask for *its own* last failure rather than the world's.
    pub origin: Origin,
}

impl OpRow {
    /// Whether either captured stream carried bytes — the pane's "expandable"
    /// hint (a row with output is worth opening).
    pub fn has_output(&self) -> bool {
        !self.stdout.is_empty() || !self.stderr.is_empty()
    }

    /// The collapsed row's one-line rendering of `argv` (bl-0bf9, see
    /// [`SUMMARY_ARGV_MAX`]): embedded newlines fold to spaces — a prompt op's
    /// goal is multi-line and would otherwise wrap the list row across several
    /// lines — then anything past the cap is cut through the middle, keeping
    /// the verb at the head and the workspace/agent at the tail
    /// ([`crate::elide::middle`], bl-3aa1). `argv` itself is untouched; the
    /// expansion always shows the verbatim field.
    pub fn summary(&self) -> String {
        let flat: String = self
            .argv
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .collect();
        crate::elide::middle(&flat, SUMMARY_ARGV_MAX)
    }

    /// `ts` for a human (bl-61db): the §11 activity row led with the raw
    /// `ts` field verbatim — unix seconds as a decimal string, the crate's
    /// timestamp convention ([`crate::ui_state::Clock::stamp`]) — which reads
    /// as `1785630266`, not a time. Rendered through
    /// [`crate::ui_state::iso8601_extended`], the same ISO 8601 extended
    /// spelling the chat header derives from a conversation id (bl-16da), so
    /// the crate has one human-timestamp grammar rather than two. `ts` that
    /// does not parse (only ever a test fixture — every real line is stamped
    /// by [`crate::ui_state::SystemClock`]) renders as itself, same as the
    /// header's own foreign-id fallback.
    pub fn when(&self) -> String {
        self.ts
            .parse::<i64>()
            .map_or_else(|_| self.ts.clone(), crate::ui_state::iso8601_extended)
    }
}

impl From<&OpEntry> for OpRow {
    fn from(entry: &OpEntry) -> Self {
        Self {
            ts: entry.ts.clone(),
            argv: entry.argv.join(" "),
            cwd: entry.cwd.clone(),
            exit: entry.exit,
            stdout: entry.stdout.clone(),
            stderr: entry.stderr.clone(),
            origin: entry.origin,
        }
    }
}

/// The last [`SURFACE_STDERR_LINES`] non-empty-trailing lines of `stderr`, the
/// cause the surface shows compactly. Empty stderr yields an empty tail.
///
/// `pub(crate)` since bl-55d8: this is the crate's **one** answer to "how much
/// of a stderr does a *surface* say", and the §7.3 no-response wound's banner
/// is a surface like the rest. A banner that quoted a different number of lines
/// than the ops row beside it would be two policies for one question.
pub(crate) fn stderr_tail(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.trim_end_matches('\n').lines().collect();
    let start = lines.len().saturating_sub(SURFACE_STDERR_LINES);
    lines.get(start..).unwrap_or_default().join("\n")
}

#[cfg(test)]
mod tests;
