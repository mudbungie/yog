//! The ack watermark and the clear verb, both directions (bl-c417): an alarm
//! before the ack and quiet after it; a re-alarm on a NEW failure after the
//! ack; a cleared trail that is exactly one row, and that row the clear itself.

use super::*;
use crate::opslog::{self, Activity, YOG_DRIFT};
use tempfile::tempdir;

/// A failed `bl close` row.
fn failure() -> OpRow {
    OpRow::from(&OpEntry::synthetic_failure(
        "1".into(),
        vec!["bl".into(), "close".into(), "bl-4db6".into()],
        "/proj".into(),
        "gate refused\n".into(),
        Origin::Balls,
    ))
}

/// A §7.2 drift observation row.
fn drift() -> OpRow {
    OpRow::from(&OpEntry::drift(
        "2".into(),
        "unannounced",
        "/state".into(),
        "/root\n".into(),
    ))
}

/// The row [`ack`] writes, without going through the filesystem.
fn ack_row() -> OpRow {
    OpRow::from(&entry("3", Path::new("/state"), ACK_STEP))
}

#[test]
fn with_no_ack_every_row_is_still_live() {
    let rows = vec![failure(), drift()];
    assert_eq!(since_ack(&rows).len(), 2);
    assert_eq!(
        opslog::activity(&rows),
        Activity {
            total: 2,
            errors: 1,
            drifts: 1
        }
    );
}

/// The whole watermark: rows at or before the newest ack are out of every alarm
/// derivation, and the ack row itself is out too (it is not an alarm about
/// anything). `total` stays the whole tail — the chip counts the rows the pane
/// renders, and the pane renders all of them.
#[test]
fn an_ack_quiets_every_alarm_before_it_and_hides_no_row() {
    let rows = vec![failure(), drift(), ack_row()];
    assert!(
        since_ack(&rows).is_empty(),
        "nothing after the ack is unacknowledged"
    );
    assert_eq!(
        opslog::activity(&rows),
        Activity {
            total: 3,
            errors: 0,
            drifts: 0
        }
    );
    assert_eq!(
        opslog::activity(&rows).chip(),
        "activity · 3 ops",
        "no ⚠, no drift word — and the op count is the trail's, undiminished"
    );
}

/// The other direction, and the reason this is a watermark rather than a
/// deletion: a failure logged *after* the ack is news the operator has not
/// seen, so it alarms exactly as it would have before.
#[test]
fn a_new_failure_after_an_ack_re_alarms() {
    let rows = vec![failure(), ack_row(), failure()];
    assert_eq!(since_ack(&rows).len(), 1);
    assert_eq!(opslog::activity(&rows).errors, 1);
    // And a second ack quiets that one too — the *newest* ack is the watermark,
    // not the first one ever written.
    let rows = vec![failure(), ack_row(), failure(), ack_row()];
    assert_eq!(opslog::activity(&rows).errors, 0);
}

/// A row that merely resembles an ack is not one: the watermark reads the
/// pseudo-binary and the step together, the same two-token verb §6's retirement
/// keys on. Neither another `yog-step` nor the ack word under a different
/// argv[0] moves it.
#[test]
fn only_a_real_ack_line_moves_the_watermark() {
    let other_step = OpRow::from(&OpEntry::step_done(
        "9".into(),
        "mint",
        "/state".into(),
        Origin::World,
    ));
    let foreign = OpRow::from(&OpEntry::drift(
        "9".into(),
        ACK_STEP,
        "/state".into(),
        String::new(),
    ));
    assert_eq!(foreign.argv, format!("{YOG_DRIFT} {ACK_STEP}"));
    let rows = vec![failure(), other_step, foreign];
    assert_eq!(since_ack(&rows).len(), 3);
    assert_eq!(opslog::activity(&rows).errors, 1);
}

#[test]
fn ack_appends_a_clean_world_step_line_that_banners_nowhere() {
    let dir = tempdir().unwrap();
    ack(dir.path(), "17").unwrap();
    let rows: Vec<OpRow> = opslog::tail(dir.path(), 8)
        .iter()
        .map(OpRow::from)
        .collect();
    let row = rows.first().unwrap();
    assert_eq!(row.argv, format!("{YOG_STEP} {ACK_STEP}"));
    assert_eq!(row.ts, "17");
    assert_eq!(row.origin, Origin::World);
    assert!(!row.failed(), "an ack is not an alarm about itself");
    assert!(!row.drift());
    assert_eq!(row.exit_label(), "exit 0");
    assert!(!row.has_output(), "and it carries no captured streams");
}

/// The clear's contract, verbatim: one row left, and that row is the clear.
/// Nothing else survives — which is the point, and why the row recording the
/// asking is what makes it not a *silent* loss (§4.2 as amended).
#[test]
fn clear_leaves_a_one_row_trail_whose_row_is_the_clear() {
    let dir = tempdir().unwrap();
    for n in 0..3 {
        opslog::append(
            dir.path(),
            &OpEntry::synthetic_failure(
                n.to_string(),
                vec!["bl".into(), "close".into()],
                "/proj".into(),
                "boom".into(),
                Origin::Balls,
            ),
        )
        .unwrap();
    }
    assert_eq!(opslog::tail(dir.path(), 8).len(), 3);

    clear(dir.path(), "42").unwrap();
    let rows: Vec<OpRow> = opslog::tail(dir.path(), 8)
        .iter()
        .map(OpRow::from)
        .collect();
    assert_eq!(rows.len(), 1, "the trail is exactly its own first row");
    let row = rows.first().unwrap();
    assert_eq!(row.argv, format!("{YOG_STEP} {CLEAR_STEP}"));
    assert_eq!(row.ts, "42");
    assert_eq!(row.origin, Origin::World);
    assert!(!row.failed());
    assert_eq!(
        opslog::activity(&rows),
        Activity {
            total: 1,
            errors: 0,
            drifts: 0
        }
    );
}

/// A clear into a state root that does not exist yet founds it, like every
/// other append — the first thing an operator does is never a precondition.
#[test]
fn clear_founds_a_missing_state_root() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("yog").join("state");
    clear(&root, "1").unwrap();
    assert_eq!(opslog::tail(&root, 8).len(), 1);
}

/// The truncate is a `set_len` on an `O_APPEND` handle, so a line another
/// instance appends between the truncate and the clear's own write survives at
/// the front instead of being overwritten by a positioned write.
#[test]
fn a_concurrent_append_after_the_truncate_is_not_clobbered() {
    let dir = tempdir().unwrap();
    let mut handle = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.path().join(FILENAME))
        .unwrap();
    handle.set_len(0).unwrap();
    // The other instance's append lands first, at the (new) end of file.
    opslog::append(
        dir.path(),
        &OpEntry::step_done("7".into(), "mint", "/state".into(), Origin::World),
    )
    .unwrap();
    // Then this clear's own write, still through O_APPEND: after it, not over it.
    handle
        .write_all(&build_line(&entry("8", dir.path(), CLEAR_STEP)))
        .unwrap();
    let tail = opslog::tail(dir.path(), 8);
    assert_eq!(tail.len(), 2);
    assert_eq!(tail.first().unwrap().ts, "7");
    assert_eq!(
        tail.last().unwrap().argv,
        vec![YOG_STEP.to_owned(), CLEAR_STEP.to_owned()]
    );
}

/// §4.2's atomicity bound holds for both new shapes, and structurally: neither
/// carries a field that can grow — two fixed argv tokens, empty streams, and a
/// cwd that is yog's own state root.
#[test]
fn both_shapes_are_bounded_and_inside_the_pipe_buf_cap() {
    for step in [ACK_STEP, CLEAR_STEP] {
        let e = entry("1785630266", Path::new("/home/u/.local/state/yog"), step);
        assert!(e.stdout.is_empty() && e.stderr.is_empty());
        assert_eq!(e.argv.len(), 2);
        assert!(build_line(&e).len() <= opslog::CAP);
    }
}
