//! **The follow lane, read off the glass** (DESIGN §7.2, REMOTE §3; bl-73e7).
//!
//! The claim minting the lane was made for is one sentence: *bytes appended to
//! the open step file are on the screen with no derivation and no asker pass in
//! between*. That is a claim only the paint layer can witness, and only
//! negatively — the assertion that matters is what did **not** run.
//!
//! So this beat pins both clocks and then moves neither. After the window has
//! settled, the derivation is held by pointer (`Rig::tick`'s equivalent is
//! never called, nothing is marked dirty) and the standing set is never
//! answered again (`World::reads` is not called). The only thing that runs
//! between the append and the frame is one look on the follow lane. Whatever
//! reaches the glass came through it.
//!
//! It is the honest replacement for the §7.2 follower's own beat, which made
//! the same assertion against a value **no production seat painted** — the
//! phantom coverage bl-73e7 retired along with the follower.

use std::sync::Arc;

use super::fixture::World;
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::keymap::InspectorTab;

const AGENT: &str = "c-1";

/// One `content_delta` line of answer text, as brazen's `v=1` writes it.
fn text_delta(fragment: &str) -> String {
    format!(
        "{{\"type\":\"content_delta\",\"index\":0,\"delta\":{{\"text_delta\":\"{fragment}\"}}}}\n"
    )
}

/// Publish `AGENT` as **in flight**, with the derivation's own fold of whatever
/// the response file holds right now.
///
/// This is the worker standing in for itself, and it is the fixture's job for
/// the same reason the transport is: liveness on a real box is a driver holding
/// an fd, which a static git fixture cannot carry. Nothing about the *fold* is
/// stood in for — `stream_from_disk` is the function the derivation calls, over
/// the real bytes — so what a beat below paints on the pull path is what the
/// engine would have answered.
fn flying(world: &mut World) {
    let ws = world.ws.clone();
    let mut snap = (**world.model.derivation()).clone();
    if let Some(tree) = snap.trees.get_mut(&ws) {
        for agent in tree.agents.iter_mut().filter(|a| a.agent_id == AGENT) {
            agent.state = AgentState::InFlight;
            agent.stream = crate::git_tree::stream_from_disk(&ws, AGENT);
        }
    }
    assert!(
        snap.trees.get(&ws).is_some_and(|t| t
            .agents
            .iter()
            .any(|a| a.agent_id == AGENT && a.state == AgentState::InFlight)),
        "the fixture's conversation is where this beat thinks it is"
    );
    crate::state::publish_snapshot(&world.model.snapshot_cell(), Arc::new(snap));
    world.model.refresh();
}

/// Append to the open response file of `AGENT`'s newest step — the literal file
/// litany's harness writes mid-call.
fn append(world: &World, bytes: &str) {
    use std::io::Write;
    let path = crate::git_tree::latest_response_path(&world.ws, AGENT)
        .expect("the fixture's conversation has a step");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open response");
    file.write_all(bytes.as_bytes()).expect("append");
}

/// **The beat this ball exists for.** The model says a word; it is on the glass
/// with no derivation and no asker pass in between. Then the call settles, the
/// lane's stream ends, and the seat swaps to the committed entry — with the
/// live row gone rather than standing beside it.
#[test]
fn appended_bytes_reach_the_glass_with_no_derivation_and_no_asker_pass() {
    let (litany, bl, bz) = (Cli::new("litany"), Cli::new("bl"), Cli::new("bz"));
    let mut world = super::fixture::world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, AGENT);
    world.model.select_tab(InspectorTab::Transcript);
    flying(&mut world);

    // The window, settled: the committed chat is on the glass and every
    // standing question has been answered once.
    let settled = super::painted(&mut world, &litany, &bl);
    assert!(
        !settled.contains("hark"),
        "nothing has been said yet:\n{settled}"
    );

    // From here nothing may move but the file. The derivation is pinned by
    // pointer, and the standing set is never asked again — `World::reads` is
    // not called below, so a character that reaches the glass reached it on the
    // lane.
    let pinned = Arc::clone(world.model.derivation());
    let ctx = egui::Context::default();
    let frame = |world: &mut World| {
        crate::paint_probe::text_of(&ctx.run(super::input(), |ctx| {
            super::super::render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
        }))
    };
    // egui measures before it paints on a fresh context, so the pane is laid
    // out once before anything is asserted off it (the `painted` pre-roll's own
    // reason).
    let _ = frame(&mut world);
    let _ = frame(&mut world);

    append(&world, &text_delta("hark"));
    world.follows();
    world.model.refresh();
    // Two frames, not one: egui's bottom-anchored chat settles onto a row that
    // was not there when it last measured (§11's own one-frame settle).
    let _ = frame(&mut world);
    let live = frame(&mut world);
    assert!(
        live.contains("hark"),
        "the character the model just wrote is on the glass:\n{live}"
    );
    assert!(
        Arc::ptr_eq(&pinned, world.model.derivation()),
        "and no derivation ran to put it there — this is the whole claim"
    );

    // It keeps writing, and the row grows: the thing a badge cannot do.
    append(&world, &text_delta(", a lark"));
    world.follows();
    world.model.refresh();
    let _ = frame(&mut world);
    let grown = frame(&mut world);
    assert!(
        grown.contains("hark, a lark"),
        "every character that landed is on the frame:\n{grown}"
    );
    assert!(Arc::ptr_eq(&pinned, world.model.derivation()));

    // The call settles. The lane's stream ends, the seat is told, and the live
    // row goes — leaving the committed transcript the pull read still carries,
    // with nothing painted twice. Nothing re-answers the standing set here
    // either, so what retires the row is the terminator and only that.
    let mut settled = (*pinned).clone();
    if let Some(tree) = settled.trees.get_mut(&ws) {
        for agent in tree.agents.iter_mut().filter(|a| a.agent_id == AGENT) {
            agent.state = AgentState::Quiescent;
        }
    }
    crate::state::publish_snapshot(&world.model.snapshot_cell(), Arc::new(settled));
    world.follows();
    world.model.refresh();
    let _ = frame(&mut world);
    let closed = frame(&mut world);
    assert!(
        !closed.contains("hark"),
        "the tail retires with its stream rather than standing over a \
         transcript that has moved on:\n{closed}"
    );
    assert!(
        closed.contains("pong reply"),
        "and the committed conversation is still what the pane paints:\n{closed}"
    );
}

/// **The pull path is the fallback, and it is not a degraded chat.** A window
/// whose lane never came up paints the tail the engine folded onto
/// `Query::Transcript` — at ask cadence, which is exactly what REMOTE §9.7's
/// migration left. The lane may fail without the chat failing, and this is the
/// witness for that half.
#[test]
fn with_no_lane_the_tail_still_arrives_on_the_pull_path() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = super::fixture::world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, AGENT);
    world.model.select_tab(InspectorTab::Transcript);
    // The bytes land BEFORE the derivation reads them, which is what makes this
    // the pull path: the tail the seat paints is the one the worker folded and
    // the engine answered, not one anything followed.
    append(&world, &text_delta("still heard"));
    flying(&mut world);

    // The lane is never answered — `World::follows` is not called below, which
    // is a window that has one and cannot reach it. What settles the chat is
    // the derivation plus the standing set, and both of those do run: this is
    // `wire::wired`'s dance with the lane taken out.
    let bz = Cli::new("bz");
    let ctx = egui::Context::default();
    let frame = |world: &mut World| {
        crate::paint_probe::text_of(&ctx.run(super::input(), |ctx| {
            super::super::render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
        }))
    };
    let _ = frame(&mut world);
    let _ = frame(&mut world);
    world.model.refresh();
    world.reads();
    world.acts();
    let _ = frame(&mut world);
    world.model.refresh();
    let _ = frame(&mut world);
    let painted = frame(&mut world);
    assert!(
        painted.contains("still heard"),
        "the tail arrives on the pull path when nothing else does:\n{painted}"
    );
}
