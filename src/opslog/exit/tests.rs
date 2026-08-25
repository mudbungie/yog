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

/// A **notice** line lernie's driver prints on the way past a decline — the
/// shape bl-1296's retired phrase table was written over. It stands here as an
/// ordinary sink line now: bl-b95e moved the decision to whether the fold ran
/// at all, so nothing at this altitude reads its words.
const NOTICE: &str = "lernie: compaction landing [c-2] superseded — a compaction landed \
     since its fork point (ARCH §2.6); the branch continues\n";

/// A detached row carrying folded stderr is the driver's *post-launch* death —
/// still the detached wording, because the handoff did happen; the row is a
/// failure because the fold only runs over a launch the derivation already
/// found stillborn ([`crate::opslog::launch::stillborn`], bl-b95e).
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

/// **THE BALL** (bl-b95e): at this altitude a notice line is not special and
/// nothing here reads it. A `-2` row whose sink was folded in is a failure
/// whatever the words say — because the caller folds only over a launch whose
/// product is missing — and an unfolded one is a bare handoff. The phrase
/// table that used to sit between them is gone.
#[test]
fn the_sink_is_no_longer_read_for_what_it_says() {
    let spoke = row(DETACHED_EXIT, NOTICE);
    assert!(
        spoke.failed(),
        "a folded tail is the derivation's verdict, not lernie's prose"
    );
    assert!(spoke.detached_died());
    assert!(!spoke.detached());
    assert_eq!(
        spoke.exit_label(),
        "detached — handed off, no exit to observe"
    );
    let unfolded = row(DETACHED_EXIT, "");
    assert!(!unfolded.failed(), "and an unfolded handoff is no failure");
    assert!(unfolded.detached());
}

/// The two readings of the `-2` sentinel **partition** it: every detached row
/// is exactly one of silent-handoff and died. Asserted as the property,
/// because two of them overlapping is how a benign line reached the §7.3
/// banner under the old three-way split.
#[test]
fn the_detached_sentinels_two_readings_never_overlap() {
    for stderr in ["", NOTICE, "refusing: version skew\n"] {
        let r = row(DETACHED_EXIT, stderr);
        let hits = u8::from(r.detached()) + u8::from(r.detached_died());
        assert_eq!(hits, 1, "not exactly one reading of {stderr:?}");
    }
}

/// The reading is asked of the `-2` sentinel and of nothing else: a piped verb
/// that happened to print a notice-shaped line still exits as it exits.
#[test]
fn the_detached_readings_are_of_that_sentinel_only() {
    assert!(!row(0, NOTICE).detached_died());
    assert!(!row(2, NOTICE).detached_died());
    assert!(row(2, NOTICE).failed(), "a real non-zero exit still fails");
}
