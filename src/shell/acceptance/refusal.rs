//! **The wireless window refuses at the paint layer** (bl-dc14): a window
//! whose engine got no wire up paints the refusal INSTEAD of the shell — no
//! composer, no tabs, no roster, nothing that looks actionable — because every
//! read and act crosses the wire (REMOTE §1.2) and REMOTE §8 rules a terminal
//! instruction in front of a desktop launch is not an answer.

use super::fixture::world;
use super::painted;
use crate::cli_outbound::Cli;

/// The window with a recorded refusal paints the engine's own sentence and
/// none of the shell's controls; without one, the same world paints the shell.
/// Two directions, so the witness is about the refusal and not about a fixture
/// that happens to paint nothing.
#[test]
fn a_wireless_window_paints_the_refusal_and_no_operable_control() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = world();
    let shell = painted(&mut world, &litany, &bl);
    assert!(
        shell.contains("Workspaces:"),
        "the wired frame paints the shell:\n{shell}"
    );

    world
        .model
        .refuse_wire("bind 127.0.0.1:7737: Address already in use".to_owned());
    let refused = painted(&mut world, &litany, &bl);
    assert!(
        refused.contains(crate::wire::post::NO_WIRE),
        "the one sentence every wireless act receipt carries heads the frame:\n{refused}"
    );
    assert!(
        refused.contains("Address already in use"),
        "the engine's own reason, verbatim:\n{refused}"
    );
    assert!(
        refused.contains("127.0.0.1:0"),
        "the remedy names the port-zero path:\n{refused}"
    );
    for control in ["Workspaces:", "new conversation", "Start"] {
        assert!(
            !refused.contains(control),
            "{control:?} would only look actionable (bl-dc14):\n{refused}"
        );
    }
}
