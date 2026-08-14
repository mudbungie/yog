//! The clock's periods (DESIGN §7.2, bl-3381): yog's backend owns the only
//! clock in the system (VISION §4.3), and this is that clock's one setting —
//! the watcher-cycle cadence, persisted in `cadence.yaml` under the yog state
//! root and surfaced as §9.5 `Number` rows.
//!
//! **The file is the single fact; absence is the default.** [`parse`] is total:
//! a missing file, a missing field or an unparseable value falls to the
//! compiled default ([`DEBOUNCE`]/[`CHEAP_SWEEP`]/[`FULL_SWEEP`],
//! [`super::dirty`]), and an out-of-range number clamps to the same bounds the
//! §9.5 control enforces — so deleting the file *is* the reset, and a
//! hand-broken one degrades to the shipped rhythm rather than to a stall
//! (severability: removing the config removes the tuning, never the clock).
//!
//! The derived periods live here too ([`wound_grace`](Cadence::wound_grace),
//! [`late_pass`](Cadence::late_pass), [`stale_after`](Cadence::stale_after)):
//! each was a const spelled as a sum of the base cadences precisely so "a
//! change to either carries it along" (§7.2) — now that the bases move at
//! runtime, the derivation moves with them or the promise breaks.

use crate::model_pick::grammar::entry_field;
use std::time::Duration;

/// The settings file's basename, under the yog state root (§4.1's territory,
/// §7.1's `YogState` watch — a landed Apply reaches the worker as an ordinary
/// announced change, and a second instance converges on it, I0).
pub const CADENCE_YAML: &str = "cadence.yaml";
/// The file's column-0 block.
pub const BLOCK: &str = "cadence";
/// The one entry: the watcher-cycle clock. A future cadence (a sensor cycle's,
/// §4.3's loop) is a sibling entry — a row, not a rebuild (§9.5).
pub const WATCHER: &str = "watcher";

/// Field names — the §9.5 rows. One unit (milliseconds) across all three, so
/// the pane teaches no conversion table.
pub const DEBOUNCE_MS: &str = "debounce_ms";
pub const CHEAP_SWEEP_MS: &str = "cheap_sweep_ms";
pub const FULL_SWEEP_MS: &str = "full_sweep_ms";

/// Bounds, shared verbatim with the §9.5 `Number` controls so the pane and
/// [`parse`] cannot disagree. A zero debounce is legal (coalescing off); a
/// debounce past 10 s hides changes behind their own announcement. The sweeps
/// floor where the pass itself costs more than the period buys, and cap at the
/// point staleness stops being a cadence and starts being an outage.
pub const DEBOUNCE_BOUNDS: (u64, u64) = (0, 10_000);
pub const CHEAP_SWEEP_BOUNDS: (u64, u64) = (100, 600_000);
pub const FULL_SWEEP_BOUNDS: (u64, u64) = (1_000, 3_600_000);

/// The default file body — what the config pane seeds an absent file's draft
/// from, and (by the parse-fallback contract) exactly [`Cadence::default`].
pub const TEMPLATE: &str = "\
# yog's clock: watcher-cycle periods, in milliseconds.
# Delete this file to restore the defaults below.
cadence:
  watcher:
    debounce_ms: 100
    cheap_sweep_ms: 2000
    full_sweep_ms: 15000
";

/// The three periods of one watcher cycle (§7.2): the coalescing window a
/// dirty root waits, the cheap sweep (enumerations + reconcile + targeted
/// liveness), and the full sweep (re-derive everything). One value, carried on
/// the [`Snapshot`](super::Snapshot) so frame-side derived periods follow the
/// operator's tuning without a frame ever reading disk (bl-ee0a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cadence {
    pub debounce: Duration,
    pub cheap_sweep: Duration,
    pub full_sweep: Duration,
}

impl Default for Cadence {
    fn default() -> Self {
        Self {
            debounce: super::dirty::DEBOUNCE,
            cheap_sweep: super::dirty::CHEAP_SWEEP,
            full_sweep: super::dirty::FULL_SWEEP,
        }
    }
}

impl Cadence {
    /// How long a wound must persist before the §11 banner paints it
    /// (§7.2, bl-90bf): one cheap-sweep tick (the worst case before the poll
    /// that re-probes liveness marks the root) plus one debounce window (what
    /// the mark then waits before it is due to re-derive).
    pub fn wound_grace(&self) -> Duration {
        self.cheap_sweep.saturating_add(self.debounce)
    }

    /// How long one derivation pass may take before it is itself drift (§7.2)
    /// — **the promise of the pass that just ran** (bl-4b28). A pass is judged
    /// against the period of the sweep it did: one that swept nothing, or swept
    /// cheaply, owes the 2 s poll cadence it rides; a *full* sweep re-derives
    /// every workspace and is budgeted its own 15 s period, which is the whole
    /// reason that period is longer.
    ///
    /// Judging every pass by the cheap bound was the storm: a full sweep of a
    /// real workspace (110 branches, a `bl` fetch per project) cannot finish
    /// inside 2 s, so every one of them reported itself late — 4447 of 4472
    /// trail rows, one every 15 s, all of them the schedule working exactly as
    /// designed.
    pub fn late_pass(&self, sweep: super::dirty::Sweep) -> Duration {
        match sweep {
            super::dirty::Sweep::Full => self.full_sweep,
            super::dirty::Sweep::Cheap | super::dirty::Sweep::None => self.cheap_sweep,
        }
    }

    /// How stale a rendered snapshot may be before the §11 ops surface says so:
    /// twice the full-sweep period — the worker re-stamps the snapshot on every
    /// full sweep even when nothing changed, so exceeding two of them means
    /// passes are not completing, not that the world is quiet.
    pub fn stale_after(&self) -> Duration {
        self.full_sweep.saturating_mul(2)
    }
}

/// Read a `cadence.yaml` body into a [`Cadence`] — total, per the module
/// contract: absent/unparseable falls to the default, out-of-range clamps.
pub fn parse(text: &str) -> Cadence {
    let defaults = Cadence::default();
    Cadence {
        debounce: field_ms(text, DEBOUNCE_MS, DEBOUNCE_BOUNDS, defaults.debounce),
        cheap_sweep: field_ms(
            text,
            CHEAP_SWEEP_MS,
            CHEAP_SWEEP_BOUNDS,
            defaults.cheap_sweep,
        ),
        full_sweep: field_ms(text, FULL_SWEEP_MS, FULL_SWEEP_BOUNDS, defaults.full_sweep),
    }
}

/// One field through the §9.4 anchored grammar (the same reader the §9.5 pane
/// derives its rows from, so the worker and the pane cannot read one file two
/// ways), parsed and clamped, or `default`.
fn field_ms(text: &str, name: &str, (min, max): (u64, u64), default: Duration) -> Duration {
    match entry_field(text, BLOCK, WATCHER, name).map(|v| v.parse::<u64>()) {
        Some(Ok(ms)) => Duration::from_millis(ms.clamp(min, max)),
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_or_broken_file_is_the_default() {
        assert_eq!(parse(""), Cadence::default(), "no file is the default");
        assert_eq!(
            parse("cadence:\n  watcher:\n    debounce_ms: nonsense\n"),
            Cadence::default(),
            "an unparseable field falls to its default, not to an error"
        );
    }

    #[test]
    fn the_template_spells_the_default() {
        assert_eq!(parse(TEMPLATE), Cadence::default());
    }

    #[test]
    fn a_set_field_lands_and_an_out_of_range_one_clamps() {
        let text = "cadence:\n  watcher:\n    cheap_sweep_ms: 5000\n    full_sweep_ms: 1\n";
        let parsed = parse(text);
        assert_eq!(parsed.cheap_sweep, Duration::from_secs(5));
        assert_eq!(
            parsed.full_sweep,
            Duration::from_millis(FULL_SWEEP_BOUNDS.0),
            "below the floor clamps to it — the §9.5 control's own rule"
        );
        assert_eq!(
            parsed.debounce,
            Cadence::default().debounce,
            "absent field keeps its default"
        );
    }

    #[test]
    fn derived_periods_follow_the_bases() {
        let c = Cadence {
            debounce: Duration::from_millis(200),
            cheap_sweep: Duration::from_secs(4),
            full_sweep: Duration::from_secs(30),
        };
        assert_eq!(c.wound_grace(), Duration::from_millis(4200));
        // Both pass bounds are tuned bases, not one of them (bl-4b28).
        assert_eq!(
            c.late_pass(super::super::dirty::Sweep::Cheap),
            Duration::from_secs(4)
        );
        assert_eq!(
            c.late_pass(super::super::dirty::Sweep::Full),
            Duration::from_secs(30)
        );
        assert_eq!(c.stale_after(), Duration::from_mins(1));
        // And the defaults reproduce the pre-bl-3381 consts byte-for-byte.
        let d = Cadence::default();
        assert_eq!(d.wound_grace(), Duration::from_millis(2100));
        assert_eq!(d.stale_after(), Duration::from_secs(30));
    }
}
