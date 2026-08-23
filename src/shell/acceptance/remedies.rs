//! **S0 step 5 at the paint layer** (bl-9e10): an unseeded wall's first turn
//! dies, and the window says both *why* and *what to do about it*.
//!
//! This is the second half of one beat. `scripts/drive/beats_unseeded.sh`
//! drives a genuinely unseeded wall on the real substrate — the degree the
//! whole harness had been seeding past since bl-1851 — and proves the two
//! declines really happen and really carry these words. It cannot read the
//! glass: the drive's screenshots are visual confirmation, never the transport
//! (stories.sh's STEERING RULE), and the inspector surfaces have no headless
//! spelling (bl-6233). So the words are the seam, and this end renders the
//! whole shell over each decline and reads the ruled remedy out of the paint
//! output. It is the split bl-55d8 already made ([`super::wound`]).
//!
//! **The two kinds, and why an unseeded wall produces exactly them.** The
//! seeded template names a provider row that reaches a wall only through
//! `config.toml`, so a wall with *nothing* in it declines CONFIG-kind — the
//! row does not resolve — and a wall with the row table and no sign-ins
//! declines AUTH-kind. Each has a ruled remedy and they are not the same one:
//! credentials are §8.3's Login, and a row that does not exist is a *file*,
//! which is the §9.1 raw-TOML editor (bl-dd7f, DESIGN §8.3 rule 6).

use super::fixture::world;
use super::painted;
use crate::cli_outbound::Cli;
use crate::opslog::{OpEntry, Origin};

/// The CONFIG-kind decline **verbatim, as the unseeded drive captured it** —
/// brazen's `ConfigError::UnknownProvider` inside lernie's `prompt` wrapper,
/// which is what the §8.1 detached sink folds into the ops row the §7.3 banner
/// renders. Copied out of a real `run-unseeded` phase-A sink, the way
/// [`super::wound`]'s `BZ_REFUSAL` was copied out of bl-55d8's falsifying run:
/// a paint beat over words no substrate emits proves nothing.
///
/// Note the shape against the one bl-9b52's screenshot caught: the pinned
/// lernie now names the row it routed to as well (`on provider row "…"`), and
/// the classifier reads both spellings, so this beat holds across that move.
///
/// **The row is one nothing ships** (bl-6244). It was `openai-chatgpt` until
/// bl-8c2d compiled offerable rows into a default install — after which that
/// name is never unknown, the drive phase these words were copied from stopped
/// producing a config decline at all, and re-pinning to a row that CAN go
/// missing is what keeps this a paint beat over words a substrate really
/// emits. Re-copied from `run-unseeded` phase A on 2026-08-20.
const CONFIG_DECLINE: &str = "lernie prompt: provider error (Config) on provider row \
     \"yogdrive-no-such-provider\": unknown provider `yogdrive-no-such-provider`";

/// The needle for the reason: brazen's own words, which appear nowhere in the
/// shell's vocabulary, so finding them in the paint output can only mean the
/// ops row's stderr reached it (INV-2 — a failure renders as itself).
const REASON_NEEDLE: &str = "unknown provider `yogdrive-no-such-provider`";

/// The needle for the **remedy**: the sentence bl-dd7f pairs with the reason,
/// naming the row that has to exist and the file it has to exist in.
const REMEDY_NEEDLE: &str = "no provider row named yogdrive-no-such-provider";

/// The §11 tab the config remedy routes to — its own label, so a rename moves
/// the control and this beat together (`CenterTab::label`).
const CONFIG_TAB: &str = "Config";

/// How many times `needle` reaches the glass. The remedy's control is counted
/// rather than found, because its label is a word the window already paints.
fn says(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

/// Lay one failed §4.2 row of `Origin::Conversation` — the shape a detached
/// `lernie prompt` that died leaves behind, and the one input
/// `AppModel::last_failure` reads for the composer's banner.
fn dead_dispatch(world: &mut super::fixture::World, stderr: &str) {
    let row = OpEntry::synthetic_failure(
        "T0".to_owned(),
        vec!["lernie".to_owned(), "prompt".to_owned()],
        "ws".to_owned(),
        stderr.to_owned(),
        Origin::Conversation,
    );
    crate::opslog::append(world.model.state_root(), &row).unwrap();
    // The banner reads the PUBLISHED snapshot, never disk (§7.2 / INV-1), so
    // the row only exists for the frame once a derivation has carried it.
    world.model.after_lernie_verb();
    world.converge();
    // The composer that paints an `Origin::Conversation` banner is the docked
    // one, and it renders only for a focused workspace — the bootstrap box is
    // its empty-world twin (bl-48f8: one surface, two seats, never both in a
    // frame). So the dead dispatch is banner-able exactly where the operator
    // who fired it is standing.
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.converge();
}

/// **THE BALL, config half.** An unseeded wall's first dispatch renders its
/// reason *and* the way out of it: before bl-dd7f this banner carried the
/// sentence and a Dismiss, and Dismiss puts the reason down without touching
/// the file that caused it.
#[test]
fn a_config_kind_dispatch_failure_paints_its_reason_and_the_way_out() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut shell = world();
    dead_dispatch(&mut shell, CONFIG_DECLINE);
    let text = painted(&mut shell, &lernie, &bl);

    assert!(
        text.contains(REASON_NEEDLE),
        "the decline's own words reach the glass:\n{text}"
    );
    assert!(
        text.contains(REMEDY_NEEDLE),
        "THE BALL: the remedy beside the reason, naming the row:\n{text}"
    );
    assert!(
        text.contains("config.toml"),
        "and the file it has to be added to:\n{text}"
    );
    // The control, not only the sentence — the §11 tab that opens the §9.1
    // editor. A remedy an operator cannot press is prose.
    //
    // Asserted as a DIFFERENCE, never as `contains("Config")`: the centre strip
    // and the navigator both paint that word in every frame, so the bare
    // containment is true of a window with no remedy on it at all — the
    // vacuity shape `beat-audit` names, reached at the paint layer. The button
    // is one more occurrence than the same window carries without the fault,
    // which is a claim only the control can satisfy.
    let mut bare = super::fixture::world();
    dead_dispatch(&mut bare, "lernie prompt: exit 1");
    let baseline = painted(&mut bare, &lernie, &bl);
    assert!(
        says(&text, CONFIG_TAB) > says(&baseline, CONFIG_TAB),
        "the route to the §9.1 editor is a control, not only a sentence:\n{text}"
    );
    // Additive, never a replacement: §8.3 rule 5's clause that a classification
    // must not become the only thing on screen holds for this kind too.
    assert!(
        text.contains(crate::opslog::operator::ACK_LABEL),
        "Dismiss stays where it was:\n{text}"
    );
}

/// Every other failure class keeps the banner it had. A transport reset is
/// nobody's remedy, and routing one to the config editor would be a guess with
/// a button on it — so the reason paints and the remedy does not.
///
/// *"Every other failure class"* is exact and stayed exact through bl-1296: a
/// detached driver's **notice** is not one. It never becomes a
/// `SurfaceFailure`, so it reaches no banner to keep — `opslog::notice` decides
/// that one line earlier, off the ops row, and this file's classifiers are
/// asked only about failures that already are failures.
#[test]
fn a_failure_of_another_class_earns_the_reason_and_no_config_route() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = world();
    dead_dispatch(
        &mut world,
        "lernie prompt: provider error (Transport) on provider row \
         \"openai-chatgpt\": connection reset by peer",
    );
    let text = painted(&mut world, &lernie, &bl);

    assert!(
        text.contains("connection reset by peer"),
        "the reason still renders as itself:\n{text}"
    );
    assert!(
        !text.contains(REMEDY_NEEDLE),
        "and claims no config remedy it cannot honour:\n{text}"
    );
    assert!(
        text.contains(crate::opslog::operator::ACK_LABEL),
        "Dismiss is what this class has:\n{text}"
    );
}

/// **THE BALL, auth half — S0 step 5 itself.** The fixture conversation's
/// latest step is a settled `kind:auth` decline, which is the shape an
/// unseeded-but-configured wall produces, and the window offers the sign-in
/// one control away rather than leaving the operator to find the Login tab.
///
/// Asserted here beside its config sibling, not only inside the whole-window
/// smoke: the two are one ruling with two arms, and a beat that held only one
/// of them would go green while the other's affordance was deleted.
#[test]
fn an_auth_kind_step_failure_paints_its_reason_and_offers_the_sign_in() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.converge();
    let text = painted(&mut world, &lernie, &bl);

    assert!(
        text.contains("failed on") && text.contains("credentials"),
        "the §13.3 derived state, in words:\n{text}"
    );
    assert!(
        text.contains("Login"),
        "THE BALL: the sign-in is one control away from the dead turn:\n{text}"
    );
    // The two remedies are told apart, or the classification is doing nothing:
    // a credential decline must not route to the config editor's sentence.
    assert!(
        !text.contains(REMEDY_NEEDLE),
        "an auth fault is not a config fault:\n{text}"
    );
}
