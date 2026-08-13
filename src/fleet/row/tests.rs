//! The ops rows are the loop's only durable, so their round trip is the whole
//! contract: what a spawn and a reap write must read back as the same act, the
//! reap's comparison must survive verbatim, and nothing else on the trail may
//! be mistaken for one.

use super::*;
use std::path::Path;

#[test]
fn a_spawn_round_trips_and_carries_no_reason() {
    let line = spawned("42".to_owned(), Path::new("/ws/a"), "bl-1f2a", "otter");
    assert_eq!(line.exit, 0, "a completed spawn is not a failure");
    assert_eq!(line.origin, Origin::Balls, "the board is where it acted");
    assert_eq!(
        of_rows(&[OpRow::from(&line)]),
        vec![Act {
            workspace: "/ws/a".to_owned(),
            verb: SPAWN.to_owned(),
            ball: "bl-1f2a".to_owned(),
            subject: "otter".to_owned(),
            reason: String::new(),
            ts: 42,
        }]
    );
}

#[test]
fn a_reap_carries_the_comparison_verbatim() {
    let line = reaped(
        "99".to_owned(),
        Path::new("/ws/a"),
        "bl-1f2a",
        "otter",
        "lease expired 14m ago",
    );
    let acts = of_rows(&[OpRow::from(&line)]);
    let act = acts.first().expect("one act");
    assert_eq!(act.verb, REAP);
    assert_eq!(
        act.reason, "lease expired 14m ago",
        "the reason is the comparison, stored exactly as it was made"
    );
    assert_eq!(act.subject, "otter", "the claimant it released it from");
}

#[test]
fn nothing_else_on_the_trail_reads_as_an_act() {
    let other = OpEntry {
        ts: "1".to_owned(),
        argv: vec!["bl".to_owned(), "close".to_owned(), "bl-1".to_owned()],
        cwd: "/p".to_owned(),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::Balls,
    };
    assert!(of_rows(&[OpRow::from(&other)]).is_empty());
    // Right pseudo-binary, wrong arity.
    let short = OpEntry {
        argv: vec![YOG_FLEET.to_owned(), SPAWN.to_owned()],
        ..other.clone()
    };
    assert!(of_rows(&[OpRow::from(&short)]).is_empty());
    // Right pseudo-binary and arity, unknown verb.
    let odd = OpEntry {
        argv: vec![
            YOG_FLEET.to_owned(),
            "scuttle".to_owned(),
            "bl-1".to_owned(),
            "otter".to_owned(),
        ],
        ..other
    };
    assert!(of_rows(&[OpRow::from(&odd)]).is_empty());
}

#[test]
fn the_last_act_is_the_newest_one_for_that_workspace() {
    let rows: Vec<OpRow> = [
        spawned("10".to_owned(), Path::new("/ws/a"), "bl-1", "one"),
        spawned("20".to_owned(), Path::new("/ws/b"), "bl-2", "two"),
        reaped("30".to_owned(), Path::new("/ws/a"), "bl-1", "one", "idle"),
    ]
    .iter()
    .map(OpRow::from)
    .collect();
    let acts = of_rows(&rows);
    assert_eq!(last_act(&acts, "/ws/a"), Some(30));
    assert_eq!(last_act(&acts, "/ws/b"), Some(20));
    assert_eq!(
        last_act(&acts, "/ws/c"),
        None,
        "a loop that has never acted names no last tick, rather than naming zero"
    );
}

#[test]
fn an_unreadable_stamp_reads_as_zero_rather_than_dropping_the_act() {
    let line = OpEntry {
        ts: "not-a-time".to_owned(),
        ..spawned("0".to_owned(), Path::new("/ws/a"), "bl-1", "one")
    };
    let acts = of_rows(&[OpRow::from(&line)]);
    assert_eq!(acts.first().map(|a| a.ts), Some(0));
}
