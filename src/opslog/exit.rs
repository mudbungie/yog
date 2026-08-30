//! What an ops row's `exit` field actually says (DESIGN §4.2, §11) — the one
//! classification of that integer and the one home of its wording.
//!
//! `exit` is not a number the operator can read on its own: three of its values
//! are §4.2 **sentinels** (`-1`/`-2`/`-3`/`-4`), and a negative integer in an
//! exit column reads as a signal death to anyone who has used a shell. Worse,
//! `-2` used to carry two opposite facts — a detached `litany prompt` that
//! handed off cleanly *and* one that never started (the spawn failure rode the
//! same line, bl-afa9) — so the trail could not tell "running fine" from "never
//! ran". The encoding was the lie and is fixed at the source: a spawn that never
//! launched now writes the synthetic-failure line every other never-launched
//! spawn writes ([`OpEntry::synthetic_failure`](super::OpEntry::synthetic_failure),
//! `-3`), and `-2` means exactly one thing — **launched detached**.
//!
//! [`ExitKind`] is that classification, and every question about the field is
//! asked of it: is this row a failure ([`OpRow::failed`]), a drift observation
//! ([`OpRow::drift`]), and what does its exit *say* ([`OpRow::exit_label`]).
//! One enum decides; no surface re-reads the integer or invents its own words.

use super::rows::OpRow;
use super::{DETACHED_EXIT, DRIFT_EXIT, PIPED_UNOBSERVED, SYNTHETIC_EXIT, YOG_STEP};

/// The shell convention a terminating signal is reported under: exit `128 + n`
/// ([`ExitInfo::shell_code`](crate::cli_outbound::ExitInfo::shell_code), the one
/// producer of these values in yog's log).
const SIGNAL_BASE: i32 = 128;

/// The highest signal number the `128 + n` reading is applied to (Linux tops out
/// at 64 with the realtime range). Past it the code is just a code.
const SIGNAL_MAX: i32 = 64;

/// What a row's `exit` integer means. Derived from the field plus `argv[0]`
/// (a `-3` line is a never-launched *spawn* unless its argv names the
/// [`YOG_STEP`] pseudo-binary, in which case no process was ever involved) —
/// never stored, and the only thing allowed to interpret the number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExitKind {
    /// A real, observed process status — `0` or any code that is not a
    /// sentinel and not a signal reading.
    Code(i32),
    /// A terminating signal, reported as `128 + n`; carries `n`.
    Signal(i32),
    /// `-1`: a piped verb that **ran** but whose status was unobservable.
    Unobserved,
    /// `-2`: a detached spawn that handed off. There is no exit to report — the
    /// child's status arrives arbitrarily later and is discarded by the reaper
    /// (§8.1). Whether it *stayed* healthy is the sink's story, not this field's
    /// ([`super::detached`]).
    Detached,
    /// `-3` on a real binary's argv: the spawn never started.
    NeverSpawned,
    /// `-3` on a `["yog-step", …]` argv: a step yog performs itself failed, with
    /// no process involved at all.
    StepFailed,
    /// `-4`: not an attempted action — yog's own §7.2 drift observation.
    Drift,
}

impl ExitKind {
    /// Classify `exit` in the light of `argv0` (the row's leading argv token).
    pub(crate) fn of(exit: i32, argv0: &str) -> Self {
        match exit {
            PIPED_UNOBSERVED => Self::Unobserved,
            DETACHED_EXIT => Self::Detached,
            SYNTHETIC_EXIT if argv0 == YOG_STEP => Self::StepFailed,
            SYNTHETIC_EXIT => Self::NeverSpawned,
            DRIFT_EXIT => Self::Drift,
            c if c > SIGNAL_BASE && c - SIGNAL_BASE <= SIGNAL_MAX => Self::Signal(c - SIGNAL_BASE),
            c => Self::Code(c),
        }
    }

    /// The exit said in words — the whole detail line the §11 expansion paints,
    /// not a number with a prefix. A real status states itself; a signal death
    /// states the signal *and* keeps the numeric code; every sentinel states the
    /// fact it stands for and never a fake integer.
    pub(crate) fn label(self) -> String {
        match self {
            Self::Code(c) => format!("exit {c}"),
            Self::Signal(n) => format!("killed by signal {n} (exit {})", SIGNAL_BASE + n),
            Self::Unobserved => "ran; exit not observable".to_owned(),
            Self::Detached => "detached — handed off, no exit to observe".to_owned(),
            Self::NeverSpawned => "failed to spawn — never started".to_owned(),
            Self::StepFailed => "step failed — nothing was spawned".to_owned(),
            Self::Drift => "drift observation — not an attempted action".to_owned(),
        }
    }
}

impl OpRow {
    /// This row's [`ExitKind`] — the single read of the `exit` field.
    fn kind(&self) -> ExitKind {
        ExitKind::of(self.exit, self.argv.split(' ').next().unwrap_or_default())
    }

    /// The exit column said in words (§11, bl-afa9): the operator never reads a
    /// bare sentinel. Delegates to [`ExitKind::label`], so the chip, the row and
    /// any other seat that reports an exit spell it identically.
    pub fn exit_label(&self) -> String {
        self.kind().label()
    }

    /// Whether this attempted action is a **failure** to render (§7.3). A clean
    /// code, a ran-but-unobservable piped status, and a drift observation are
    /// not failures. A [`ExitKind::Detached`] row is a failure only when it
    /// carries stderr — which for a `-2` line is the driver's own dying words,
    /// folded in at read time from its per-spawn sink ([`super::detached`]),
    /// since a spawn that never launched no longer writes one (it is
    /// [`ExitKind::NeverSpawned`]). Everything else — a non-zero code, a signal,
    /// a never-started spawn, a failed step — is.
    ///
    /// **Content is no longer the trigger, so this arm is honest again**
    /// (bl-b95e). It once read `!stderr.is_empty()` over a sink folded in
    /// unconditionally, substituting "the driver said anything" for an exit
    /// nobody observed — and litany's driver stderr is an *operator-notice*
    /// channel as much as a dying one, so bl-1296 bolted a phrase table
    /// (`opslog::notice`) on to hold the benign lines back. The fold is now
    /// gated on the **state** the launch produced ([`super::launch::stillborn`])
    /// and the table is gone: a folded tail means the derivation already found
    /// nothing where the launch's product should be, and the bytes are its
    /// diagnosis rather than its cause (§13.3's `driver.log` rule).
    pub fn failed(&self) -> bool {
        match self.kind() {
            ExitKind::Code(0) | ExitKind::Unobserved | ExitKind::Drift => false,
            ExitKind::Detached => !self.stderr.is_empty(),
            _ => true,
        }
    }

    /// Whether this row is a **handoff** (§6, bl-8433): a detached spawn that
    /// launched with nothing said against it. Distinct from
    /// [`failed`](Self::failed) — the two partition the `-2` sentinel, so no
    /// row is ever both. The §11 rollup layer ([`super::live::outcomes`]) reads
    /// this, never the raw `-2` sentinel, to classify the row
    /// `OpOutcome::Detached` rather than `Clean`: nobody observed the exit, so
    /// it must not read as one.
    pub(crate) fn detached(&self) -> bool {
        matches!(self.kind(), ExitKind::Detached) && !self.failed()
    }

    /// Whether this row is a detached spawn whose driver **died in the
    /// handoff** (bl-ab13): the `-2` sentinel with its sink folded in, which is
    /// the trail's one durable statement that a launch that reported success
    /// did not survive. The other half of the sentinel's partition — and the
    /// reading the §4.3 loop's stillbirth check asks for
    /// — a caller that re-read the integer itself would be inventing a second
    /// meaning for it, which is the defect this module exists to prevent.
    pub(crate) fn detached_died(&self) -> bool {
        matches!(self.kind(), ExitKind::Detached) && self.failed()
    }

    /// Whether this row is a **drift** observation (§7.2) rather than an
    /// attempted action: yog's own instrumentation naming a change nobody
    /// announced. Counted separately on the §11 chip
    /// ([`super::live::Activity`]) — an alarm about the watcher, never a failed
    /// verb, so it stays out of [`failed`](Self::failed) and out of the §7.3
    /// banner.
    pub fn drift(&self) -> bool {
        matches!(self.kind(), ExitKind::Drift)
    }
}

#[cfg(test)]
mod tests;
