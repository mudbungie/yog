//! The §8.1 launch verdict (bl-b95e): what a detached row's failure is derived
//! from, now that it is not derived from the sink's words.
//!
//! Every beat drives [`stillborn`] over a hand-built forest, because the point
//! of the rule is that the answer comes from the **world** — a fixture that
//! only appended ops lines could not tell the two verdicts apart at all.

use super::*;
use crate::git_tree::{Agent, AgentState, GitTree, Stream};
use crate::opslog::{DETACHED_EXIT, OpEntry, Origin};

const WS: &str = "/ws/cobalt";
const MINTED: &str = "vanished-heron";
/// Comfortably past [`GRACE`], so a beat that is not about the window says so.
const NOW: i64 = 10_000;
const GRACE: Duration = Duration::from_secs(20);

fn agent(id: &str, name: Option<&str>, state: AgentState, last_action: i64) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_owned(),
        tip_oid: format!("tip-{id}"),
        tip_short_oid: "tip".into(),
        tip_timestamp_unix: 0,
        last_action_unix: last_action,
        messages: 0,
        call_start_unix: None,
        steps: vec![],
        preview: None,
        stream: Stream::default(),
        tool_calls: vec![],
        state,
        truncated: false,
        failure: None,
        state_uncertain: false,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: name.map(str::to_owned),
        goal_name: None,
    }
}

/// A world holding exactly `agents` under [`WS`].
fn forest(agents: Vec<Agent>) -> HashMap<PathBuf, GitTree> {
    HashMap::from([(
        PathBuf::from(WS),
        GitTree {
            commits: vec![],
            agents,
        },
    )])
}

/// An empty world: no workspace derived at all.
fn nothing() -> HashMap<PathBuf, GitTree> {
    HashMap::new()
}

fn entry(ts: &str, argv: &[&str], exit: i32) -> OpEntry {
    OpEntry {
        ts: ts.into(),
        argv: argv.iter().map(|a| (*a).to_owned()).collect(),
        cwd: WS.into(),
        exit,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::Conversation,
    }
}

/// The row `start::execute_prompt` writes for a start named `name`.
fn prompt(ts: &str, name: &str) -> OpEntry {
    entry(
        ts,
        &["litany", PROMPT, NAME_FLAG, name, WS, "the goal"],
        DETACHED_EXIT,
    )
}

/// The row `boundary::control::advance` writes for a resume of `agent`.
fn advance(ts: &str, agent: &str) -> OpEntry {
    entry(ts, &["litany", ADVANCE, WS, agent], DETACHED_EXIT)
}

/// **THE BALL**: a start whose conversation is nowhere on disk produced
/// nothing. Both halves of the state hold vacuously — there is no agent to be
/// driven and none to have acted — which is exactly the class the §8.1 sink was
/// added for (bl-4895): the driver died before writing a branch.
#[test]
fn a_start_that_left_no_conversation_is_stillborn() {
    assert!(stillborn(
        &forest(vec![agent(
            "c-1",
            Some("other-name"),
            AgentState::Quiescent,
            1
        )]),
        &prompt("100", MINTED),
        NOW,
        GRACE
    ));
}

/// And the other direction, which is the whole of the ruling: the same launch,
/// the same row, with its conversation on disk — no verdict, so the sink is
/// never read and its words decide nothing.
#[test]
fn a_start_whose_conversation_is_there_is_not() {
    assert!(!stillborn(
        &forest(vec![agent("c-1", Some(MINTED), AgentState::Quiescent, 200)]),
        &prompt("100", MINTED),
        NOW,
        GRACE
    ));
}

/// A driver **at work** is the answer to "did it survive", so nothing else is
/// asked — even where the branch has not acted since the launch (a model call
/// in flight has written no step yet).
#[test]
fn a_driven_target_is_never_stillborn() {
    for state in [AgentState::Live, AgentState::InFlight] {
        assert!(
            !stillborn(
                &forest(vec![agent("c-1", Some(MINTED), state, 1)]),
                &prompt("100", MINTED),
                NOW,
                GRACE
            ),
            "{state:?} is a driver at work"
        );
    }
}

/// A conversation that exists but has not moved since the launch, with nobody
/// driving it, is the launch having produced nothing after all — a branch minted
/// by an earlier fire whose newest driver never got anywhere.
#[test]
fn a_quiet_target_that_has_not_acted_since_the_launch_is_stillborn() {
    assert!(stillborn(
        &forest(vec![agent("c-1", Some(MINTED), AgentState::Stopped, 99)]),
        &prompt("100", MINTED),
        NOW,
        GRACE
    ));
}

/// The §8.2 resume is the same rule with the other spelling of a target: the
/// row names an **agent id**, and a step written after the launch is its
/// product.
#[test]
fn a_resume_is_judged_on_the_agent_it_named() {
    let dead = forest(vec![agent("c-1", None, AgentState::Stopped, 99)]);
    assert!(stillborn(&dead, &advance("100", "c-1"), NOW, GRACE));
    let moved = forest(vec![agent("c-1", None, AgentState::Stopped, 101)]);
    assert!(!stillborn(&moved, &advance("100", "c-1"), NOW, GRACE));
    // A resume of an agent this world has never heard of is stillborn for the
    // same vacuous reason a start with no conversation is.
    assert!(stillborn(&moved, &advance("100", "c-9"), NOW, GRACE));
}

/// The §7.3 grace window (bl-90bf): a launch younger than it has not had time
/// to produce anything, and yog's own derivation may simply not have looked yet
/// (bl-18e8's rising edge). Asserted on both sides of the boundary so a window
/// that stopped holding cannot pass.
#[test]
fn a_launch_inside_the_grace_window_has_no_verdict() {
    let world = forest(vec![]);
    assert!(!stillborn(&world, &prompt("100", MINTED), 119, GRACE));
    assert!(stillborn(&world, &prompt("100", MINTED), 120, GRACE));
}

/// A workspace with no derived tree is **no verdict**, never a death (§10: never
/// a false definite) — a start into a wall yog has not enumerated yet must be
/// silent, not accused.
#[test]
fn a_world_yog_cannot_read_answers_nothing() {
    assert!(!stillborn(&nothing(), &prompt("100", MINTED), NOW, GRACE));
}

/// The question is asked of the `-2` sentinel and of nothing else: a piped verb
/// and a synthetic failure carry their own observed status.
#[test]
fn only_a_detached_line_is_asked() {
    let piped = entry("100", &["litany", PROMPT, NAME_FLAG, MINTED, WS, "g"], 0);
    assert!(!stillborn(&forest(vec![]), &piped, NOW, GRACE));
}

/// An argv whose product cannot be named cannot be found missing: a verb yog
/// does not fire detached, a pre-bl-08f2 `prompt` line with no `--name`, a
/// truncated line, and a `ts` that is not a stamp all answer nothing rather
/// than accusing.
#[test]
fn a_row_that_names_no_target_answers_nothing() {
    let world = forest(vec![]);
    for argv in [
        vec!["litany", "prime", WS, "x"],
        vec!["litany", PROMPT, WS, "the goal"],
        vec!["litany", PROMPT, NAME_FLAG],
        vec!["litany", ADVANCE, WS],
        vec!["litany"],
    ] {
        let e = entry("100", &argv, DETACHED_EXIT);
        assert!(!stillborn(&world, &e, NOW, GRACE), "{argv:?}");
    }
    assert!(!stillborn(&world, &prompt("TS", MINTED), NOW, GRACE));
}
