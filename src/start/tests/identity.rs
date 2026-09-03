//! The conversation identity (§3.3): the per-workspace mint, the composer's
//! name prediction, and the **legacy** stamp parse — the read-only survivors of
//! the retired `You are <name>.` compose (bl-6920). What the goal says about
//! the *work* — prefills, the ball header, the driver cwd — is [`super::goal`]'s.

use crate::start::identity::mint_conversation;
use crate::start::{parse_identity_stamp, strip_identity_stamp};

/// The **legacy** parse reads the line the retired compose used to write
/// (`You are <x>.`, pre-0.0.4 roots only) — the fallback rung of the §3.3
/// display ladder and nothing else. No live path composes this shape, so the
/// fixture is the literal legacy bytes.
#[test]
fn parse_identity_stamp_reads_the_legacy_line_only() {
    let goal = "You are cobalt-gecko.\n\nfree prose";
    assert_eq!(parse_identity_stamp(goal).as_deref(), Some("cobalt-gecko"));
    // Line one only: the retired compose put the stamp there, so a `You are`
    // sentence further down the operator's own payload is prose, not a name.
    assert_eq!(parse_identity_stamp("hi\n\nYou are late."), None);
    // A post-bl-6920, hand-typed, or foreign root carries no stamp at all.
    assert_eq!(parse_identity_stamp("just do the thing"), None);
    // A real name is one token and the line ends in the period the compose emitted.
    assert_eq!(parse_identity_stamp("You are two words."), None);
    assert_eq!(parse_identity_stamp("You are ."), None);
    assert_eq!(parse_identity_stamp("You are cobalt-gecko"), None);
}

/// The legacy stamp's other inverse (§3.3): what the payload was before the
/// pre-bl-6920 harness stamped it — the string the display ladder's second rung
/// is drawn from, which is why rungs one and two can never be the same line.
/// A goal with no stamp — every new root — is its own payload, verbatim.
#[test]
fn strip_identity_stamp_gives_back_the_payload_the_stamp_was_prepended_to() {
    let goal = "You are cobalt-gecko.\n\nBall bl-1: fix\n\nbody";
    assert_eq!(strip_identity_stamp(goal), "Ball bl-1: fix\n\nbody");
    // No stamp (post-bl-6920 / foreign / hand-typed): the goal verbatim.
    assert_eq!(
        strip_identity_stamp("just do the thing"),
        "just do the thing"
    );
    assert_eq!(
        strip_identity_stamp("hi\n\nYou are late."),
        "hi\n\nYou are late."
    );
    // A stamp and nothing else: an empty payload, not a stray identity line.
    assert_eq!(strip_identity_stamp("You are cobalt-gecko."), "");
}

/// The conversation mint scans past the names its workspace's living agents
/// wear (§3.3) — the occupied set is per-workspace, and nothing wider.
#[test]
fn mint_conversation_skips_the_workspaces_occupied_names() {
    let first = mint_conversation(&[], &super::rng()).unwrap();
    assert!(!first.contains('-'), "one word, no compound (bl-d12f)");
    let second = mint_conversation(std::slice::from_ref(&first), &super::rng()).unwrap();
    assert_ne!(second, first, "an occupied name is scanned past");
}

/// **Mint parity** (§3.3, bl-cd38): the fire's mint is `litany::mint::mint`
/// drawn through the crate yog links, not a second list of yog's own. Same
/// seed, same occupied set, same word; and calling the crate directly with
/// those inputs lands the same word again, which is what says yog kept no draw
/// of its own. This is the assertion a re-grown local wordlist would fail.
#[test]
fn the_fire_draws_the_one_litany_mint() {
    let occupied = ["ash".to_owned(), "bay".to_owned()];
    let fired = mint_conversation(&occupied, &super::rng()).unwrap();
    assert_eq!(
        litany::mint::mint(&super::rng(), &occupied.iter().cloned().collect()).unwrap(),
        fired,
        "the word is the crate's own draw — yog holds no second list"
    );
    assert!(!occupied.contains(&fired));
}
