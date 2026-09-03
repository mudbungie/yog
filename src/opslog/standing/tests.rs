//! The §7.3 carrier's own fold (bl-4d81): every arm of [`Standing`], the two
//! ways an alarm ends composed in one answer, and the slicing argument the
//! projection's doc makes.

use super::{Standing, standings};
use crate::opslog::{OpRow, Origin};

/// One row, spelled by its argv and exit — the two fields every rule here keys
/// on. `cwd` is constant, since the retirement key is `(cwd, verb)` and these
/// cases are all about the verb.
fn row(argv: &str, exit: i32) -> OpRow {
    OpRow {
        ts: "1700".to_owned(),
        argv: argv.to_owned(),
        cwd: "/p".to_owned(),
        exit,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::Balls,
    }
}

/// The ack line as `opslog::operator` writes it.
fn ack() -> OpRow {
    row("yog-step ack-failures", 0)
}

fn words(rows: &[OpRow]) -> Vec<Standing> {
    standings(rows).into_iter().map(|v| v.standing).collect()
}

/// The vocabulary is total and every arm is reachable: a clean run, a handoff
/// with nothing said against it, a live wound, one a later clean run retired,
/// and one the operator acked.
#[test]
fn every_standing_is_reachable_from_one_trail() {
    let trail = [
        row("bl close x", 1), // retired by the clean close below
        row("bl close x", 0), // clean
        row("bz login", 2),   // acked by the watermark below
        ack(),
        row("litany prime", 3), // live: after the ack, never re-run
        row("litany prompt b", crate::opslog::DETACHED_EXIT), // handed off
    ];
    assert_eq!(
        words(&trail),
        [
            Standing::Retired,
            Standing::Clean,
            Standing::Acked,
            Standing::Clean,
            Standing::Live,
            Standing::Detached,
        ]
    );
}

/// The banner §7.3 describes, read off the answer: the rows standing `live`,
/// grouped by the origin they already carry. Nothing here classifies.
#[test]
fn one_banner_per_origin_is_the_live_rows_grouped_by_origin() {
    let mut ball = row("bl close x", 1);
    ball.origin = Origin::Balls;
    let mut conv = row("litany message y", 1);
    conv.origin = Origin::Conversation;
    let mut world = row("yog-step mint", crate::opslog::SYNTHETIC_EXIT);
    world.origin = Origin::World;

    let live: Vec<Origin> = standings(&[ball, conv, world])
        .into_iter()
        .filter(|v| v.standing == Standing::Live)
        .map(|v| v.row.origin)
        .collect();
    assert_eq!(live, [Origin::Balls, Origin::Conversation, Origin::World]);
}

/// A drift observation is not an attempted action, so it never stands `live` —
/// §7.2's alarm must not reach §7.3's banner.
#[test]
fn a_drift_row_stands_clean() {
    let drift = row("yog-drift unannounced", crate::opslog::DRIFT_EXIT);
    assert_eq!(words(&[drift]), [Standing::Clean]);
}

/// The doc's slicing argument, exercised: answering only the tail of a trail
/// whose ack line fell off the front gives every remaining row the standing it
/// had over the whole of it.
#[test]
fn slicing_a_prefix_off_changes_no_row_standing() {
    let whole = [ack(), row("litany prime", 1), row("bl close x", 1)];
    assert_eq!(
        words(&whole),
        [Standing::Clean, Standing::Live, Standing::Live]
    );
    assert_eq!(
        words(whole.get(1..).unwrap()),
        [Standing::Live, Standing::Live],
        "the dropped ack line left both rows after it, which is what they were"
    );
}

/// The tokens are the codec's whole vocabulary, and each is its own word.
#[test]
fn each_standing_spells_itself() {
    let all = [
        Standing::Clean,
        Standing::Detached,
        Standing::Live,
        Standing::Retired,
        Standing::Acked,
    ];
    let mut spelled: Vec<&str> = all.iter().map(|s| s.token()).collect();
    spelled.sort_unstable();
    spelled.dedup();
    assert_eq!(spelled.len(), all.len(), "no two standings share a word");
}
