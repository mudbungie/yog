//! **A spawn row means a conversation** (bl-ab13, DESIGN §11): the deferred
//! half of that invariant, decided rather than done.
//!
//! Split from [`super`] at §12's cap on the seam the plan itself draws — the
//! lease table above compares a *drone's* idleness against the operator's own
//! number, and this one compares the loop's own birth record against a world
//! that never grew a conversation from it. Different evidence, different gate
//! (none), one move.

use super::*;
use crate::opslog::{OpEntry, OpRow};

/// The loop's own spawn row for `ball`, and the detached driver row it handed
/// off to, both stamped `ts` in [`WS`] — the pair one tick writes (§4.2), which
/// is exactly what makes the join need no field of its own.
fn birth_rows(ts: i64, ball: &str, conversation: &str, dying_words: &str) -> Vec<OpRow> {
    let fleet = crate::fleet::row::spawned(ts.to_string(), Path::new(WS), ball, conversation);
    let driver = OpEntry {
        ts: ts.to_string(),
        argv: vec!["lernie".to_owned(), "prompt".to_owned()],
        cwd: crate::nav::ws_key(Path::new(WS)),
        exit: crate::opslog::DETACHED_EXIT,
        stdout: String::new(),
        stderr: dying_words.to_owned(),
        origin: crate::opslog::Origin::Balls,
    };
    vec![OpRow::from(&driver), OpRow::from(&fleet)]
}

/// A world as [`snap`] makes one, plus an ops tail and a derivation that
/// completed at [`NOW`] — the causality the stillbirth check reads.
fn snap_with(ops: Vec<OpRow>) -> Snapshot {
    Snapshot {
        ops,
        derived_at_unix: NOW,
        ..snap(vec![])
    }
}

/// The loop's own spawn row, a driver that died in the handoff, and no
/// conversation on the ball — the claim comes back with no lease anywhere in
/// sight, because this is not a judgement about a quiet worker.
#[test]
fn a_birth_whose_driver_died_gives_the_claim_back_without_a_lease() {
    let droneless = vec![row("bl-1", Column::Claimed, vec![])];
    let snapshot = snap_with(birth_rows(NOW - 5, "bl-1", "OtterBrook", "no such role\n"));
    let Some(Move::Reap { row, since, .. }) = plan(&snapshot, &facts(3, 1, None), &droneless, NOW)
    else {
        panic!("a reap");
    };
    assert_eq!(row.id, "bl-1");
    assert_eq!(
        since, "spawn OtterBrook left no conversation",
        "the loop's record set against the world, never why the driver died"
    );
}

/// The two conditions that keep the check off a healthy birth: a driver with
/// nothing said against it, and a snapshot older than the spawn it is judging.
#[test]
fn a_live_handoff_and_a_snapshot_that_predates_the_spawn_are_left_alone() {
    let droneless = vec![row("bl-1", Column::Claimed, vec![])];
    let quiet = snap_with(birth_rows(NOW - 5, "bl-1", "OtterBrook", ""));
    assert_eq!(
        plan(&quiet, &facts(3, 1, None), &droneless, NOW),
        None,
        "a launch with nothing said against it is a birth still converging"
    );
    let stale = Snapshot {
        derived_at_unix: NOW - 9,
        ..snap_with(birth_rows(NOW - 5, "bl-1", "OtterBrook", "boom\n"))
    };
    assert_eq!(
        plan(&stale, &facts(3, 1, None), &droneless, NOW),
        None,
        "the conversation is missing from a snapshot taken before the spawn: \
         that is yog's own latency, not a fact about the world"
    );
}

/// A birth on ANOTHER ball, and a claim naming nobody, are not evidence about
/// this one.
#[test]
fn only_this_loops_own_birth_on_this_ball_is_evidence() {
    let droneless = vec![row("bl-1", Column::Claimed, vec![])];
    let elsewhere = snap_with(birth_rows(NOW - 5, "bl-7", "OtterBrook", "boom\n"));
    assert_eq!(plan(&elsewhere, &facts(3, 1, None), &droneless, NOW), None);
    let anonymous = vec![BoardRow {
        claimant: None,
        ..row("bl-1", Column::Claimed, vec![])
    }];
    assert_eq!(
        plan(
            &snap_with(birth_rows(NOW - 5, "bl-1", "OtterBrook", "boom\n")),
            &facts(3, 1, None),
            &anonymous,
            NOW
        ),
        None,
        "a row that names nobody cannot be released from anyone"
    );
}
