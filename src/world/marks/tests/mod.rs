//! The §16.3 balls-space tests: the space fold (world vs. an agent's own), the
//! branch read/write round trip, the lawfulness guard, and the `ops.jsonl` row
//! every write leaves. No subprocess and no runner seam — since the
//! per-agent ruling the value's one home is the space's own balls config file,
//! so every path here is pure fold + one file.

use super::*;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// An ambient snapshot with the anchor set, plus a state home to prove the
/// world's space keeps §16.2's state exactly where it already is.
fn ambient() -> Env {
    Env::from_pairs([
        ("HOME", "/h"),
        ("XDG_DATA_HOME", "/d"),
        ("XDG_STATE_HOME", "/s"),
    ])
}

/// A real world rooted in a temp dir, so the file halves write somewhere.
fn world_at(dir: &TempDir) -> Env {
    crate::world::compose(&Env::from_pairs([
        ("HOME", dir.path().to_string_lossy().into_owned()),
        (
            "XDG_DATA_HOME",
            dir.path().join("data").to_string_lossy().into_owned(),
        ),
        (
            "XDG_STATE_HOME",
            dir.path().join("state").to_string_lossy().into_owned(),
        ),
    ]))
}

#[test]
fn no_marks_var_resolves_the_world_space_and_keeps_ss16_2_state_where_it_is() {
    let world = crate::world::compose(&ambient());
    let space = space(&world);
    // The state home is §16.2's, untouched — every clone yog already founded is
    // still the one it reads. Only the CONFIG home is new, and it nests.
    assert_eq!(space.state, PathBuf::from("/d/yog/world/state"));
    assert_eq!(space.config, PathBuf::from("/d/yog/world/config"));
    assert_eq!(
        config_file(&space.config),
        PathBuf::from("/d/yog/world/config/balls/config.toml")
    );
}

#[test]
fn an_unset_state_home_still_folds_balls_own_default() {
    // The world composes XDG_STATE_HOME, so the fallback is only reachable on a
    // bare ambient snapshot — which is exactly what a test seat hands in.
    let space = space(&Env::from_pairs([("HOME", "/h"), ("XDG_DATA_HOME", "/d")]));
    assert_eq!(space.state, PathBuf::from("/h/.local/state"));
}

#[test]
fn the_marks_var_resolves_one_root_serving_as_both_homes() {
    let world = crate::world::compose(&ambient()).with_overrides(&[(YOG_MARKS, "/agent/space")]);
    let space = space(&world);
    assert_eq!(space.state, PathBuf::from("/agent/space"));
    assert_eq!(space.config, PathBuf::from("/agent/space"));
}

#[test]
fn an_agents_own_root_is_its_wall_plus_marks_and_the_pairs_ride_the_wall_seam() {
    let world = crate::world::compose(&ambient());
    let ws = Path::new("/d/yog/workspaces/corp");
    assert_eq!(
        own_root(&world, ws),
        PathBuf::from("/d/yog/world/walls/corp/marks")
    );
    assert_eq!(
        pairs(&world, ws, true),
        vec![(
            YOG_MARKS.to_owned(),
            "/d/yog/world/walls/corp/marks".to_owned()
        )]
    );
    // A launch pointed at a project layers nothing: an absent var IS "the
    // board's own space", so there is no second value meaning the same thing.
    assert!(pairs(&world, ws, false).is_empty());
}

#[test]
fn a_space_with_nothing_written_reads_as_balls_own_default() {
    let dir = tempdir().unwrap();
    let world = world_at(&dir);
    let ws = dir.path().join("workspaces").join("home");
    assert_eq!(read(&world, &ws).branch(), SHARED_BRANCH);
}

#[test]
fn a_write_lands_in_the_spaces_own_balls_config_and_reads_back() {
    let dir = tempdir().unwrap();
    let world = world_at(&dir);
    let ws = dir.path().join("workspaces").join("home");
    let state = dir.path().join("ops");

    let landed = apply(&read(&world, &ws), &state, "T0", "balls/agents/home").unwrap();
    assert_eq!(landed, "balls/agents/home");

    let space = read(&world, &ws);
    let file = config_file(&space.config);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "tasks_branch = \"balls/agents/home\"\n"
    );
    // The file is balls' own §4 layer-2 config, at balls' own path under the
    // space root — the whole point being that an agent's `bl` resolves it
    // wherever that agent runs, which a per-clone landing could never do.
    assert!(file.starts_with(dir.path().join("data/yog/world/walls/home/marks")));
    assert_eq!(space.branch(), "balls/agents/home");

    // Re-pointing replaces rather than accumulating: one key, one home.
    assert_eq!(
        apply(&read(&world, &ws), &state, "T1", SHARED_BRANCH).unwrap(),
        SHARED_BRANCH
    );
    assert_eq!(read(&world, &ws).branch(), SHARED_BRANCH);
}

#[test]
fn every_write_leaves_one_ops_row_naming_the_file_and_the_branch() {
    let dir = tempdir().unwrap();
    let world = world_at(&dir);
    let ws = dir.path().join("workspaces").join("home");
    let state = dir.path().join("ops");
    apply(&read(&world, &ws), &state, "T0", "balls/mine").unwrap();

    let rows = crate::opslog::tail(&state, 10);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].argv,
        vec!["yog-step", "marks", "balls/mine"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
    assert_eq!(rows[0].exit, 0);
    assert_eq!(rows[0].origin, crate::opslog::Origin::World);
    assert!(rows[0].cwd.ends_with("balls/config.toml"));
}

#[test]
fn an_unlawful_branch_refuses_at_the_write_and_still_logs_the_attempt() {
    let dir = tempdir().unwrap();
    let world = world_at(&dir);
    let ws = dir.path().join("workspaces").join("home");
    let state = dir.path().join("ops");

    let err = apply(&read(&world, &ws), &state, "T0", "balls/config").unwrap_err();
    assert!(err.to_string().contains("landing branch"), "{err}");
    // Nothing was written, and the refusal is a durable row like every other
    // failed mutation (§4.2's -3 synthetic-failure exit).
    assert!(!config_file(&read(&world, &ws).config).exists());
    let rows = crate::opslog::tail(&state, 10);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].exit, -3);
}

#[test]
fn lawfulness_is_the_whole_guard_and_it_names_every_way_a_branch_can_fail() {
    assert!(lawful("balls/tasks"));
    assert!(lawful("balls/agents/corp"));
    assert!(!lawful(""));
    // balls itself refuses a store branch naming the landing ("one branch
    // cannot back two checkouts"), so this states the same invariant earlier.
    assert!(!lawful("balls/config"));
    assert!(!lawful("two words"));
    assert!(!lawful("with\ttab"));
    // Anything a TOML string would have to escape, so [`body`] can quote and
    // [`parse_branch`] can unquote with no parser between them.
    assert!(!lawful("say\"hi\""));
    assert!(!lawful("back\\slash"));
}

#[test]
fn a_config_body_yog_did_not_write_degrades_to_the_default_rather_than_guessing() {
    // balls surfaces a real parse error on every op it runs, so this layer stays
    // quiet: no key, an empty value, or a body that is not this shape at all is
    // "nothing written", which is the default.
    assert_eq!(parse_branch(""), None);
    assert_eq!(parse_branch("log_level = \"debug\"\n"), None);
    assert_eq!(parse_branch("tasks_branch = \"\"\n"), None);
    assert_eq!(parse_branch("tasks_branch\n"), None);
    assert_eq!(
        parse_branch("log_level = \"debug\"\ntasks_branch = \"balls/x\"\n"),
        Some("balls/x".to_owned())
    );
    // Unquoted is still read: the file is balls' schema, but a human editing it
    // by hand is reading a value, not a quoting convention.
    assert_eq!(
        parse_branch("tasks_branch = balls/y\n"),
        Some("balls/y".to_owned())
    );
}

#[test]
fn an_unwritable_space_refuses_and_says_so_rather_than_reporting_a_branch() {
    let dir = tempdir().unwrap();
    let world = world_at(&dir);
    let ws = dir.path().join("workspaces").join("home");
    let state = dir.path().join("ops");
    // A file where the space's directory must be: `create_dir_all` fails, which
    // is the one hard error the write can hit.
    let root = own_root(&world, &ws);
    std::fs::create_dir_all(root.parent().unwrap()).unwrap();
    std::fs::write(&root, "not a dir").unwrap();

    assert!(apply(&read(&world, &ws), &state, "T0", "balls/x").is_err());
    assert_eq!(crate::opslog::tail(&state, 10)[0].exit, -3);
}
