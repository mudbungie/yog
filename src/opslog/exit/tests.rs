//! The exit classification and its wording (bl-afa9): every sentinel says the
//! fact it stands for, and no two facts share a rendering.

use super::super::{
    DETACHED_EXIT, DRIFT_EXIT, OpEntry, OpRow, Origin, PIPED_UNOBSERVED, SYNTHETIC_EXIT, YOG_STEP,
};
use super::ExitKind;

fn row(exit: i32, stderr: &str) -> OpRow {
    OpRow::from(&OpEntry {
        ts: "TS".into(),
        argv: vec!["lernie".into(), "prompt".into(), "/ws".into()],
        cwd: "/proj".into(),
        exit,
        stdout: String::new(),
        stderr: stderr.into(),
        origin: Origin::default(),
    })
}

/// The `-2` half of bl-afa9: a detached spawn that handed off says so — never a
/// numeric exit, and never the words a failure would get.
#[test]
fn a_clean_detached_handoff_renders_as_detached() {
    let r = row(DETACHED_EXIT, "");
    assert_eq!(r.exit_label(), "detached — handed off, no exit to observe");
    assert!(!r.failed());
    assert!(!r.exit_label().contains("-2"));
}

/// The other half: a spawn that never started renders as failed-to-spawn. It is
/// a `-3` line now (the source fix), which is exactly why it can be worded
/// apart from the handoff above.
#[test]
fn a_spawn_that_never_started_renders_as_failed_to_spawn() {
    let r = row(SYNTHETIC_EXIT, "failed to spawn /bin/nope: No such file");
    assert_eq!(r.exit_label(), "failed to spawn — never started");
    assert!(r.failed());
}

/// A real signal death keeps its signal rendering — and its numeric code, which
/// is the fact the shell convention encodes.
#[test]
fn a_signal_death_renders_as_the_signal() {
    let r = row(128 + 9, "");
    assert_eq!(r.exit_label(), "killed by signal 9 (exit 137)");
    assert!(r.failed());
}

/// The three renderings are mutually distinct — the bug was two facts sharing
/// one rendering, so this is the property, not the individual strings.
#[test]
fn the_three_detached_and_signal_renderings_never_collide() {
    let labels = [
        row(DETACHED_EXIT, "").exit_label(),
        row(SYNTHETIC_EXIT, "boom").exit_label(),
        row(128 + 15, "").exit_label(),
    ];
    for (i, a) in labels.iter().enumerate() {
        for b in labels.iter().skip(i + 1) {
            assert_ne!(a, b, "two exit facts must never render alike");
        }
    }
}

/// A `-3` line whose argv names the `yog-step` pseudo-binary spawned nothing at
/// all, so "failed to spawn" would be a second lie in the same column.
#[test]
fn a_failed_yog_step_is_not_worded_as_a_spawn() {
    let e = OpEntry::step_failure(
        "TS".into(),
        "mint",
        "/proj".into(),
        "pool exhausted".into(),
        Origin::default(),
    );
    let r = OpRow::from(&e);
    assert_eq!(r.argv, format!("{YOG_STEP} mint"));
    assert_eq!(r.exit_label(), "step failed — nothing was spawned");
    assert!(r.failed());
}

/// A real status states itself, `0` included; an unobservable piped status says
/// it ran, since it did (§4.2's `-1`).
#[test]
fn real_codes_and_the_unobserved_sentinel_state_themselves() {
    assert_eq!(row(0, "").exit_label(), "exit 0");
    assert!(!row(0, "").failed());
    assert_eq!(row(2, "gate").exit_label(), "exit 2");
    assert!(row(2, "gate").failed());
    assert_eq!(
        row(PIPED_UNOBSERVED, "").exit_label(),
        "ran; exit not observable"
    );
    assert!(!row(PIPED_UNOBSERVED, "").failed());
}

/// Drift is not an attempted action and never a failure (§7.2) — it says so.
#[test]
fn drift_says_it_is_not_an_action() {
    let e = OpEntry::drift("TS".into(), "unannounced", "/state".into(), "/root".into());
    let r = OpRow::from(&e);
    assert!(r.drift() && !r.failed());
    assert_eq!(
        r.exit_label(),
        "drift observation — not an attempted action"
    );
    assert!(!row(0, "").drift());
}

/// The `128 + n` reading is bounded: past the signal range a large code is just
/// a code, and a negative non-sentinel is neither a signal nor a sentinel.
#[test]
fn the_signal_reading_is_bounded_at_both_ends() {
    assert_eq!(
        ExitKind::of(SIGNAL_TOP, ""),
        ExitKind::Signal(super::SIGNAL_MAX)
    );
    assert_eq!(
        ExitKind::of(SIGNAL_TOP + 1, ""),
        ExitKind::Code(SIGNAL_TOP + 1)
    );
    assert_eq!(ExitKind::of(super::SIGNAL_BASE, ""), ExitKind::Code(128));
    assert_eq!(ExitKind::of(-9, ""), ExitKind::Code(-9));
}

/// The top of the `128 + n` band.
const SIGNAL_TOP: i32 = super::SIGNAL_BASE + super::SIGNAL_MAX;

/// A detached row carrying stderr is the driver's *post-launch* death (folded
/// from its sink) — still the detached wording, because the handoff did happen;
/// the row is a failure on the strength of what the child said.
#[test]
fn a_driver_that_died_after_launching_stays_detached_and_fails() {
    let r = row(DETACHED_EXIT, "refusing: version skew\n");
    assert_eq!(r.exit_label(), "detached — handed off, no exit to observe");
    assert!(r.failed());
    assert!(!r.drift());
    assert_eq!(ExitKind::of(DRIFT_EXIT, "yog-drift"), ExitKind::Drift);
}

/// `OpRow::detached` (bl-8433) — the §11 rollup's one read of "is this a
/// handoff nobody has spoken against yet": true for a clean `-2`, false the
/// moment stderr folds in (that row is [`OpRow::failed`] instead, never both),
/// and false for every non-`-2` kind.
#[test]
fn detached_is_true_only_for_a_clean_handoff() {
    assert!(row(DETACHED_EXIT, "").detached());
    assert!(
        !row(DETACHED_EXIT, "refusing: version skew\n").detached(),
        "a post-launch death is a failure, not a bare handoff"
    );
    assert!(!row(0, "").detached());
    assert!(!row(SYNTHETIC_EXIT, "boom").detached());
    assert!(!row(PIPED_UNOBSERVED, "").detached());
}
