//! **STORIES S0 step 3 at the paint layer** (bl-49cb): *"the reply streams into
//! the focused view"*. After Enter the center pane must be the started
//! conversation's transcript — not the new-conversation placeholder with the
//! reply arriving unwatched one row to the left.
//!
//! The gesture under test is the *absence* of one: nothing here selects a
//! conversation. The §3.4 claim the fire makes ([`AppModel::await_conversation`],
//! held by the minted §3.3 name) is spent by the frame's own
//! [`AppModel::refresh`], and the very next painted frame is the transcript.
//! The live tail's own shape — [`EntryKind::Streaming`](crate::transcript::EntryKind),
//! merged only while the agent is in flight — is proven in
//! `tests/integration/reply_streams.rs`; a headless fixture cannot hold the
//! flock that classifies one, so what this asserts is the *view*: the
//! conversation's transcript is what renders.

use super::fixture::world_titled;
use super::painted;
use crate::cli_outbound::Cli;
use crate::keymap::InspectorTab;

/// The fixture's root carries the **legacy** §3.3 identity stamp (a pre-0.0.4
/// root — a modern fire wears the litany `name` blob instead, bl-6920); the
/// claim spends off `name_fact` the same way on either rung.
const GOAL: &str = "You are stench-pug.\n\nfix the gate";

#[test]
fn a_fired_start_renders_its_own_transcript_with_no_selection_gesture() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = world_titled(GOAL);
    let ws = world.ws.clone();
    world.model.select_tab(InspectorTab::Transcript);

    // Where Enter leaves the operator today: the workspace focused, nothing
    // selected — the §11 birth block and its placeholder.
    let before = painted(&mut world, &litany, &bl);
    assert!(
        before.contains("select a conversation"),
        "the pre-start center is the placeholder:\n{before}"
    );
    assert!(
        !before.contains("pong reply"),
        "and no transcript is on screen yet:\n{before}"
    );

    // Enter. The fire minted `stench-pug` and claimed it; the root is already on
    // disk here, so the next frame's refresh spends the claim.
    world
        .model
        .await_conversation(&ws, "stench-pug", "fix the gate");
    world.converge();

    let after = painted(&mut world, &litany, &bl);
    assert!(
        !after.contains("select a conversation"),
        "the placeholder is gone — the start focused what it started:\n{after}"
    );
    assert!(
        after.contains("pong reply"),
        "the started conversation's transcript is what renders (S0.3):\n{after}"
    );
    assert!(
        after.contains("→ message stench-pug"),
        "and the composer is aimed at it, named not identified:\n{after}"
    );
}
