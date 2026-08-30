//! The §6 attention facts that ride on a derivation: what a conversation's
//! classified rest does to the strip, and the acknowledgement the tick holds.
//!
//! These sat in `tests/derive.rs` under a "plus" in its module doc — the tell
//! of a second responsibility. bl-2194 had to route the clean-rest test there
//! because `tests/focus.rs` was at 294/300; this is the seam that debt was
//! standing in for.

use super::Harness;
use super::derive::settle;
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::boundary::{Action, Query};
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::watch::Mark;
use crate::{AppModel, boundary::answer::queue::QueueRow};

/// The §6 decision queue as the escalation now reads it (REMOTE §9.7): the
/// boundary chokepoint's answer, which is what stands on the wire.
fn queue(model: &AppModel, deps: &Deps) -> Vec<QueueRow> {
    let Ok(Reply::Attention(rows)) = model.answer(deps, &Query::Attention, 0) else {
        panic!("attention answers attention");
    };
    rows
}

/// bl-2194: the strip is a **turn queue**. A conversation that ended its turn
/// *cleanly* — a complete `response.json`, no wound, so §4.4 classifies it
/// `Quiescent` — stirs the strip exactly as a stopped one does, and the ack
/// clears it through the same tip watermark. The state badge says which rest it
/// was; attention only says your turn has come.
#[test]
fn a_clean_rest_stirs_the_strip_just_as_a_wounded_one_does() {
    let h = Harness::new();
    h.write_response(
        "c-1",
        b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
    );
    let (_c, mut model) = h.model();
    assert_eq!(
        model.tree(&h.ws).map(|t| t.agents[0].state),
        Some(AgentState::Quiescent),
        "a complete response with no wound is a clean rest (§4.4)"
    );
    assert_eq!(model.strip_total(), 1, "your turn has come (§6 rule 2)");
    model.focus_agent(&h.ws, "c-1");
    assert_eq!(model.strip_total(), 0, "acked at the tip it rests on");
}

/// bl-aa1f: the §6 ack is a **state**, not a gesture. The tick re-stamps the
/// focused agent's evidence, so a mark landing on the conversation the operator
/// is reading never stirs the strip — and the same mark landing while they are
/// looking elsewhere does.
#[test]
fn the_tick_holds_the_focused_agents_acknowledgement() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    model.focus_agent(&h.ws, "c-1");
    assert_eq!(model.strip_total(), 0, "focusing acked the unseen stop");

    // A notify mark lands while c-1 is still the focused conversation.
    h.fx.mark_ref("refs/litany/notify/c-1");
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    settle(&mut model, &clock);
    assert!(
        model
            .focused_agent()
            .is_some_and(|a| a.notify_oid.is_some())
    );
    assert_eq!(
        model.strip_total(),
        0,
        "evidence that arrived while you were looking is already seen"
    );

    // The converse: look away (workspace focus acks nothing), and the next mark
    // to land raises the flag.
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    h.fx.mark_ref("refs/litany/budget-exhausted/c-1");
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    settle(&mut model, &clock);
    assert_eq!(
        model.strip_total(),
        1,
        "evidence that arrived while you weren't looking stirs the strip"
    );
}

/// bl-e160: the desktop escalation reads the **same** §6 derivation the strip
/// counts — so a conversation the strip counts is a conversation the desktop
/// can name, and the ack that clears one clears the other. The knob it is gated
/// on is armed by default.
///
/// Asked through the boundary chokepoint since bl-f297: the escalation's queue
/// is `Query::Attention` over the wire now, and the model's own
/// `decision_queue` accessor went with the migration, so this reads what that
/// seat reads.
#[test]
fn the_desktop_escalation_reads_the_strip_s_own_queue() {
    let h = Harness::new();
    h.write_response(
        "c-1",
        b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
    );
    let (_c, mut model) = h.model();
    assert!(model.notify_unfocused(), "armed by default (§4.1)");

    let deps = model.boundary_deps(&Cli::new("/no/litany"), &Cli::new("/no/bl"));
    let alerts = crate::alert::of_queue(&queue(&model, &deps));
    assert_eq!(
        model.strip_total(),
        alerts.len(),
        "one alert per thing the strip counts"
    );
    let one = alerts
        .first()
        .expect("the resting conversation is announced");
    // The wall's §3.1 leaf, then the conversation's §3.3 display name — the two
    // words an operator glancing at a desktop popup needs to place it.
    assert_eq!(one.summary, "ws · hello");
    assert_eq!(one.body, "came to rest — your turn");

    // The acknowledgement that empties the strip empties the desktop too.
    model.focus_agent(&h.ws, "c-1");
    assert!(crate::alert::of_queue(&queue(&model, &deps)).is_empty());
}

/// bl-22ab, a regression of bl-f5f6: **the address a row answers is the address
/// the next gesture takes**. The chokepoint resolves workspace *names* (REMOTE
/// §8), so a row that answered an engine-local path handed a remote seat two
/// values it could only be refused for — a disclosure and a broken teleoperation
/// in one field. The round trip is therefore the assertion: the very pair
/// `/attention` answered is posted straight back as `/seen`, unedited, and the
/// engine's own re-derived queue comes back empty.
#[test]
fn a_queue_rows_address_posts_straight_back_as_the_next_gesture() {
    let h = Harness::new();
    h.write_response(
        "c-1",
        b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
    );
    let (_c, model) = h.model();
    let deps = model.boundary_deps(&Cli::new("/no/litany"), &Cli::new("/no/bl"));
    let rows = queue(&model, &deps);
    let row = rows.first().expect("the resting conversation is queued");
    assert_eq!(
        row.workspace,
        crate::naming::leaf(&h.ws),
        "the §3.1 name, which is what a gesture addresses"
    );

    let answered = crate::test_support::engine::act(
        &model,
        &deps,
        "0",
        &Action::MarkSeen {
            workspace: row.workspace.clone(),
            agent: row.agent.clone(),
        },
    )
    .expect("the address the queue answered resolves");
    let Reply::Attention(remaining) = answered else {
        panic!("an acknowledgement answers the queue that remains");
    };
    assert!(remaining.is_empty(), "answered, so it is off the queue");
}
