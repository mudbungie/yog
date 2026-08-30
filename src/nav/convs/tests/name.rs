//! The §3.3 display ladder: the litany-stored name fact → the legacy goal-stamp
//! parse → first payload line → root id, one function, every seat.

use super::{agent, plain, unseen};
use crate::git_tree::AgentState;
use crate::nav::convs::{build, display_name, display_name_of, liveness, member_title, started_at};

#[test]
fn the_ladder_climbs_stamped_name_then_payload_line_then_root_id() {
    // The rule itself, rung by rung — nothing else in yog decides this.
    assert_eq!(
        display_name(Some("stench-pug"), "Ball bl-1: fix", "r1-0"),
        "stench-pug"
    );
    assert_eq!(
        display_name(None, "Ball bl-1: fix", "r1-0"),
        "Ball bl-1: fix"
    );
    assert_eq!(display_name(None, "", "r1-0"), "r1-0");
}

#[test]
fn the_litany_name_fact_is_rung_one_and_the_goal_stamp_its_legacy_fallback() {
    // bl-08f2: the fold every seat reads — the litany-stored blob wins, the
    // legacy `You are <x>.` parse covers only pre-0.0.4 roots (no blob), and
    // retiring that rung is one deletion in `Agent::name_fact`.
    let mut a = agent("r1-0", AgentState::Quiescent, 10);
    a.name = Some("pale-otter".to_owned());
    a.goal_name = Some("stale-stamp".to_owned());
    assert_eq!(a.name_fact().as_deref(), Some("pale-otter"));
    a.name = None;
    assert_eq!(
        a.name_fact().as_deref(),
        Some("stale-stamp"),
        "a pre-0.0.4 root still names itself by its goal stamp"
    );
    a.goal_name = None;
    assert_eq!(a.name_fact(), None);
}

#[test]
fn a_litany_named_row_and_a_named_child_read_the_same_rung() {
    // The fact rides any agent — root or descent child — with no special case;
    // the row title and the agents-side seats agree.
    let mut root = agent("r1-0", AgentState::Quiescent, 10);
    root.name = Some("pale-otter".to_owned());
    root.preview = Some("Ball bl-1: fix".to_owned());
    let mut child = agent("r1-0-c1", AgentState::Quiescent, 11);
    child.name = Some("quiet-heron".to_owned());
    let agents = [root.clone(), child];
    assert_eq!(display_name_of(&agents, "r1-0"), "pale-otter");
    let rows = build(&[root], "/ws", &unseen, 10, &plain, &[]);
    assert_eq!(rows[0].display_name(), "pale-otter");
    assert_eq!(rows[0].subtitle(), "Ball bl-1: fix");
}

#[test]
fn a_legacy_rung_title_declares_itself_display_only() {
    // bl-8068 (the energize/marbling-lake diagnosis): a goal-stamp title with
    // no litany-stored blob behind it renders, but litany cannot resolve it as
    // a message target — the row says so, so an operator never hands an
    // unaddressable name to an agent. A fact-named row and a nameless row both
    // stay silent: the first is addressable, the second claims no name.
    let mut legacy = agent("r1-0", AgentState::Quiescent, 10);
    legacy.goal_name = Some("marbling-lake".to_owned());
    assert!(legacy.name_display_only());
    let mut fact = agent("r2-0", AgentState::Quiescent, 20);
    fact.name = Some("energize".to_owned());
    fact.goal_name = Some("energize".to_owned());
    assert!(!fact.name_display_only());
    let bare = agent("r3-0", AgentState::Quiescent, 30);
    assert!(!bare.name_display_only());
    let rows = build(&[legacy, fact, bare], "/ws", &unseen, 100, &plain, &[]);
    let flag = |id: &str| {
        rows.iter()
            .find(|r| r.root_id == id)
            .is_some_and(|r| r.name_display_only)
    };
    assert!(flag("r1-0"), "the legacy rung carries the warning");
    assert!(!flag("r2-0"), "a fact-named row is addressable");
    assert!(!flag("r3-0"), "no name claimed, nothing to warn about");
}

#[test]
fn a_descent_member_is_titled_by_the_ladder_never_the_raw_id() {
    // bl-df72: the operator read "some incoherent timestamp" at the top of the
    // shell — the descent-tree member row painting agent ids. A member is
    // titled by its own rungs: name fact, else payload line, else (the honest
    // floor — an id is a fact, and only the ladder may spell it) the id.
    let mut child = agent("r1-0-c1", AgentState::Quiescent, 11);
    child.preview = Some("Analyze the files under the XDG location".to_owned());
    assert_eq!(
        member_title(&child),
        "Analyze the files under the XDG location"
    );
    child.name = Some("quiet-heron".to_owned());
    assert_eq!(member_title(&child), "quiet-heron");
    child.name = None;
    child.preview = None;
    assert_eq!(member_title(&child), "r1-0-c1");
    // The snapshot seat shares the floor: an id no agent carries is its name.
    assert_eq!(display_name_of(&[], "r1-0-c1"), "r1-0-c1");
}

#[test]
fn the_floor_spells_only_the_terminal_generation_of_a_chained_id() {
    // bl-63a1: the operator's screenshot — dozens of fan-out children each
    // titled with the FULL ancestry chain, one `<stamp>-<hash>` per
    // generation. The tree's indentation already states the lineage, so the
    // floor spells the agent's own trailing generation and nothing more.
    let chained = "20260803T045410Z-2f2d2165-20260803T045643Z-1e5f99d4-20260803T045647Z-0527cb2c";
    let child = agent(chained, AgentState::Quiescent, 11);
    assert_eq!(member_title(&child), "20260803T045647Z-0527cb2c");
    // Every floor seat shares the spelling — the direct ladder and the
    // snapshot seat's no-agent fallback alike.
    assert_eq!(display_name(None, "", chained), "20260803T045647Z-0527cb2c");
    assert_eq!(display_name_of(&[], chained), "20260803T045647Z-0527cb2c");
    // A root id is one generation — its own terminal segment, unchanged.
    assert_eq!(
        display_name(None, "", "20260803T045410Z-2f2d2165"),
        "20260803T045410Z-2f2d2165"
    );
    // An id the stamp grammar does not recognize (foreign, hand-made — a
    // malformed stamp among them) is spelled whole: the general path.
    assert_eq!(display_name(None, "", "my-branch"), "my-branch");
    assert_eq!(
        display_name(None, "", "2026080T045410Z-deadbeef"),
        "2026080T045410Z-deadbeef"
    );
    assert_eq!(
        display_name(None, "", "2026o803T04541oZ-deadbeef"),
        "2026o803T04541oZ-deadbeef"
    );
    // The upper rungs never reach the floor: a name or payload line wins.
    let mut named = agent(chained, AgentState::Quiescent, 11);
    named.name = Some("quiet-heron".to_owned());
    assert_eq!(member_title(&named), "quiet-heron");
    named.name = None;
    named.preview = Some("Research live sources".to_owned());
    assert_eq!(member_title(&named), "Research live sources");
}

#[test]
fn a_stamped_row_is_titled_by_its_name_and_subtitled_by_its_payload_line() {
    // §11: the name is the title, the first payload line rides weak beside it.
    // The two never collide — `git_tree::detect` takes the stamp off the preview
    // at its source, so the preview here is payload, never the identity line.
    let mut a = agent("r1-0", AgentState::Quiescent, 10);
    a.goal_name = Some("stench-pug".to_owned());
    a.preview = Some("Ball bl-1: fix".to_owned());
    let rows = build(&[a], "/ws", &unseen, 10, &plain, &[]);
    assert_eq!(rows[0].display_name(), "stench-pug");
    assert_eq!(rows[0].subtitle(), "Ball bl-1: fix");
}

#[test]
fn an_unstamped_row_is_titled_by_its_payload_line_with_no_subtitle() {
    // A foreign or hand-typed root: rung two is the title, so the subtitle is
    // empty rather than repeating it.
    let mut a = agent("r1-0", AgentState::Quiescent, 10);
    a.preview = Some("wire the gate".to_owned());
    let rows = build(&[a], "/ws", &unseen, 10, &plain, &[]);
    assert_eq!(rows[0].display_name(), "wire the gate");
    assert_eq!(rows[0].subtitle(), "");
}

#[test]
fn a_row_with_neither_is_titled_by_its_root_id() {
    let rows = build(
        &[agent("r1-0", AgentState::Quiescent, 10)],
        "/ws",
        &unseen,
        10,
        &plain,
        &[],
    );
    assert_eq!(rows[0].display_name(), "r1-0");
    assert_eq!(rows[0].subtitle(), "");
}

#[test]
fn the_agents_side_seats_read_the_same_ladder() {
    // The §11 center header and the §3.6 deletion gate hold agents, not rows;
    // they climb the identical rungs, and an unknown id is its own name.
    let mut a = agent("r1-0", AgentState::Quiescent, 10);
    a.goal_name = Some("stench-pug".to_owned());
    a.preview = Some("Ball bl-1: fix".to_owned());
    let agents = [a, agent("r2-0", AgentState::Quiescent, 20)];
    assert_eq!(display_name_of(&agents, "r1-0"), "stench-pug");
    assert_eq!(display_name_of(&agents, "r2-0"), "r2-0");
    assert_eq!(display_name_of(&agents, "ghost"), "ghost");
    assert_eq!(display_name_of(&[], "r1-0"), "r1-0");
    let named: Vec<String> = liveness(&agents).into_iter().map(|c| c.name).collect();
    assert_eq!(named, ["stench-pug", "r2-0"]);
}

/// bl-16da: the header's when-seat reads the id's own stamp out in extended
/// ISO 8601. The hash discriminator is not a timestamp and does not survive
/// into the headline — it hovers, with the raw id, because the id is the key.
#[test]
fn the_when_seat_reads_the_id_stamp_as_extended_iso8601() {
    let got = started_at("20260801T225418Z-2286254c");
    assert_eq!(got.label, "2026-08-01 22:54:18Z");
    assert!(
        !got.label.contains("2286254c"),
        "a hash is not a timestamp: {got:?}"
    );
    assert!(
        got.hover.starts_with("20260801T225418Z-2286254c"),
        "the raw id stays discoverable: {got:?}"
    );
    assert!(got.hover.contains("on-disk key"), "and says why: {got:?}");
}

/// A bare stamp with no discriminator is the same fact with one part missing —
/// the general path with an empty input, not a case of its own.
#[test]
fn a_stamp_with_no_discriminator_reads_the_same() {
    assert_eq!(started_at("20260801T225418Z").label, "2026-08-01 22:54:18Z");
}

/// Anything the stamp grammar does not recognize is its own label — the same
/// last-rung rule the display-name ladder ends on, so a foreign or hand-made
/// branch never renders a guessed date.
#[test]
fn an_id_that_is_not_a_stamp_is_its_own_label() {
    for id in [
        "r1-0",
        "",
        "2026080T225418Z-a",  // eight digits required
        "20260801T22541Z-a",  // six digits required
        "20260801T225418-a",  // the Z is the marker
        "2026080xT225418Z-a", // digits, not merely length
        "20260801T2254x8Z-a", // in the time half too
        "20260801225418Z-a",  // the T separates them
    ] {
        assert_eq!(started_at(id).label, id, "not a stamp: {id}");
        assert!(started_at(id).hover.starts_with(id));
    }
}

/// The pair is inspectable and comparable like every other view-model here.
#[test]
fn the_when_seat_is_comparable_and_printable() {
    let a = started_at("20260801T225418Z-aa");
    assert_eq!(a.clone(), a);
    assert!(format!("{a:?}").contains("StartedAt"));
}
