//! Shared story-test fixture (DESIGN §15 M6 Z7, STORIES "Test harness"): fake
//! substrate **recorder** binaries + their read-back parser and a
//! workspace-on-disk builder for the pure-derivation stories.
//!
//! The recorder is the `editor_roundtrip` idiom generalized: a script
//! that appends argv+env+cwd to a NUL-delimited log and plays a canned
//! stdout/exit per verb, injected at the dispatch API as `Cli::new(path)` /
//! `Deps{…}`. It mutates **no** process-global env — the runner is parallel and
//! `std::env::set_var` is `unsafe` in edition 2024 — so what a child observes is
//! whatever the injected `Cli` stands on it (the `*_BINARY` resolution vars stay
//! production wiring, covered by the `resolve_with` unit tests).

// Not every module of this binary uses every fixture, and the three standalone
// tests at `tests/` use none of it — so an item can be live yet unused from
// here. The second allow covers the `recorder` split's `pub use`.
#![allow(dead_code)]
#![allow(unused_imports)]
// clippy's allow-*-in-tests reaches `#[test]` fns, not the free fixture helpers
// of an integration-test crate — those unwrap freely like any test (the
// `editor_roundtrip` precedent).
#![allow(clippy::unwrap_used)]

// The executable-fixture writer, shared with the standalone `tests/*.rs`
// binaries that write one too (bl-fd28). Declared inside `support` rather than
// beside it so it inherits this module's test-code lint relaxations.
#[path = "../../support/write_exec.rs"]
pub mod write_exec;

use std::fs;
use std::io;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use std::collections::HashMap;

use yog::projects::balls::{Ball, parse_list};
use yog::projects::runner::BlRunner;

/// A fake [`BlRunner`] for `AppModel`-construction stories: canned bedrock JSON
/// per project keyed by the decoded project path (parsed through the same
/// forgiving reader the closed listing uses), no store on disk — unlike the real
/// `BlStore`, which loads balls' typed catalog from the nested clone.
///
/// Setting [`delivered`](FakeBl::delivered) empties the **live** answer, which
/// is the disk change `bl close` makes — the ball leaves the live set, and the
/// dead set it was already in stops being shadowed (`join` treats live as
/// authoritative). One model can therefore be driven across a delivery
/// (STORIES S3-T7) instead of two models being compared across two fixtures.
#[derive(Default)]
pub struct FakeBl {
    pub live: HashMap<PathBuf, String>,
    pub closed: HashMap<PathBuf, String>,
    pub delivered: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl BlRunner for FakeBl {
    fn live(&self, project: &Path) -> io::Result<Vec<Ball>> {
        if self.delivered.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(Vec::new());
        }
        Ok(parse_list(
            self.live.get(project).map_or("[]", String::as_str),
        ))
    }
    fn closed(&self, project: &Path) -> io::Result<Vec<Ball>> {
        Ok(parse_list(
            self.closed.get(project).map_or("[]", String::as_str),
        ))
    }
    fn detail(&self, _project: &Path, _id: &str) -> Option<Ball> {
        None
    }
}

/// The fake substrate recorder binary + its read-back parser (STORIES "Test
/// harness"), split out per the 300-line cap.
mod recorder;
pub use recorder::*;

/// Multi-agent workspace fixtures for the S4/S6/S7 rows, split out per the cap.
mod world;
pub use world::*;

/// The on-disk payload writers the fixtures compose, split out per the cap.
mod payload;
pub use payload::*;

/// The harness's own hand-driven [`yog::ui_state::Clock`] (bl-9006), so a beat
/// about elapsed time does not measure the machine it runs on.
mod clock;
pub use clock::*;

/// The §11 reads — every `boundary::Query` a story puts to the engine, folded
/// the way the seat folds it. Everything else in this file WRITES the world a
/// story runs over; those only read it back, which is the seam.
mod asked;
pub use asked::*;

fn run_git(repo: &Path, args: &[&str]) {
    // Scrubbed (`yog::git_env`): inherited from the cargo-test process, `GIT_DIR`
    // and friends would redirect these fixture commits onto the outer repo.
    let mut cmd = yog::git_env::git();
    // Fixture git reads no machine config: the ambient global config can carry
    // a `core.hooksPath` whose commit-msg hook refuses the fixture identity
    // (the multiplex tests scrub the same way).
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null");
    cmd.env("GIT_CONFIG_SYSTEM", "/dev/null");
    let status = cmd.arg("-C").arg(repo).args(args).status().unwrap();
    assert!(status.success(), "git {args:?}");
}

/// Build a minimal litany workspace at `ws` (ARCH §2.2 layout): a bare
/// `repo.git` with a `config/default` commit and one `agents/c-001` dispatch
/// commit — enough for `GitTree::from_repo` to derive a single agent, so the
/// derived roster is non-empty.
pub fn build_workspace(ws: &Path) {
    let repo = ws.join("repo.git");
    fs::create_dir_all(&repo).unwrap();
    run_git(&repo, &["init", "-q", "--bare", "-b", "config/default"]);
    run_git(&repo, &["config", "user.email", "t@t.local"]);
    run_git(&repo, &["config", "user.name", "Tester"]);
    run_git(&repo, &["config", "commit.gpgsign", "false"]);
    // The orphan config-root commit (§2.2), authored in a throwaway worktree.
    let author = ws.join(".author");
    let author_str = author.to_string_lossy().to_string();
    run_git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "--orphan",
            "-b",
            "config/default",
            &author_str,
        ],
    );
    fs::write(author.join("version"), "1\n").unwrap();
    run_git(&author, &["add", "version"]);
    run_git(&author, &["commit", "-q", "-m", "config: init"]);
    run_git(&repo, &["worktree", "remove", &author_str]);
    // One agent branch with a dispatch commit; its worktree stays in place, the
    // shape `from_repo` derives an agent from.
    let wt = ws.join("agents").join("c-001");
    let wt_str = wt.to_string_lossy().to_string();
    run_git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "agents/c-001",
            &wt_str,
            "config/default",
        ],
    );
    fs::write(wt.join("goal.md"), "hello\n").unwrap();
    run_git(&wt, &["add", "goal.md"]);
    run_git(&wt, &["commit", "-q", "-m", "dispatch [c-001]"]);
}

/// **A wall the `Prompt` door lets a fire through, with nothing signed in**
/// (DESIGN §8.1's *sign in first*, bl-2291): a bare `repo.git` whose
/// `config/default` names brazen's built-in keyless `ollama` row in a role. The
/// door reads the lineage the fire forks off, and a keyless row a role names
/// readies the wall — the operator's own hand rather than the built-in merge.
/// For a beat about the fire rather than the wall; it touches no credential
/// store and no real world, which an integration test could not reach anyway.
pub fn keyless_wall(ws: &Path) {
    let repo = ws.join("repo.git");
    fs::create_dir_all(&repo).unwrap();
    run_git(&repo, &["init", "-q", "--bare", "-b", "config/default"]);
    run_git(&repo, &["config", "user.email", "t@t.local"]);
    run_git(&repo, &["config", "user.name", "Tester"]);
    run_git(&repo, &["config", "commit.gpgsign", "false"]);
    let author = ws.join(".author");
    let author_str = author.to_string_lossy().to_string();
    run_git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "--orphan",
            "-b",
            "config/default",
            &author_str,
        ],
    );
    fs::write(
        author.join("providers.yaml"),
        "roles:\n  worker:\n    provider: ollama\n    model: tiny\n",
    )
    .unwrap();
    run_git(&author, &["add", "providers.yaml"]);
    run_git(&author, &["commit", "-q", "-m", "config: keyless role"]);
    run_git(&repo, &["worktree", "remove", &author_str]);
}
