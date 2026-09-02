//! The writer, against real trees. Every assertion reads the disk back through
//! the **production** derivations — `binding`, `git_tree`, `steps_view`,
//! `registry` — because a fixture that only satisfies its own writer is a
//! fixture that proves nothing about what an engine will serve.

use super::*;
use crate::fixture::roster;
use tempfile::TempDir;

/// Lay one named state under a fresh root.
fn laid(state: &str, origin: i64) -> (TempDir, Places, Vec<PathBuf>) {
    let tmp = TempDir::new().expect("tmp");
    let recipe = roster::resolve(state).expect("state");
    let hold = lay(tmp.path(), recipe, origin).expect("lay");
    let places = Places::under(tmp.path());
    (tmp, places, hold)
}

/// The busy state, read back by the enumerator and the derivation an engine
/// runs: a **named** workspace (the kind that carries a claimant), and one
/// agent per conversation in the recipe.
#[test]
fn a_laid_state_enumerates_as_a_named_workspace_with_its_conversations() {
    let (_tmp, places, _) = laid("busy", 2_000_000_000);
    let found = crate::binding::workspaces(&places.data, &places.litany);
    assert_eq!(found.len(), 1, "one workspace");
    let ws = &found[0];
    assert_eq!(
        ws.kind,
        crate::binding::WorkspaceKind::Named {
            name: roster::WORKSPACE.to_owned()
        }
    );
    let tree = crate::git_tree::GitTree::from_repo(&ws.path).expect("derive");
    let recipe = roster::resolve("busy").expect("busy");
    assert_eq!(tree.agents.len(), recipe.workspaces[0].convs.len());
}

/// The §7.3 wound, both arms, read by the production predicate — the state's
/// whole reason to exist.
#[test]
fn the_wound_state_lays_a_spoken_wound_and_a_mute_one() {
    let (_tmp, places, _) = laid("wound", 2_000_000_000);
    let ws = places.workspace(roster::WORKSPACE);
    let spoke = crate::steps_view::build(&ws, "c-101", crate::git_tree::AgentState::Stopped);
    let mute = crate::steps_view::build(&ws, "c-102", crate::git_tree::AgentState::Stopped);
    assert!(matches!(
        crate::steps_view::latest_wound(&spoke),
        crate::steps_view::Wound::Spoke(_)
    ));
    assert!(matches!(
        crate::steps_view::latest_wound(&mute),
        crate::steps_view::Wound::Mute
    ));
    // And neither is an orphan: the states are distinct on purpose, so a
    // harness comparing one banner is never shown the other's.
    assert!(!spoke.orphan.orphaned());
    assert!(!mute.orphan.orphaned());
}

/// The orphaned tail, both shapes.
#[test]
fn the_orphan_state_lays_both_tails() {
    let (_tmp, places, _) = laid("orphan", 2_000_000_000);
    let ws = places.workspace(roster::WORKSPACE);
    let mail = crate::steps_view::build(&ws, "c-201", crate::git_tree::AgentState::Stopped);
    let window = crate::steps_view::build(&ws, "c-202", crate::git_tree::AgentState::Stopped);
    assert!(
        mail.orphan
            .banner()
            .contains(crate::steps_view::ORPHANED_MAIL)
    );
    assert!(
        window
            .orphan
            .banner()
            .contains(crate::steps_view::ORPHANED_WINDOW)
    );
    // The mail arm carries its driver's own words; the window arm has none.
    assert!(mail.orphan.banner().contains("driver.log"));
}

/// Every leaf a harness might present is registered in every workspace a state
/// lays. An admitted certificate registered nowhere sees an empty world
/// (REMOTE §4), which is the one way a fixture can look laid and be useless.
#[test]
fn every_client_leaf_is_registered_in_every_workspace() {
    let (_tmp, places, _) = laid("busy", 2_000_000_000);
    for role in LEAVES {
        if role == Role::Server {
            continue;
        }
        let client = registry::Client::parse(&role.common_name()).expect("name");
        let seen = registry::registered(&places.state, &client);
        assert!(seen.contains(roster::WORKSPACE), "{}", role.leaf());
    }
}

/// The settings state's two destinations, at the paths the production folds
/// resolve.
#[test]
fn the_settings_state_lays_a_cadence_and_a_wall() {
    let (_tmp, places, _) = laid("settings", 2_000_000_000);
    let cadence = places.state.join(crate::app::cadence::CADENCE_YAML);
    assert!(cadence.is_file(), "{}", cadence.display());
    let wall = places
        .walls
        .join(roster::WORKSPACE)
        .join("brazen")
        .join("config.toml");
    assert!(wall.is_file(), "{}", wall.display());
    // …and a state that asks for neither lays neither.
    let (_bare, bare, _) = laid("busy", 2_000_000_000);
    assert!(!bare.state.join(crate::app::cadence::CADENCE_YAML).exists());
    assert!(!bare.walls.exists());
}

/// The empty state lays the seed marker and no workspace — a first run, not a
/// missing world.
#[test]
fn the_empty_state_is_seeded_and_has_no_workspace() {
    let (_tmp, places, hold) = laid("empty", 2_000_000_000);
    assert!(places.litany.join("models.yaml").is_file());
    assert!(crate::binding::workspaces(&places.data, &places.litany).is_empty());
    assert!(hold.is_empty());
}

/// The `hold` list is exactly the two fds a `Streaming` step needs, and only
/// states carrying one produce any.
#[test]
fn a_streaming_step_asks_for_the_two_fds_that_make_it_live() {
    let (_tmp, _places, hold) = laid("busy", 2_000_000_000);
    assert_eq!(hold.len(), 2, "one inbox dir and one response file");
    assert!(hold[0].is_dir(), "the executor lock is a directory");
    assert!(hold[1].is_file(), "the writer fd is the response file");
    assert!(laid("settings", 2_000_000_000).2.is_empty());
}

/// **Determinism.** Two lays of one state at one origin produce the same tree,
/// byte for byte — and the trunk commit is identical even across origins,
/// because its date is fixed rather than stamped from the clock.
#[test]
fn two_lays_of_one_state_are_the_same_tree() {
    let (_a, first, _) = laid("transcript", 2_000_000_000);
    let (_b, second, _) = laid("transcript", 2_000_000_000);
    let (_c, later, _) = laid("transcript", 2_000_000_777);
    let read = |p: &Places, tail: &str| {
        std::fs::read(p.workspace(roster::WORKSPACE).join(tail)).expect("read")
    };
    for tail in ["agents/c-301/goal.md", "steps/c-301/000/response.json"] {
        assert_eq!(read(&first, tail), read(&second, tail), "{tail}");
    }
    let trunk = |p: &Places| {
        let repo = p.workspace(roster::WORKSPACE).join("repo.git");
        std::fs::read(repo.join("refs/heads/config/default")).expect("trunk ref")
    };
    assert_eq!(trunk(&first), trunk(&second));
    assert_eq!(trunk(&first), trunk(&later), "the trunk date is fixed");
}

/// A conversation's dispatch commit is dated from the recipe, so the roster's
/// one sort key is the state's choice and not the laying machine's clock.
#[test]
fn a_conversations_last_action_is_the_recipes_offset() {
    let origin = 2_000_000_000;
    let (_tmp, places, _) = laid("busy", origin);
    let ws = places.workspace(roster::WORKSPACE);
    let tree = crate::git_tree::GitTree::from_repo(&ws).expect("derive");
    let recipe = roster::resolve("busy").expect("busy");
    for conv in recipe.workspaces[0].convs {
        let agent = tree
            .agents
            .iter()
            .find(|a| a.agent_id == conv.id)
            .unwrap_or_else(|| panic!("{}", conv.id));
        assert_eq!(
            agent.last_action_unix,
            origin - conv.age_secs,
            "{}",
            conv.id
        );
    }
}

/// The `meta.json` a settled step carries, at the recipe's own instant.
#[test]
fn a_settled_step_carries_the_meta_a_returned_call_writes() {
    assert!(meta(0).contains("1970-01-01"));
    let (_tmp, places, _) = laid("settings", 2_000_000_000);
    let path = places
        .workspace(roster::WORKSPACE)
        .join("steps/c-401/000/meta.json");
    assert!(path.is_file());
}

/// The whole lay refuses when its root cannot even be made — the first
/// primitive's failure is the lay's, with nothing half-written behind it.
#[test]
fn a_lay_into_an_unmakeable_root_refuses() {
    let tmp = TempDir::new().expect("tmp");
    let file = tmp.path().join("f");
    std::fs::write(&file, "x").expect("write");
    let recipe = roster::resolve("busy").expect("busy");
    assert!(lay(&file.join("under"), recipe, 2_000_000_000).is_err());
}

/// A registry that cannot be written refuses the lay rather than laying a
/// workspace no client can see.
#[test]
fn an_unwritable_registry_refuses_the_lay() {
    let tmp = TempDir::new().expect("tmp");
    let places = Places::under(tmp.path());
    mkdir(&places.state).expect("state");
    std::fs::write(places.state.join(registry::CLIENTS), "not a dir").expect("write");
    let recipe = roster::resolve("busy").expect("busy");
    assert!(
        lay(tmp.path(), recipe, 2_000_000_000)
            .expect_err("refuse")
            .contains("register")
    );
}
