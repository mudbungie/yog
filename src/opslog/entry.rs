//! **The record itself** — one attempted CLI action as it lands in
//! `ops.jsonl` (§4.2 as amended), and the three constructors for the lines no
//! process status ever backed. Split from [`super`] at §12's budget on the
//! seam that module already had: the log's *policy* (the size cap, the exit
//! sentinels, the argv-0 grammars, append and tail) is one subject, and the
//! shape of a line is another.

use super::{DRIFT_EXIT, Origin, SYNTHETIC_EXIT, YOG_DRIFT, YOG_STEP};

/// One attempted CLI action — the on-disk `ops.jsonl` record (§4.2 as amended):
/// a completed run's captured outcome, or a synthetic failure line for a spawn
/// or non-spawn step that never produced a process status.
///
/// `ts` is an already-formatted timestamp string supplied by the caller's
/// clock — unix seconds as decimal digits ([`crate::ui_state::Clock::stamp`],
/// the crate's timestamp convention), not RFC3339; this module never reads
/// time. [`rows::OpRow::when`] renders it for a human (bl-61db).
/// `origin` is the §7.3 attribution — which surface the gesture was made on
/// ([`Origin`]), recorded at dispatch because no reading of `argv`/`cwd` can
/// tell a ball-rung start's `litany new` from the composer's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpEntry {
    pub ts: String,
    pub argv: Vec<String>,
    pub cwd: String,
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
    pub origin: Origin,
    /// **Who made this gesture** (REMOTE §4.1, ruling 5 of round 1; bl-e59e):
    /// the identity the intake carried — a wire connection's certificate common
    /// name, or [`local`](crate::registry::Client::local) for the window, the
    /// `gestures/` inbox, `yog gesture` and yog's own loops.
    ///
    /// The trail recorded `argv`, `cwd`, `origin` and `exit` and nothing about
    /// *whose* act it was, so a workspace two seats share could not say who
    /// said what — and unlike every other fact here that is unrecoverable
    /// afterwards, since no later derivation can reconstruct the author of a
    /// line nobody attributed. The engine knew the name at every one of those
    /// acts: it reads the common name off the presented leaf, narrows the whole
    /// published derivation by it and prints it in the client roster.
    ///
    /// It changes at the rate of the **act**, not of the connection, which is
    /// why it is durable here while presence stays RAM (REMOTE §5). Its type is
    /// the registry's own [`Client`](crate::registry::Client), so a trail row
    /// and a roster row spell one identity one way, and the default is `local`
    /// rather than an empty string nothing could resolve.
    pub client: crate::registry::Client,
}

impl OpEntry {
    /// A **synthetic failure line** (§4.2 as amended): an attempted action that
    /// produced no process status. `argv` is the intended argv, `stderr` the
    /// failure text, `stdout` empty, `exit` [`SYNTHETIC_EXIT`]. This is the one
    /// place "attempted" diverges from "completed" — a spawn that never launched
    /// still leaves a rendered fact (the §7.3 row), never a dropped error.
    pub fn synthetic_failure(
        ts: String,
        argv: Vec<String>,
        cwd: String,
        stderr: String,
        origin: Origin,
        client: crate::registry::Client,
    ) -> Self {
        Self {
            ts,
            argv,
            cwd,
            exit: SYNTHETIC_EXIT,
            stdout: String::new(),
            stderr,
            origin,
            client,
        }
    }

    /// A non-spawn **step-failure line** (§4.2): the mint/mkdir/cross-check class
    /// that names no binary. Encodes `argv = ["yog-step", <step>]` over
    /// [`synthetic_failure`](Self::synthetic_failure); the start flow (Z3) logs
    /// its non-spawn aborts through this same encoding.
    pub fn step_failure(
        ts: String,
        step: &str,
        cwd: String,
        stderr: String,
        origin: Origin,
        client: crate::registry::Client,
    ) -> Self {
        Self::synthetic_failure(
            ts,
            vec![YOG_STEP.to_string(), step.to_string()],
            cwd,
            stderr,
            origin,
            client,
        )
    }

    /// A **completed** non-spawn step line (§4.2): the same `["yog-step",
    /// <step>]` encoding as [`step_failure`](Self::step_failure) with a real
    /// exit 0 — a step yog performed *itself* and finished, e.g. §3.6's
    /// `["yog-step","delete-workspace"]`. The sentinels are for failures; a step
    /// that succeeded has a status, so it states one, and the trail records the
    /// deletion rather than vanishing with its subject (§3.6, §4.2).
    pub fn step_done(
        ts: String,
        step: &str,
        cwd: String,
        origin: Origin,
        client: crate::registry::Client,
    ) -> Self {
        Self {
            ts,
            argv: vec![YOG_STEP.to_string(), step.to_string()],
            cwd,
            exit: 0,
            stdout: String::new(),
            stderr: String::new(),
            origin,
            client,
        }
    }

    /// A **drift line** (§7.2): what a sweep or the watch backend FOUND.
    /// `argv = ["yog-drift", <kind>]`, `cwd` the yog state root the observation
    /// was made from, and `roots` the newline-joined paths it names, carried in
    /// `stderr` (the field the §11 accessory already expands). Exit is
    /// [`DRIFT_EXIT`], so it is a counted alarm and never a failed action. Its
    /// origin is [`Origin::World`] and takes no parameter: a drift is yog's
    /// observation about its own watcher, made by no operator gesture, so there
    /// is no surface it could have come from.
    pub fn drift(ts: String, kind: &str, cwd: String, roots: String) -> Self {
        Self {
            ts,
            argv: vec![YOG_DRIFT.to_string(), kind.to_string()],
            cwd,
            exit: DRIFT_EXIT,
            stdout: String::new(),
            stderr: roots,
            origin: Origin::World,
            // yog's own observation about its own watcher, made by no operator
            // gesture — so its author is the in-world identity for the same
            // reason its origin takes no parameter (bl-e59e).
            client: crate::registry::Client::local(),
        }
    }
}
