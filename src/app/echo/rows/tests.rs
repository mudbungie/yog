//! The row-altitude echo's own contract (REMOTE §9.7, bl-44e9): what a seat
//! adds to an answered §11 list, and what it refuses to add. The paint-layer
//! proof that an operator sees it is `shell::acceptance::echo`; these pin the
//! decisions that beat rides on.

use super::with_echo;
use crate::app::echo::{Echo, Target};
use crate::nav::convs::ConvRow;
use crate::transcript::Tone;
use std::path::{Path, PathBuf};

const WS: &str = "/ws";

fn ws() -> PathBuf {
    PathBuf::from(WS)
}

/// One answered row, as the boundary hands it over.
fn row(root_id: &str, name: Option<&str>, age_secs: i64) -> ConvRow {
    ConvRow {
        root_id: root_id.to_owned(),
        state: crate::git_tree::AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs,
        flight: None,
        attention: 0,
        members: 1,
        depth: 0,
        direct: 0,
        ball: None,
        name: name.map(str::to_owned),
        name_display_only: false,
        verdict: None,
        stoppable: false,
        stop_children: false,
        tone: Tone::Plain,
    }
}

fn ids(rows: &[ConvRow]) -> Vec<&str> {
    rows.iter().map(|r| r.root_id.as_str()).collect()
}

/// The whole of §3.4 at this altitude: a start whose branch does not exist yet
/// is a row anyway, in the operator's own words, faded, leading the list.
#[test]
fn a_start_with_no_row_yet_leads_the_list_in_the_operators_own_words() {
    let answered = vec![row("c-1", Some("other"), 30)];
    let echo = Echo::started(Path::new(WS), "stench-pug", "open the gate", 95);
    let rows = with_echo(Some(&echo), &ws(), answered, 100);

    assert_eq!(ids(&rows), ["stench-pug", "c-1"], "the start leads");
    let pending = rows.first().expect("the minted row");
    assert_eq!(
        pending.name.as_deref(),
        Some("stench-pug"),
        "keyed by the minted §3.3 name — the only identity a start has yet"
    );
    assert_eq!(pending.subtitle(), "open the gate", "the goal, verbatim");
    assert_eq!(
        pending.tone,
        Tone::Weak,
        "faded: yog's own word for it, not yet a statement (§11)"
    );
    assert_eq!(pending.age_secs, 5, "dated by the send");
    assert_eq!(pending.depth, 0, "a start is a root, so a fold keeps it");
    assert_eq!((pending.members, pending.direct), (1, 0));
    assert!(pending.ball.is_none(), "a start stamps no ball here");
}

/// Once the derivation carries the started conversation the echo stops adding a
/// row and only dates the one that is there — the same two-armed identity
/// `compose` reads off a snapshot, read off rows instead.
#[test]
fn a_target_already_in_the_answer_is_freshened_and_never_duplicated() {
    let answered = vec![row("c-1", Some("other"), 30), row("c-2", None, 40)];
    let by_name = Echo::started(Path::new(WS), "other", "open the gate", 95);
    let rows = with_echo(Some(&by_name), &ws(), answered.clone(), 100);
    assert_eq!(ids(&rows), ["c-1", "c-2"], "no second row for one thing");
    assert_eq!(rows[0].age_secs, 5, "dated by the send, not the derivation");
    assert_eq!(rows[1].age_secs, 40, "and nothing else moves");

    // A follow-up names an agent id rather than a minted name.
    let by_id = Echo::started(Path::new(WS), "unused", "hi", 95);
    let by_id = Echo {
        target: Target::Agent("c-2".to_owned()),
        ..by_id
    };
    let rows = with_echo(Some(&by_id), &ws(), answered, 100);
    assert_eq!(ids(&rows), ["c-1", "c-2"]);
    assert_eq!(rows[1].age_secs, 5);
}

/// Three ways to have nothing to say, and none of them is a case of its own:
/// no echo, an echo fired in another workspace, and a follow-up whose
/// conversation the answer no longer carries (inventing a row for it would be a
/// false definite, §10).
#[test]
fn nothing_pending_and_nothing_matching_hand_the_answer_straight_back() {
    let answered = vec![row("c-1", Some("other"), 30)];
    assert_eq!(with_echo(None, &ws(), answered.clone(), 100), answered);

    let elsewhere = Echo::started(Path::new("/other"), "stench-pug", "go", 95);
    assert_eq!(
        with_echo(Some(&elsewhere), &ws(), answered.clone(), 100),
        answered
    );

    let gone = Echo {
        target: Target::Agent("c-9".to_owned()),
        ..Echo::started(Path::new(WS), "unused", "hi", 95)
    };
    assert_eq!(
        with_echo(Some(&gone), &ws(), answered.clone(), 100),
        answered
    );
}

/// A clock that went backwards is not a row from the future, and an echo older
/// than the derivation's own reading does not age the row up.
#[test]
fn the_freshened_age_is_clamped_at_zero_and_never_older_than_the_answer() {
    let answered = vec![row("c-1", Some("other"), 3)];
    let future = Echo::started(Path::new(WS), "other", "go", 200);
    let rows = with_echo(Some(&future), &ws(), answered.clone(), 100);
    assert_eq!(
        rows[0].age_secs, 0,
        "clamped, exactly as every other age is"
    );

    let stale = Echo::started(Path::new(WS), "other", "go", 10);
    let rows = with_echo(Some(&stale), &ws(), answered, 100);
    assert_eq!(
        rows[0].age_secs, 3,
        "the answer already knew something newer"
    );
}

/// The two readers [`pending`](super::pending) injects, held to their contracts
/// where they live: a pending conversation is acknowledged about nothing, and a
/// stamp nothing joins renders as its own id.
#[test]
fn the_pending_rows_injected_readers_are_total_answers() {
    assert!(
        !super::unseen(crate::ui_state::SeenKind::Notify, "/ws", "c-1", "oid"),
        "nothing is acknowledged about a conversation that does not exist yet"
    );
    let stray = super::stray_ball("bl-9");
    assert_eq!(stray.id, "bl-9", "the id always renders (source 1)");
    assert_eq!(
        (stray.state, stray.title, stray.badge),
        (None, None, None),
        "and the join supplies nothing — `answer::conv_ball`'s own miss arm"
    );
}
