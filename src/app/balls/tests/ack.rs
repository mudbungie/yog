//! The operator's two trail gestures against the live model (bl-c417): the ack
//! that quiets every banner and the chip without losing a row, and the clear
//! that starts a fresh trail. Both directions, end to end — write the line,
//! let the worker re-read the tail (`tick`), assert what the surfaces derive.

use super::{model, world};
use crate::AppModel;
use crate::opslog::{self, Origin};

/// Append one failed `litany prime` line attributed to `origin`.
fn fail(m: &AppModel, origin: Origin) {
    opslog::append(
        m.state_root(),
        &opslog::OpEntry::synthetic_failure(
            "TS".into(),
            vec!["litany".into(), "prime".into()],
            "/proj".into(),
            "unrecognized subcommand\n".into(),
            origin,
        ),
    )
    .unwrap();
}

/// Append one §7.2 drift observation.
fn drift(m: &AppModel) {
    opslog::append(
        m.state_root(),
        &opslog::OpEntry::drift("TS".into(), "unannounced", "/state".into(), "/root".into()),
    )
    .unwrap();
}

/// The operator complaint, closed in both directions: alarms before the ack,
/// quiet after it, and the whole trail still on screen throughout. The failure
/// is never retried — which is the case the old rule could not end, since a
/// banner cleared only on a *newer clean op of the same origin*.
#[test]
fn an_ack_quiets_the_banners_and_the_chip_without_hiding_a_row() {
    let w = world();
    let (_c, mut m) = model(&w);
    fail(&m, Origin::Balls);
    fail(&m, Origin::Conversation);
    drift(&m);
    m.after_litany_verb();
    m.tick();
    assert!(m.last_failure(Origin::Balls).is_some());
    assert!(m.last_failure(Origin::Conversation).is_some());
    assert_eq!(m.activity().errors, 2);
    assert_eq!(m.activity().drifts, 1);
    assert!(m.activity().alarming(), "and the pane offers its Dismiss");

    m.ack_failures();
    m.tick();
    assert!(
        m.last_failure(Origin::Balls).is_none(),
        "every surface's banner is quiet, not just the one it was clicked on"
    );
    assert!(m.last_failure(Origin::Conversation).is_none());
    assert_eq!(m.activity().errors, 0);
    assert_eq!(
        m.activity().drifts,
        0,
        "drift is an alarm too, and is acknowledged with the rest (§7.2)"
    );
    assert!(
        !m.activity().alarming(),
        "so the Dismiss control retires with them"
    );
    assert_eq!(
        m.activity().total,
        4,
        "three rows plus the ack — the chip's op count is the trail's"
    );
    let rows = &m.snap.ops;
    assert_eq!(rows.len(), 4, "ack quiets alarms; it hides no history");
    assert!(
        rows.iter().filter(|r| r.failed()).count() == 2,
        "the acked failures are still failures in the expanded trail"
    );
    let ack_row = rows.last().unwrap();
    assert_eq!(ack_row.argv, "yog-step ack-failures");
    assert_eq!(ack_row.origin, Origin::World);
    assert!(!ack_row.failed(), "and the ack row is not itself an alarm");
}

/// The re-alarm: the ack is a watermark over the log, not a mute switch, so the
/// next failure is news again — on its own surface, and on no other.
#[test]
fn a_new_failure_after_an_ack_banners_again() {
    let w = world();
    let (_c, mut m) = model(&w);
    fail(&m, Origin::Balls);
    m.ack_failures();
    m.tick();
    assert!(m.last_failure(Origin::Balls).is_none());

    fail(&m, Origin::Conversation);
    m.after_litany_verb();
    m.tick();
    assert_eq!(
        m.last_failure(Origin::Conversation).unwrap().argv,
        "litany prime"
    );
    assert!(
        m.last_failure(Origin::Balls).is_none(),
        "and the acked one stays acknowledged — bl-48f8's per-surface rule holds"
    );
    assert_eq!(m.activity().errors, 1);
}

/// The clear verb: a fresh trail whose one row is the clear itself. The rows
/// are gone from disk — that is the ask — and the record of the asking is what
/// keeps it from being a silent loss (§4.2 as amended).
#[test]
fn clear_leaves_a_one_row_trail_whose_row_is_the_clear() {
    let w = world();
    let (_c, mut m) = model(&w);
    fail(&m, Origin::Balls);
    drift(&m);
    m.after_litany_verb();
    m.tick();
    assert_eq!(m.snap.ops.len(), 2);

    m.clear_trail();
    m.tick();
    let rows = &m.snap.ops;
    assert_eq!(rows.len(), 1);
    let row = rows.first().unwrap();
    assert_eq!(row.argv, "yog-step clear-trail");
    assert_eq!(row.origin, Origin::World);
    assert!(!row.failed());
    assert_eq!(row.exit_label(), "exit 0");
    assert!(m.last_failure(Origin::Balls).is_none());
    assert!(!m.activity().alarming());
    assert_eq!(m.activity().total, 1, "the chip counts the new trail only");
    assert_eq!(
        opslog::tail(m.state_root(), 64).len(),
        1,
        "and the file itself is the new trail, not a longer one filtered down"
    );
}
