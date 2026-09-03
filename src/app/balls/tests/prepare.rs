//! The start flow's first act (§8.1 step 1): `Action::Prepare` through the one
//! chokepoint — what it resolves, what it refuses, and by which name.
//!
//! **What the prepare MOVES is a seat's** (bl-1747, completed by bl-7942). The
//! §3.4 focus adoption rode `prepare_start`'s synchronous `Ok`; the act crossed
//! the wire at bl-1747 and the adoption hung off the receipt, and with the
//! window gone the whole notion of a focus is the seat's (REMOTE §7). What this
//! file pins is the half that was always the engine's: the raise founds a wall
//! under the operator's own §3.1 name, and the reply names it — so a seat has
//! something to adopt, and a *failed* prepare answers no `Prepared` and there
//! is nothing to adopt.
//!
//! Hermetic: the world is pre-seeded (`models.yaml` present ⇒ `litany prime` skips,
//! §16.6 W3), so the only spawn is `litany new` — a fake that materializes the same
//! `<ws>/repo.git` marker the real one does, or fails, per test.

use super::{model, world};
use crate::boundary::{Action, reply::Reply};
use crate::cli_outbound::Cli;
use crate::test_support::{authoring_new_arm, engine};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// A fake `litany` authoring `new`'s workspace like the real one (ARCH §2.2,
/// the shared [`authoring_new_arm`]). Every other verb exits 0.
fn news() -> String {
    format!(
        "#!/bin/sh\ncase \"$1\" in\n{}esac\nexit 0\n",
        authoring_new_arm()
    )
}
/// A fake `litany` that refuses — the failed-prepare path.
const FAILS: &str = "#!/bin/sh\nprintf 'boom\\n' 1>&2\nexit 3\n";

/// Write `body` as an executable `litany` in `dir` and hand back its [`Cli`].
fn fake_litany(dir: &Path, body: &str) -> Cli {
    let path = dir.join("litany");
    crate::test_support::write_exec(&path, body);
    Cli::new(path)
}

/// The prepare as a start rung fires it (bl-1747): `Action::Prepare` carrying
/// §3.4's two axes, through the engine's own chokepoint, answering the
/// composer's [`Prepared`](crate::start::Prepared) or the refusal.
fn staged(
    m: &mut crate::AppModel,
    litany: &Cli,
    inputs: &crate::start::StartInputs,
) -> Result<crate::start::Prepared, String> {
    // `bl` is never spawned here: no rung under test mutates a ball.
    let deps = m.boundary_deps(litany, &Cli::new("/no/bl"));
    let action = Action::Prepare {
        workspace: m.snap.ws_name(&inputs.workspace),
        payload: inputs.payload.clone(),
    };
    match engine::act(m, &deps, "TS", &action)? {
        Reply::Prepared(prepared) => Ok(prepared),
        other => panic!("the prepare door answers a Prepared, not {other:?}"),
    }
}

/// The §3.4 inputs a start rung hands the prepare, built from the model's own
/// roots. It was three `AppModel` constructors (`new_workspace_inputs`,
/// `start_bare_inputs`, `new_ball_inputs`) while a composer called them; those
/// were the seat's rungs and went with it (bl-7942), and what the ENGINE has
/// always taken is this struct.
fn inputs(
    m: &crate::AppModel,
    w: &super::World,
    workspace: &Path,
    payload: crate::start::Payload,
) -> crate::start::StartInputs {
    crate::start::StartInputs {
        workspace: workspace.to_path_buf(),
        payload,
        repo: None,
        home: w.roots.home.clone(),
        yog_data_root: w.roots.yog_data.clone(),
        balls_state_root: m.balls_state_root(),
        conversation_names: Vec::new(),
    }
}

/// Lay the world's seed marker so `prime` short-circuits (§16.6 W3) — the ordinary
/// case, and the one that keeps the fake `litany` to a single verb.
fn seed(yog_data_root: &Path) {
    let litany = crate::world::layout_under(yog_data_root).litany;
    fs::create_dir_all(&litany).unwrap();
    fs::write(litany.join("models.yaml"), b"models: {}\n").unwrap();
}

#[test]
fn a_raise_founds_the_wall_the_reply_names() {
    let bin = tempdir().unwrap();
    let w = world();
    seed(&w.roots.yog_data);
    let (_c, mut m) = model(&w);

    let target = crate::binding::workspace_path(&w.roots.yog_data, "ops");
    let inputs = inputs(&m, &w, &target, crate::start::Payload::Bare);
    let prepared = staged(&mut m, &fake_litany(bin.path(), &news()), &inputs).unwrap();

    assert_eq!(
        prepared.workspace, "ops",
        "the operator's own name (§3.1) — the address and the stamp are one"
    );
    // The wall the raise just made: a name resolves against the *published*
    // set, and this snapshot predates the `litany new` that founded it.
    let raised = crate::binding::workspace_path(&w.roots.yog_data, &prepared.workspace);
    assert_ne!(raised, w.ws_cobalt, "the raise always raises a fresh wall");
    assert!(raised.join("repo.git").is_dir(), "`litany new` ran");
    // The reply names the wall by its §3.1 NAME, which is what a seat adopts
    // and what the next gesture addresses (bl-7407): the raise's whole product.
    assert_eq!(crate::naming::leaf(&raised), prepared.workspace);
}

#[test]
fn a_failed_prepare_answers_the_refusal_and_founds_nothing() {
    let bin = tempdir().unwrap();
    let w = world();
    seed(&w.roots.yog_data);
    let (_c, mut m) = model(&w);

    let raised = crate::binding::workspace_path(&w.roots.yog_data, "ops");
    let inputs = inputs(&m, &w, &raised, crate::start::Payload::Bare);
    let err = staged(&mut m, &fake_litany(bin.path(), FAILS), &inputs).unwrap_err();

    assert!(err.contains("boom"), "the refusal rode back");
    assert!(
        !raised.join("repo.git").is_dir(),
        "nothing was founded, so there is nothing to adopt"
    );
}

/// The Prepare arm resolves the payload's project **name** (REMOTE §8,
/// bl-f5f6), and a name this snapshot enumerates no project for is a refusal
/// saying so, before a `bl` runs. One resolution, so every caller fails the
/// same way on the same token.
#[test]
fn a_ball_rung_whose_project_this_world_does_not_enumerate_refuses_by_name() {
    let bin = tempdir().unwrap();
    let w = world();
    seed(&w.roots.yog_data);
    let (_c, mut m) = model(&w);

    let inputs = inputs(
        &m,
        &w,
        &crate::binding::workspace_path(&w.roots.yog_data, "ops"),
        crate::start::Payload::Ball {
            project: "/nowhere/at/all".to_owned(),
            ball: crate::start::BallSpec::New {
                title: "Fresh".to_owned(),
                body: "do it".to_owned(),
            },
        },
    );
    let err = staged(&mut m, &fake_litany(bin.path(), &news()), &inputs).unwrap_err();

    assert_eq!(err, "unknown project \"/nowhere/at/all\" — known: a");
}
