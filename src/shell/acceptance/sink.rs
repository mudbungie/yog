//! The §8.1 **detached stderr sink** at the paint layer — the pair of beats
//! that decide what a `-2` row's sink does to the window.
//!
//! Split from [`super::smoke`] at §12's cap when bl-1296 added the second one,
//! and on a seam rather than at the line: these two are one claim with two
//! arms — a launch that produced nothing must banner, a launch that produced
//! its conversation must not, **however loudly its sink talks** — and a suite
//! holding only the first would go green while the sink equated speech with
//! death, which is exactly the defect bl-1296 opened on and bl-b95e closed.
//! Read them together or neither is evidence.
//!
//! bl-b95e is why the second arm now carries a **death-shaped** line rather
//! than a benign one. Under the retired phrase table the two arms differed by
//! their prose, so the pair could only ever prove that the table matched what
//! it was written against. They now differ by the **state** the launch left
//! behind, and the sink text is deliberately the same shape in both — which is
//! the only arrangement in which "content is never the trigger" is a claim a
//! test can fail.

use super::fixture::world;
use super::painted;
use crate::cli_outbound::Cli;

/// The §3.3 name the fire minted, carried on the row's own `--name` — how the
/// verdict finds what the launch was supposed to produce.
const MINTED: &str = "vanished-heron";

/// One detached `lernie prompt` row, launched at `ts` for the conversation
/// `name`, appended to the trail exactly as `start::execute_prompt` writes it
/// (the goal last, the workspace before it, `--name` up front).
fn launched(world: &crate::shell::acceptance::fixture::World, ts: &str, name: &str) {
    let ws = world.ws.to_string_lossy().into_owned();
    crate::opslog::append(
        world.model.state_root(),
        &crate::opslog::OpEntry {
            ts: ts.into(),
            argv: vec![
                "lernie".into(),
                "prompt".into(),
                "--name".into(),
                name.into(),
                ws.clone(),
                "yo".into(),
            ],
            cwd: ws,
            exit: crate::opslog::DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: crate::opslog::Origin::Conversation,
        },
    )
    .unwrap();
}

/// Write `text` into the §8.1 sink the launch at `ts` routed its child's stderr
/// to — the file yog holds no fd on and reads back at fold time.
fn sink(world: &crate::shell::acceptance::fixture::World, ts: &str, text: &str) {
    let path = crate::opslog::detached::sink(world.model.state_root(), ts, &world.ws);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
}

/// bl-4895: a detached `lernie prompt` whose driver dies right after launch must
/// banner. The failure lands in the model on a **later sweep** — long after the
/// dispatch handler that fired the prompt returned — so a banner cached at
/// dispatch showed nothing at all (three live prompts, three populated §8.1 sink
/// files, zero banners). The paint is therefore derived per frame from
/// [`AppModel::last_failure`]; this drives the whole window with **no dispatch**
/// between the sink appearing and the frame.
///
/// The conversation the row names is one no agent in this world carries, which
/// since bl-b95e is what makes the launch stillborn: the driver died before
/// writing a branch, so there is nothing on disk where its product should be.
#[test]
fn a_driver_that_dies_after_launch_banners_on_the_sweep_that_folds_its_sink() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");

    // The launch: a clean `-2` row. Nothing to see yet.
    launched(&world, "17", MINTED);
    world.model.after_lernie_verb();
    world.converge();
    let quiet = painted(&mut world, &lernie, &bl);
    assert!(
        !quiet.contains("was retired"),
        "a live launch is not a failure:\n{quiet}"
    );

    // The driver then dies on a stale workspace config, into its §8.1 sink.
    sink(&world, "17", "lernie prompt: config: action was retired\n");

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

/// **THE BALL** (bl-b95e): the same drive over a launch whose conversation
/// **exists** — and a sink holding the byte-identical death line the beat above
/// banners on. The window must not alarm: no §7.3 banner, no ⚠ anywhere the row
/// reaches, and the row on the glass wearing the handoff badge.
///
/// The two beats differ in exactly one thing — whether the workspace holds the
/// conversation the row named — so this is the claim §13.3 makes about
/// `driver.log`, held against the §8.1 sink: *content is never the trigger*. It
/// also pins the defect no phrase table could reach, since the sink is
/// append-only for the driver's whole life and every sweep re-read its tail: a
/// driver that spoke once and went on working stayed red forever.
#[test]
fn a_live_launchs_sink_never_alarms_however_it_reads() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    // The trail open, because the row this beat is about is inside it (§11: the
    // accessory is a collapsed chip until the operator asks).
    world.state.activity_open = true;
    // Zero the watcher debounce so the launch's product derives on the very
    // next pass instead of on a wall clock this beat would have to sleep
    // against (the seam `super::name_column` opens for the same reason). It
    // leaves every other period at its default, the §7.3 grace included.
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    // The launch's product, on disk: a conversation wearing the very name the
    // row minted. This is the whole difference from the beat above.
    world.add_root("c-2", MINTED);
    world.model.mark_dirty([ws.clone()]);
    world.converge();

    launched(&world, "19", MINTED);
    sink(&world, "19", "lernie prompt: config: action was retired\n");
    world.model.after_lernie_verb();
    world.converge();
    let out = painted(&mut world, &lernie, &bl);

    let (glyph, _, _) = crate::theme::op_badge(crate::opslog::OpOutcome::Detached);
    let row = out
        .lines()
        .find(|line| line.starts_with(glyph))
        .unwrap_or_default();
    assert!(
        row.contains("lernie prompt --name"),
        "the row is on the glass wearing the handoff badge:\n{out}"
    );
    assert!(
        !row.ends_with('⋯'),
        "and offers no expansion, because nothing was folded into it: {row:?}"
    );
    assert!(
        !out.contains("was retired"),
        "the sink is not read at all, so its words reach nothing:\n{out}"
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
