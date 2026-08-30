//! §8.7 — the birth policy a ball's tags select, on both acts that consume it:
//! the §8.6 policy convergence during [`prepare`] and the fire's own
//! `--config`. Two tagged balls reach two observably different frozen birth
//! configurations, which is the claim §4.2 makes and this proves.

use super::prompt::{make_fifo, prepared, workspace};
use super::{World, fake_bl, write_exec};
use crate::binding::workspace_path;
use crate::cli_outbound::Cli;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Deps, Payload, execute_prompt, prepare};
use crate::test_support::workspace::{seed_workspace_config, seed_workspace_lineage};
use std::path::Path;

/// litany's shipped worker manifest, reduced — it composes no
/// `instructions/**`, so §3.7's glob always drifts and the convergence always
/// drives. That makes the `litany config <ws> <name>` argv the observable.
const MANIFEST: &str = "roles:\n  worker:\n    pinned:\n      - goal.md\n";

fn deps(w: &World, bl: &Cli, litany: &Cli) -> Deps {
    Deps {
        bl: bl.clone(),
        litany: litany.clone(),
        state_root: w.state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
    }
}

/// A ball rung carrying `tags`, already bound (so no `bl claim` runs).
fn tagged(project: &Path, tags: &[&str]) -> Payload {
    Payload::Ball {
        project: crate::naming::leaf(project),
        ball: BallSpec::Existing {
            id: "bl-1111".to_owned(),
            title: "T".to_owned(),
            body: "B".to_owned(),
            join: JoinState::Bound,
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
        },
    }
}

/// A world whose workspace already exists, carrying `config/default` plus a
/// lineage per name — the state an operator's `litany config` leaves behind.
fn seeded(w: &World, lineages: &[&str]) -> std::path::PathBuf {
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    seed_workspace_config(
        &ws,
        &[
            ("workflow.yaml", "events: {}\n"),
            ("manifest.yaml", MANIFEST),
        ],
    );
    for name in lineages {
        seed_workspace_lineage(&ws, name);
    }
    let litany = crate::world::layout_under(w.yog.path()).litany;
    std::fs::create_dir_all(&litany).unwrap();
    std::fs::write(litany.join("models.yaml"), b"models: {}\n").unwrap();
    ws
}

/// The `litany config` rows this prepare drove, by the lineage they targeted
/// (`argv[3]` — `litany config <ws> <name>`).
fn converged(w: &World) -> Vec<String> {
    w.ops()
        .into_iter()
        .filter(|e| e.argv.get(1).map(String::as_str) == Some("config"))
        .filter_map(|e| e.argv.get(3).cloned())
        .collect()
}

/// The whole claim, in one table: two balls, two tags, two lineages — and the
/// **same** two answers on both consumers of the fact. A third ball whose tag
/// names no lineage is the default, which is the untagged path with a
/// different input rather than a case of its own.
#[test]
fn two_tagged_balls_are_born_on_two_different_lineages() {
    for (tags, want) in [
        (vec!["deep"], Some("deep")),
        (vec!["quick"], Some("quick")),
        (vec!["unmapped"], None),
        (vec![], None),
    ] {
        let w = World::new();
        let ws = seeded(&w, &["deep", "quick"]);
        let bl = Cli::new("/no/bl"); // a bound ball claims nothing
        let litany = w.litany();
        let mut inputs = w.inputs("cobalt-gecko", tagged(w.project.path(), &tags));
        inputs.workspace = ws.clone();
        let p = prepare(&deps(&w, &bl, &litany), &inputs, "TS").unwrap();
        assert_eq!(p.lineage.as_deref(), want, "tags {tags:?}");
        // The §8.6 convergence authored onto the *same* branch the fire will
        // fork off — never `default` while the drone is born elsewhere.
        assert_eq!(converged(&w), vec![want.unwrap_or("default").to_owned()]);
    }
}

/// The fire's half: the selected lineage rides `--config` ahead of the binding,
/// and a start that selected none spells no flag at all — litany's own
/// `config/default` is the default, not a word yog writes.
#[test]
fn the_fire_carries_the_selected_lineage_and_omits_an_unselected_one() {
    for (lineage, want) in [
        (Some("deep".to_owned()), vec!["--config", "deep"]),
        (None, vec![]),
    ] {
        let w = World::new();
        let fifo = w.bin.path().join("report");
        make_fifo(&fifo);
        let body = format!(
            "#!/bin/sh\nprintf '%s\\037%s' \"$4\" \"$5\" > '{}'\n",
            fifo.display()
        );
        let litany = Cli::new(write_exec(w.bin.path(), "litany", &body));
        let ws = workspace(&w);
        execute_prompt(
            &litany,
            w.state.path(),
            "TS",
            &crate::start::Fire {
                workspace: ws.clone(),
                prepared: crate::start::Prepared {
                    lineage,
                    ..prepared("cobalt-gecko", None)
                },
                goal: "do it".to_owned(),
            },
            &[],
            &super::rng(),
        )
        .unwrap();
        // argv is `prompt --name <conv> [--config <name>] <ws> <goal>`, so the
        // logged row says it too — one list, spawned and logged (§3.3).
        let logged = &w.ops()[0].argv;
        assert_eq!(logged[4..4 + want.len()], want[..], "lineage {want:?}");
        assert_eq!(
            logged[logged.len() - 2..],
            [ws.to_string_lossy().into_owned(), "do it".to_owned()],
            "the workspace and the goal still trail",
        );
        // And the fake saw the same two words the row records.
        let recorded = std::fs::read_to_string(&fifo).unwrap();
        let seen: Vec<&str> = recorded.split('\u{1f}').collect();
        let expect = if want.is_empty() {
            vec![ws.to_string_lossy().into_owned(), "do it".to_owned()]
        } else {
            want.iter().map(|s| (*s).to_owned()).collect()
        };
        assert_eq!(seen, expect);
    }
}

/// §3.7's filename policy follows the fire, not `config/default`: a lineage
/// declaring its own `instructions.yaml` freezes its own filenames, so one
/// lineage's answer can never compose another lineage's files.
#[test]
fn the_instruction_filename_policy_is_read_off_the_fired_lineage() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    seed_workspace_config(&ws, &[("instructions.yaml", "- HOUSE.md\n")]);
    seed_workspace_lineage(&ws, "deep");
    assert_eq!(
        crate::start::instructions::names::names(&ws, "deep"),
        vec!["HOUSE.md".to_owned()],
        "the fork carries the policy it was forked with",
    );
    // A lineage that never existed reads nothing, so the shipped default stands.
    assert_eq!(
        crate::start::instructions::names::names(&ws, "absent"),
        vec!["AGENTS.md".to_owned()],
    );
}

/// A ball `bl create` just minted has no tags, so the re-planned start is the
/// default — the general path with an empty input, not a bootstrap case.
#[test]
fn a_freshly_minted_ball_selects_the_default_lineage() {
    let w = World::new();
    let ws = seeded(&w, &["deep"]);
    let wt = crate::binding::work_worktree_path(w.balls.path(), w.project.path(), "bl-2222", None);
    let bl = Cli::new(fake_bl(w.bin.path(), "bl-2222", &wt));
    let litany = w.litany();
    let mut inputs = w.inputs(
        "cobalt-gecko",
        Payload::Ball {
            project: crate::naming::leaf(w.project.path()),
            ball: BallSpec::New {
                title: "T".to_owned(),
                body: "B".to_owned(),
            },
        },
    );
    inputs.workspace = ws;
    let p = prepare(&deps(&w, &bl, &litany), &inputs, "TS").unwrap();
    assert_eq!(p.lineage, None);
}
