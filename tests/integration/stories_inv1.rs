//! STORIES **INV-1** idle-is-pure: constructing an `AppModel` and ticking it with
//! no user action performs **no mutating spawn and no substrate write** (I7).
//! Since §16.7 W8 the §7.2 fetch cadence spawns *nothing at all* — the ball read
//! is an in-process typed store load — so the invariant tightens from "only
//! read-only spawns" to "no `bl` process at all"; §16.7 W13 then deleted the last
//! read-only spawns in the crate (W5's capability probes), so construction now
//! reaches the substrate by no process whatsoever. Nothing mints, seeds, claims,
//! or logs an ops line (STORIES "Invariant tests", DESIGN §2 I7, §7.2).
//!
//! **Time is the harness's, not the machine's** (bl-9006). The beat booted on
//! `SystemClock` and so measured whatever wall-clock a loaded gate happened to
//! give it. A pass that takes ≥ the cheap-sweep period is drift by §7.2's own
//! rule and writes a `yog-drift late` line — a *substrate write at idle* — so
//! under nine concurrent tarpaulins this beat read yog's honest self-accusation
//! as a mutation and reddened inside an unrelated agent's close gate. The
//! lateness was real and the line was correct; the beat's mistake was asking
//! the question in a unit it did not control. It drives a [`TestClock`] now,
//! which moves **only between passes** — the one place elapsed time means
//! "periods fell due" rather than "this pass ran late" — so the sweeps §7.2
//! schedules are exercised (they never were: five real-clock passes finish
//! inside one cadence, so every pass after the first swept nothing) while the
//! ops tail stays the assertion it was written to be. Lateness has its own
//! beat, on the crate's own lurching fake: `app::tests::worker`.

#![allow(clippy::unwrap_used)]

use crate::support::{Recorder, TestClock};
use balls::layout::Xdg;
use std::time::Duration;
use tempfile::tempdir;
use yog::binding::names_root;
use yog::cli_outbound::Cli;
use yog::projects::runner::BlStore;
use yog::world::layout_under;
use yog::{AppModel, Roots};

/// Comfortably past §7.2's 15 s full-sweep period, so every pass below owes one.
/// A period the beat cannot import (the schedule's consts are crate-private), so
/// the loop asserts the sweep actually fell due rather than trusting this number
/// to stay generous.
const FULL_SWEEP: Duration = Duration::from_secs(20);

#[test]
fn inv1_idle_construction_and_ticks_perform_no_mutation() {
    let root = tempdir().unwrap();
    let bin = tempdir().unwrap();
    // One hermetic balls state root, addressed through balls' OWN layout — the
    // same arithmetic the model's clone enumeration and the store read use.
    let state = root.path().join("state");
    let xdg = Xdg::with(root.path(), None, Some(&state.to_string_lossy()));
    let roots = Roots {
        yog_data: root.path().join("yog"),
        litany_data: root.path().join("litany"),
        yog_state: root.path().join("state-yog"),
        balls_clones: xdg.clones_dir(),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    // One cloned project with a founded landing and one ready ball on its store,
    // so the fetch cadence has a real read to perform (§5.1 #1/#2).
    let project = root.path().join("proj");
    std::fs::create_dir_all(&project).unwrap();
    let clone = xdg.clone_dir(&project);
    std::fs::create_dir_all(clone.landing().join("config")).unwrap();
    std::fs::create_dir_all(clone.store().join("tasks")).unwrap();
    std::fs::write(
        clone.store().join("tasks").join("bl-1.md"),
        "+++\ntitle = \"Ready\"\ncreated = 1\nupdated = 1\n+++\n",
    )
    .unwrap();
    // A `bl` recorder standing in for the physical binary: it must never run.
    let bl = Recorder::new(bin.path(), "bl").on("list", "[]", 0);

    let clock = TestClock::new();
    let (mut model, mut deriver) = AppModel::boot(
        roots.clone(),
        None,
        clock.arc(),
        Box::new(BlStore::new(xdg, Cli::new(bl.path()))),
        Some("me".to_owned()),
    );
    // Five derivation passes, driven by hand: in the app these are the worker
    // thread's (§7.2), and the frame's only duty is taking what they publish.
    //
    // The clock moves **between** them, never inside one: each pass therefore
    // runs in zero elapsed time (so §7.2's late-pass drift cannot fire — that
    // is `app::tests::worker`'s beat, not this one) while each finds a full
    // sweep due, the schedule having baselined its deadlines at construction. A
    // full sweep publishes even when it found nothing, which is exactly what
    // `step` reports — so the return value is this loop's own proof that the
    // sweep really fell due, rather than the beat idling past a period this
    // constant no longer clears.
    for pass in 0..5 {
        clock.advance(FULL_SWEEP);
        assert!(
            deriver.step(),
            "pass {pass} owed a full sweep and published it (§7.2)"
        );
        model.refresh();
    }

    // The read really happened: the ready ball is start-eligible (§3.5) — and it
    // reached the model with no process spawned at all (§16.7 W8).
    assert_eq!(model.startable().len(), 1, "the store read produced a ball");
    assert!(
        bl.invocations().is_empty(),
        "the fetch cadence spawns no process at all: {:?}",
        bl.invocations()
    );

    // No substrate write: nothing minted a workspace, seeded a home, or logged an
    // ops line (a mutating verb's tell).
    assert!(
        std::fs::read_dir(names_root(&roots.yog_data)).is_err(),
        "no workspace minted"
    );
    assert!(
        !layout_under(&roots.yog_data)
            .litany
            .join("models.yaml")
            .exists(),
        "no world seeded"
    );
    // Not one line, of any kind: no verb's, and — since every pass here swept
    // and none of them was late — none of §7.2's drift kinds either. *"A quiet
    // sweep writes nothing, and that silence is the target state."*
    assert!(
        yog::opslog::tail(&roots.yog_state, 16).is_empty(),
        "idle logs no ops line: {:?}",
        yog::opslog::tail(&roots.yog_state, 16)
    );
}
