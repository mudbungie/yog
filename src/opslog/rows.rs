//! Ops-pane and surface-failure view-models (DESIGN §4.2, §7.3, §11).
//!
//! [`OpRow`] carries the **full** `ops.jsonl` entry (argv, cwd, exit, stdout,
//! stderr) so the ops pane can expand a row to the whole record — "a trail that
//! hides *why* is not a trail" (§7.3). What that `exit` field *means* — is the
//! row a failure, a drift, and how does its exit read in words — is the one
//! classification in [`super::exit`], which carries `OpRow`'s other half;
//! [`SurfaceFailure`]
//! is what the originating surface (start pane, input bar) holds as its §5.3
//! RAM item and paints in ichor red — argv + a stderr tail. Both are pure
//! projections of the durable ops line, so the surface's RAM item and the ops
//! pane never diverge (§5.3: "renders the durable fact from the pane").

use super::{OpEntry, Origin};

/// How many trailing stderr lines the compact surface-failure view keeps: the
/// error's tail is where the cause lands; the ops pane expands to the full text.
const SURFACE_STDERR_LINES: usize = 3;

/// The one elision policy for [`OpRow::summary`] (bl-0bf9): a collapsed row is
/// a scan surface, one row per op — a prompt op's `argv` carries an arbitrary-
/// length, multi-line goal (a whole ball body is the ordinary case) that used
/// to flow into the list unwrapped, breaking the scan.
/// Past this many `char`s the summary is cut with a trailing `…`; this is the
/// ONLY place `argv` elides — the expansion (`cwd`/`exit`/`stdout`/`stderr`,
/// §4.2) always carries `argv` byte-exact.
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
    /// lines — then anything past the cap elides with a trailing `…`. `argv`
    /// itself is untouched; the expansion always shows the verbatim field.
    pub fn summary(&self) -> String {
        let flat: String = self
            .argv
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .collect();
        if flat.chars().count() <= SUMMARY_ARGV_MAX {
            flat
        } else {
            let head: String = flat.chars().take(SUMMARY_ARGV_MAX - 1).collect();
            format!("{head}…")
        }
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

/// The failure a surface paints in ichor red (§7.3): the attempted `argv` and
/// the tail of its `stderr`. Built from the same ops line the pane renders, so
/// the two never diverge.
///
/// It carries no origin of its own (bl-48f8). The attribution lives on the
/// durable row ([`OpRow::origin`]) and is the *question*
/// [`AppModel::last_failure`](crate::AppModel::last_failure) is asked — a
/// surface that named an origin to get this value cannot learn anything by
/// reading it back, and a second copy of one fact is how the two drift.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SurfaceFailure {
    pub argv: String,
    pub stderr_tail: String,
}

impl From<&OpRow> for SurfaceFailure {
    fn from(row: &OpRow) -> Self {
        Self {
            argv: row.argv.clone(),
            stderr_tail: stderr_tail(&row.stderr),
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
mod tests {
    use super::*;

    fn entry(exit: i32, stderr: &str) -> OpEntry {
        OpEntry {
            ts: "TS".into(),
            argv: vec!["bl".into(), "close".into(), "bl-4db6".into()],
            cwd: "/proj".into(),
            exit,
            stdout: "out".into(),
            stderr: stderr.into(),
            origin: Origin::default(),
        }
    }

    #[test]
    fn op_row_carries_the_full_entry_and_joins_argv() {
        let row = OpRow::from(&entry(0, ""));
        assert_eq!(row.ts, "TS");
        assert_eq!(row.argv, "bl close bl-4db6");
        assert_eq!(row.cwd, "/proj");
        assert_eq!(row.exit, 0);
        assert_eq!(row.stdout, "out");
        assert_eq!(row.stderr, "");
    }

    /// bl-61db: the leading column reads `when()`, not the raw `ts`. An
    /// unparseable `ts` (the "TS" fixture above; every real line is a decimal
    /// epoch) falls back to itself rather than a made-up date.
    #[test]
    fn when_renders_iso8601_and_falls_back_on_a_non_numeric_ts() {
        assert_eq!(OpRow::from(&entry(0, "")).when(), "TS");
        let stamped = OpEntry {
            ts: "1785630266".into(),
            ..entry(0, "")
        };
        assert_eq!(OpRow::from(&stamped).when(), "2026-08-02 00:24:26Z");
    }

    #[test]
    fn has_output_reflects_either_stream() {
        assert!(OpRow::from(&entry(0, "")).has_output()); // stdout "out"
        let neither = OpEntry {
            stdout: String::new(),
            ..entry(0, "")
        };
        assert!(!OpRow::from(&neither).has_output());
        let only_err = OpEntry {
            stdout: String::new(),
            ..entry(1, "boom")
        };
        assert!(OpRow::from(&only_err).has_output());
    }

    /// bl-0bf9: a short, single-line `argv` renders as itself — no elision, no
    /// trailing `…`, when it already fits the cap.
    #[test]
    fn summary_leaves_a_short_argv_unchanged() {
        assert_eq!(OpRow::from(&entry(0, "")).summary(), "bl close bl-4db6");
    }

    /// bl-0bf9 acceptance: a 500-char multi-line goal (a prompt op's payload —
    /// a ball body reaches this length routinely) renders as ONE elided
    /// single-line summary of bounded length — the collapsed row's whole point
    /// — while the row's own `argv` field stays byte-exact, because the
    /// expansion (§4.2) must never lose bytes, only the collapsed summary
    /// elides.
    #[test]
    fn summary_folds_newlines_and_elides_a_long_multiline_goal() {
        let goal = format!(
            "identity preamble\n{}\nball worktree preamble",
            "payload segment ".repeat(30)
        );
        assert!(goal.chars().count() >= 500, "fixture must exceed 500 chars");
        let e = OpEntry {
            argv: vec![goal.clone()],
            ..entry(0, "")
        };
        let row = OpRow::from(&e);

        let summary = row.summary();
        assert!(!summary.contains('\n') && !summary.contains('\r'));
        assert_eq!(summary.chars().count(), SUMMARY_ARGV_MAX);
        assert!(summary.ends_with('…'));

        // The trail never loses bytes: only the collapsed summary elides.
        assert_eq!(row.argv, goal);
    }

    /// bl-0bf9: the elision boundary is exact — a flat argv of exactly the cap
    /// passes through whole; one char over triggers the `…`.
    #[test]
    fn summary_elision_boundary_is_exact() {
        let at_cap = "a".repeat(SUMMARY_ARGV_MAX);
        let e = OpEntry {
            argv: vec![at_cap.clone()],
            ..entry(0, "")
        };
        assert_eq!(OpRow::from(&e).summary(), at_cap);

        let over_cap = "a".repeat(SUMMARY_ARGV_MAX + 1);
        let e = OpEntry {
            argv: vec![over_cap],
            ..entry(0, "")
        };
        let summary = OpRow::from(&e).summary();
        assert_eq!(summary.chars().count(), SUMMARY_ARGV_MAX);
        assert!(summary.ends_with('…'));
    }

    #[test]
    fn surface_failure_carries_argv_and_stderr_tail() {
        let row = OpRow::from(&entry(2, "line1\nline2"));
        let f = SurfaceFailure::from(&row);
        assert_eq!(f.argv, "bl close bl-4db6");
        assert_eq!(f.stderr_tail, "line1\nline2");
    }

    #[test]
    fn stderr_tail_keeps_only_the_last_lines() {
        assert_eq!(stderr_tail(""), "");
        assert_eq!(stderr_tail("only\n"), "only");
        assert_eq!(stderr_tail("a\nb\nc\nd\ne\n"), "c\nd\ne");
    }
}
