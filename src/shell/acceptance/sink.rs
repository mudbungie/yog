//! The §8.1 **detached stderr sink** at the paint layer — the pair of beats
//! that decide what a `-2` row's folded tail does to the window.
//!
//! Split from [`super::smoke`] at §12's cap when bl-1296 added the second one,
//! and on a seam rather than at the line: these two are one claim with two
//! arms — a driver that dies must banner, a driver that files an operator
//! notice must not — and a suite holding only the first would go green while
//! the sink equated speech with death, which is exactly the defect bl-1296
//! closed. Read them together or neither is evidence.

use super::fixture::world;
use super::painted;
use crate::cli_outbound::Cli;

/// bl-4895: a detached `lernie prompt` whose driver dies right after launch must
/// banner. The failure lands in the model on a **later sweep** — long after the
/// dispatch handler that fired the prompt returned — so a banner cached at
/// dispatch showed nothing at all (three live prompts, three populated §8.1 sink
/// files, zero banners). The paint is therefore derived per frame from
/// [`AppModel::last_failure`]; this drives the whole window with **no dispatch**
/// between the sink appearing and the frame.
#[test]
fn a_driver_that_dies_after_launch_banners_on_the_sweep_that_folds_its_sink() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");

    // The launch: a clean `-2` row. Nothing to see yet.
    crate::opslog::append(
        world.model.state_root(),
        &crate::opslog::OpEntry {
            ts: "17".into(),
            argv: vec![
                "lernie".into(),
                "prompt".into(),
                ws.to_string_lossy().into_owned(),
                "yo".into(),
            ],
            cwd: ws.to_string_lossy().into_owned(),
            exit: crate::opslog::DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: crate::opslog::Origin::Conversation,
        },
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    let quiet = painted(&mut world, &lernie, &bl);
    assert!(
        !quiet.contains("was retired"),
        "a live launch is not a failure:\n{quiet}"
    );

    // The driver then dies on a stale workspace config, into its §8.1 sink.
    let sink = crate::opslog::detached::sink(world.model.state_root(), "17", &ws);
    std::fs::create_dir_all(sink.parent().unwrap()).unwrap();
    std::fs::write(&sink, "lernie prompt: config: action was retired\n").unwrap();

    // The sweep folds it in — no dispatch, no click. The next frame banners it.
    world.model.after_lernie_verb();
    world.converge();
    let banner = painted(&mut world, &lernie, &bl);
    assert!(
        banner.contains("was retired"),
        "the dead driver's stderr tail reaches the paint layer:\n{banner}"
    );
    assert!(
        banner.contains("⚠ lernie prompt"),
        "with the attempted argv:\n{banner}"
    );
}

/// **THE BALL** (bl-1296): the same drive, over a sink holding exactly one
/// benign lernie driver notice. The window must not alarm — no §7.3 banner, no
/// ⚠ anywhere the row reaches — and the ops trail must still hold the row,
/// marked with the notice badge and offering its expansion.
///
/// The hue claim is pinned where the hue lives (`theme::op_badge`'s own test:
/// the notice badge is not `ICHOR`). What only the paint layer can witness is
/// what this beat asserts: that the ⚠ and the banner text are absent from the
/// whole rendered window, and that the row is on the glass all the same — the
/// difference between a quieted alarm and a deleted row.
#[test]
fn a_driver_notice_on_the_sink_reaches_the_trail_without_alarming_the_window() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    // The trail open, because the row this beat is about is inside it (§11: the
    // accessory is a collapsed chip until the operator asks).
    world.state.activity_open = true;

    crate::opslog::append(
        world.model.state_root(),
        &crate::opslog::OpEntry {
            ts: "19".into(),
            argv: vec![
                "lernie".into(),
                "prompt".into(),
                ws.to_string_lossy().into_owned(),
                "yo".into(),
            ],
            cwd: ws.to_string_lossy().into_owned(),
            exit: crate::opslog::DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: crate::opslog::Origin::Conversation,
        },
    )
    .unwrap();
    // The driver declines a compaction landing and carries on — the class
    // lernie ARCH binds this file to, printed on a path that returns `Ok(())`.
    let sink = crate::opslog::detached::sink(world.model.state_root(), "19", &ws);
    std::fs::create_dir_all(sink.parent().unwrap()).unwrap();
    std::fs::write(
        &sink,
        "lernie: compaction landing [c-2] superseded — a compaction landed since \
         its fork point (ARCH §2.6); the branch continues\n",
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    let out = painted(&mut world, &lernie, &bl);

    let (glyph, _, _) = crate::theme::op_badge(crate::opslog::OpOutcome::Notice);
    assert!(
        out.lines()
            .any(|line| line.starts_with(glyph) && line.ends_with('⋯')),
        "the notice row is on the glass, badged and expandable:\n{out}"
    );
    assert!(
        !out.contains("⚠ lernie prompt"),
        "and no §7.3 failure banner is raised over it:\n{out}"
    );
    assert!(
        !out.contains("failed ⚠"),
        "nor does the chip count it:\n{out}"
    );
}
