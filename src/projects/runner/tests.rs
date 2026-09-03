//! The typed read surface (§16.7 W8): [`BlStore`]'s in-process catalog load
//! against a real nested store on disk, the founded-clone gate that is the §3.5
//! orphan signal, and the one residual subprocess (the closed listing).

use super::*;
use crate::cli_outbound::Cli;
use crate::xdg::Env;
use tempfile::{TempDir, tempdir};

/// A hermetic balls state root with one clone of `/proj`, laid out the way balls
/// itself lays it out — `clones/<pct-enc-path>/{config/config,tasks/tasks}` —
/// through balls' OWN layout arithmetic, never a hand-spelled path.
struct World {
    _root: TempDir,
    xdg: Xdg,
    project: PathBuf,
}

impl World {
    /// A world whose clone of `/proj` is founded (landing present) unless
    /// `founded` is false — the unlistable/orphaned case.
    fn new(founded: bool) -> Self {
        let root = tempdir().unwrap();
        let env = Env::from_pairs([
            ("HOME", root.path().to_string_lossy().into_owned()),
            (
                "XDG_STATE_HOME",
                root.path().join("state").to_string_lossy().into_owned(),
            ),
        ]);
        let xdg = env.balls_layout();
        // A REAL dir: the residual `bl` subprocess runs cwd = the project (§5.1 #2).
        let project = root.path().join("proj");
        std::fs::create_dir_all(&project).unwrap();
        let clone = xdg.clone_dir(&project);
        std::fs::create_dir_all(clone.store().join("tasks")).unwrap();
        if founded {
            std::fs::create_dir_all(clone.landing().join("config")).unwrap();
        }
        Self {
            _root: root,
            xdg,
            project,
        }
    }

    /// Write `tasks/<id>.md` into the store checkout with the given frontmatter.
    fn ball(&self, id: &str, frontmatter: &str, body: &str) {
        let store = self.xdg.clone_dir(&self.project).store();
        std::fs::write(
            store.join("tasks").join(format!("{id}.md")),
            format!("+++\n{frontmatter}+++\n{body}"),
        )
        .unwrap();
    }

    /// A store over this world, its residual `bl` subprocess bound to `cli`.
    fn store(&self, cli: Cli) -> BlStore {
        BlStore::new(self.xdg.clone(), cli)
    }
}

/// Write an executable fake `bl`. Nothing brackets the write: the crate's one
/// fork (`crate::git_env`) owns the ETXTBSY exclusion.
fn write_bl(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("bl");
    crate::test_support::write_exec(&path, body);
    path
}

#[test]
fn live_loads_the_typed_catalog_in_process_with_no_spawn() {
    let world = World::new(true);
    // A claimed ball with every projected field, and a bare one that defaults.
    world.ball(
        "bl-1",
        "title = \"Work\"\ncreated = 10\nupdated = 20\nclaimant = \"cobalt\"\n\
         parent = \"bl-0\"\npriority = 3\nroot_commit = \"abc\"\ntags = [\"epic\"]\n\
         blockers = [{ id = \"bl-9\", on = \"claim\" }]\n",
        "the body\n",
    );
    world.ball("bl-2", "title = \"Bare\"\ncreated = 1\nupdated = 2\n", "");
    // The Cli names a binary that does not exist: a read that spawned would
    // fail, so passing proves the load is in-process.
    let store = world.store(Cli::new("/nonexistent/bl"));
    let balls = store.live(&world.project).unwrap();
    assert_eq!(balls.len(), 2);
    let one = &balls[0];
    assert_eq!(one.id, "bl-1");
    assert_eq!(one.title, "Work");
    assert_eq!(one.body, "the body\n");
    assert_eq!(one.claimant.as_deref(), Some("cobalt"));
    assert_eq!(one.parent.as_deref(), Some("bl-0"));
    assert_eq!(one.priority, 3);
    assert_eq!(one.root_commit.as_deref(), Some("abc"));
    assert_eq!(one.tags, ["epic"]);
    assert_eq!(one.created, Some(10));
    assert_eq!(one.updated, Some(20));
    assert_eq!(one.blockers.len(), 1);
    assert_eq!(one.blockers[0].id, "bl-9");
    assert_eq!(one.blockers[0].on, "claim");
    // Absent optionals default exactly as the JSON projection defaulted them.
    let two = &balls[1];
    assert_eq!(two.priority, 0);
    assert_eq!(two.claimant, None);
    assert_eq!(two.parent, None);
    assert_eq!(two.root_commit, None);
    assert!(two.blockers.is_empty() && two.tags.is_empty());
}

#[test]
fn an_empty_string_field_normalizes_to_none_like_the_json_projection() {
    let world = World::new(true);
    world.ball(
        "bl-1",
        "title = \"T\"\ncreated = 1\nupdated = 1\nclaimant = \"\"\nparent = \"\"\nroot_commit = \"\"\n",
        "",
    );
    let ball = world
        .store(Cli::new("bl"))
        .detail(&world.project, "bl-1")
        .unwrap();
    assert_eq!(
        (ball.claimant, ball.parent, ball.root_commit),
        (None, None, None)
    );
}

#[test]
fn an_unfounded_clone_is_unlistable_the_orphaned_project_signal() {
    // The clone dir exists (it is enumerated) but carries no landing: `live`
    // errors rather than reporting an empty store, so the §3.5 join renders the
    // project orphaned instead of clean-but-empty.
    let world = World::new(false);
    let store = world.store(Cli::new("bl"));
    assert!(store.live(&world.project).is_err());
    assert!(store.detail(&world.project, "bl-1").is_none());
}

#[test]
fn detail_resolves_one_live_ball_and_misses_on_an_unknown_id() {
    let world = World::new(true);
    world.ball("bl-1", "title = \"T\"\ncreated = 1\nupdated = 1\n", "");
    let store = world.store(Cli::new("bl"));
    assert_eq!(store.detail(&world.project, "bl-1").unwrap().title, "T");
    assert!(store.detail(&world.project, "bl-nope").is_none());
}

#[test]
fn closed_runs_the_one_residual_subprocess_and_parses_its_bedrock_json() {
    let world = World::new(true);
    let dir = tempdir().unwrap();
    // Noise on stderr (exercises the Stderr arm) + a bedrock array on stdout.
    let bin = write_bl(
        dir.path(),
        "#!/bin/sh\nprintf 'warn\\n' 1>&2\nprintf '[{\"id\":\"bl-old\"}]\\n'\n",
    );
    let closed = world.store(Cli::new(bin)).closed(&world.project).unwrap();
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].id, "bl-old");
}

#[test]
fn closed_errors_on_spawn_failure_and_on_a_nonzero_exit() {
    let world = World::new(true);
    let dir = tempdir().unwrap();
    {
        let missing = world.store(Cli::new(dir.path().join("no-such-bl")));
        assert!(missing.closed(&world.project).is_err(), "spawn failure");
    }
    let bin = write_bl(dir.path(), "#!/bin/sh\nprintf 'partial'\nexit 3\n");
    assert!(
        world.store(Cli::new(bin)).closed(&world.project).is_err(),
        "nonzero exit"
    );
}

#[test]
fn identity_prefers_recorded_then_user_then_empty() {
    assert_eq!(
        identity(Some("recorded".to_owned()), Some("user".to_owned())),
        "recorded"
    );
    assert_eq!(identity(None, Some("user".to_owned())), "user");
    assert_eq!(identity(None, None), "");
}
