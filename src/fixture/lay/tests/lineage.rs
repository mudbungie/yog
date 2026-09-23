//! **The trunk a fixture founds, driven by the §9.4 surfaces** (bl-59cf).
//!
//! [`found`](super::super::found) used to commit `version` alone, so every
//! laid world's `config/default` declared no role at all: `/roles` answered an
//! empty table and `/effort` and `/priority` refused with the grammar's *no
//! such entry*. That is a refusal no harness can clear from outside — the
//! lineage is inside the world the verb just wiped and rebuilt — so no seat
//! lane could drive the tuning pair end to end against a fixture, which is how
//! it was found (yog-android bl-0691's drive of `yog fixture busy`).
//!
//! Driven through the boundary's own chokepoint and not through the grammar,
//! because the refusal this dissolves was one the *boundary* made: the read is
//! [`answer`], the writes are [`dispatch`], and the only stand-in is a
//! recorder `litany` catching what the §9.3 commit staged.

use super::*;
use crate::boundary::answer::answer;
use crate::boundary::config::{Read, Write};
use crate::boundary::dispatch::{Caller, Deps, dispatch};
use crate::boundary::reply::{ConfigAnswer, Reply};
use crate::boundary::{Action, Query};
use crate::cli_outbound::Cli;
use crate::model_pick::{Effort, Tuning};
use crate::ui_state::UiState;
use std::sync::Arc;

/// The `Deps` an engine booted on a laid root carries: the §16.2 world
/// composed off that root — asserted equal to the writer's own state place, so
/// this fixture cannot drift into reading where nothing was laid — and the
/// laid workspace published by name, which is what REMOTE §8's address table
/// resolves a gesture's sphere against.
fn deps_on(root: &Path, litany: &Path) -> Deps {
    let places = Places::under(root);
    let world = crate::world::compose(&crate::xdg::Env::from_pairs([(
        "XDG_DATA_HOME",
        root.display().to_string(),
    )]));
    assert_eq!(
        world.yog_state_root(),
        places.state,
        "the engine's own fold"
    );
    Deps {
        litany: Cli::new(litany),
        bl: Cli::new(Path::new("/definitely/not/a/bl-xyz")),
        state_root: places.state.clone(),
        yog_binary: root.join("yog"),
        world,
        home: root.join("home"),
        yog_data_root: places.data.clone(),
        snapshot: Arc::new(crate::boundary::tests::snapshot(
            &places.workspace(roster::WORKSPACE),
            roster::WORKSPACE,
            vec![],
            vec![],
        )),
        caller: Caller::default(),
    }
}

/// A `litany` that commits nothing and keeps what it was handed — the §9.3
/// staged `providers.yaml`, which is the byte-exact text the gesture wrote.
fn recorder(bin: &Path, log: &Path) -> PathBuf {
    let path = bin.join("litany");
    crate::test_support::write_exec(
        &path,
        &format!(
            "#!/bin/sh\ncat \"$YOG_EDIT_SRC/providers.yaml\" > {}\nexit 0\n",
            log.display()
        ),
    );
    path
}

fn tune(tuning: Tuning) -> Action {
    Action::Config(Write::Tune(tuning))
}

fn fire(deps: &Deps, action: &Action) -> Result<Reply, String> {
    let mut ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    dispatch(deps, &mut ui, "T0", action)
}

/// **The read answers, in every state that lays a workspace.** The roster is
/// swept rather than sampled: a recipe is founded by one writer, so a state
/// whose trunk declared nothing would be a world a lane silently cannot tune.
#[test]
fn every_laid_world_declares_its_roles_to_the_config_read() {
    let mut swept = 0;
    for (state, recipe) in roster::ROSTER {
        if recipe.workspaces.is_empty() {
            continue;
        }
        let (tmp, _places, _) = laid(state, 2_000_000_000);
        let deps = deps_on(tmp.path(), Path::new("/definitely/not/a/litany-xyz"));
        let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
        let rows = match answer(
            &Query::Config(Read::Roles {
                workspace: roster::WORKSPACE.to_owned(),
            }),
            &deps,
            &ui,
            0,
        ) {
            Ok(Reply::Config(ConfigAnswer::Roles(rows))) => rows,
            other => panic!("{state}: roles answers roles: {other:?}"),
        };
        assert_eq!(rows.len(), 2, "{state}");
        assert_eq!(rows[0].role, "worker", "{state}");
        assert!(!rows[0].provider.is_empty(), "{state}: a row is named");
        assert!(!rows[0].model.is_empty(), "{state}: a model is bound");
        // Nothing tuned yet — the knobs are what the gestures below write.
        assert_eq!(rows[0].effort, None, "{state}");
        assert!(!rows[0].priority, "{state}");
        swept += 1;
    }
    assert!(swept > 0, "the roster laid no workspace at all");
}

/// **The writes take.** Both knobs, through the same chokepoint a seat's
/// `/effort` and `/priority` reach, against a world `yog fixture` laid — and a
/// role the trunk does not declare still refuses, so the pair above is the
/// gesture working rather than the gesture unable to fail.
#[test]
fn the_tuning_pair_takes_against_a_laid_world() {
    let (tmp, _places, _) = laid("busy", 2_000_000_000);
    let bin = TempDir::new().expect("bin");
    let log = bin.path().join("staged");
    let deps = deps_on(tmp.path(), &recorder(bin.path(), &log));
    let ws = roster::WORKSPACE.to_owned();
    let knobs = [
        (
            Tuning::Effort {
                workspace: ws.clone(),
                role: "worker".to_owned(),
                level: Some(Effort::High),
            },
            "effort: high",
        ),
        (
            Tuning::Priority {
                workspace: ws.clone(),
                role: "worker".to_owned(),
                on: true,
            },
            "priority: true",
        ),
    ];
    for (tuning, line) in knobs {
        let reply = fire(&deps, &tune(tuning));
        assert!(
            matches!(&reply, Ok(Reply::Outcome(o)) if o.ok()),
            "{line}: {reply:?}"
        );
        let staged = std::fs::read_to_string(&log).expect("staged");
        assert!(staged.contains(line), "{line} missing from {staged}");
    }
    let refused = fire(
        &deps,
        &tune(Tuning::Effort {
            workspace: ws,
            role: "nobody".to_owned(),
            level: Some(Effort::Low),
        }),
    );
    assert!(
        matches!(&refused, Err(e) if e.contains("nobody")),
        "{refused:?}"
    );
}
