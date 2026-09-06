//! End-to-end drive of the W8/bl-2930 `bl` multiplex arm through
//! `yog::multiplex::dispatch` — `yog bl <argv…>` IS balls, over the FULL verb
//! surface: the W9 `prime`/`sync`/`install` refusal is deleted, so this file
//! proves the whole ball rung against a real store on disk — `prime` founds
//! the checkout and binds the plugin chain to the world's own shims, `create`
//! seals a ball, `claim` cuts the code worktree (spawning the bound
//! `bl-delivery`/`bl-tracker` shims as real subprocesses), and `close`
//! delivers the tagged squash to the project's `main`.
//!
//! **This test binary owns its process environment** (the
//! `tests/multiplex_litany.rs` precedent): the arm is a *binding* — it reads
//! `$XDG_STATE_HOME`/`$XDG_DATA_HOME`/`$HOME`/`$USER` live at the process
//! boundary, none injectable through `dispatch`'s argv surface — so this file
//! mutates its own env, lawful exactly here: every `tests/*.rs` is a separate
//! process and the one `#[test]` below is this binary's only test.
//!
//! **The plugin spawns re-enter the real yog binary.** In-process, the shims
//! `ensure_tools` writes would target *this test binary* (`current_exe`), so
//! the `BL_DELIVERY_BINARY`/`BL_TRACKER_BINARY` seams point at wrapper
//! scripts that exec `$CARGO_BIN_EXE_yog <namespace> "$@"` — the claim's
//! plugin chain then runs the built yog's own multiplex arms, which is the
//! production shape (a shim whose target is yog) with the target made
//! explicit.
#![allow(clippy::unwrap_used)]

// The executable-fixture writer, shared by every integration binary that
// writes one (bl-fd28). `#[path]` because this file IS the test target's
// crate root, and a second top-level `tests/*.rs` would be a second binary.
#[path = "support/write_exec.rs"]
mod write_exec;

use std::fs;
use std::path::Path;

use yog::multiplex::dispatch;

// The scaffolding seam (bl-ff85). `#[path]` because this file IS the test
// target's crate root, so a bare `mod` would resolve to `tests/fixtures.rs` —
// and a second top-level `tests/*.rs` is a second test binary, not a module.
#[path = "multiplex_bl/fixtures.rs"]
mod fixtures;
use fixtures::{IDENT, fixture_gitconfig, found_project, git, plugin_wrapper, sole_child};

/// The §16.3 space half of the same drive (bl-c21d), split at the 300-line cap:
/// the rung above runs in the directory's own store, this one in an agent's
/// own space over a world-owned directory, where the worktree must follow the
/// store.
#[path = "multiplex_bl/marks.rs"]
mod marks;

/// The board half (bl-262a): yog's own §5.1 #2 read, asked for the same
/// project, must answer out of the same store the arm above just wrote.
#[path = "multiplex_bl/board.rs"]
mod board;

// git vars a hook-invoked test may inherit. This binary scrubs them from its
// OWN env rather than a child's — the balls it drives runs in-process — but the
// list is `yog::git_env`'s, the one the spawn sites use.
use yog::git_env::INHERITED as INHERITED_GIT_ENV;

fn set(key: &str, value: &Path) {
    // SAFETY: single-threaded — this binary runs exactly one #[test], and no
    // other thread exists to read the env concurrently (module doc).
    unsafe { std::env::set_var(key, value) };
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

/// bl-52ed — a **discovery probe** (an argv that is exactly the flag) reads
/// balls' interface, never the world: the arm's shim converge is skipped, so
/// each probe answers 0 and `world` — the anchor's yog root — stays absent.
fn probes_answer_and_found_nothing(world: &Path) {
    for probe in ["--help", "-h", "--skill"] {
        assert_eq!(dispatch(&argv(&["yog", "bl", probe])), Some(0), "{probe}");
        assert!(!world.exists(), "{probe} founded {}", world.display());
    }
}

/// An unusable anchor is a failed converge: the shims are every verb's
/// precondition (a `prime` binds them as siblings), so a tools dir that cannot
/// exist reports and fails (1) before balls runs at all — while a probe, which
/// needs no shim, still answers 0 (bl-52ed: help must not depend on a world
/// being writable). Anchoring `$XDG_DATA_HOME` inside a regular file makes the
/// dir uncreatable (ENOTDIR) — the `tests/multiplex_litany.rs` idiom.
fn an_unusable_anchor_fails_every_verb_but_a_probe(tmp: &Path) {
    let blocked = tmp.join("not-a-dir");
    fs::write(&blocked, "").unwrap();
    set("XDG_DATA_HOME", &blocked.join("xdg-data"));
    assert_eq!(dispatch(&argv(&["yog", "bl", "--skill", "prime"])), Some(1));
    assert_eq!(dispatch(&argv(&["yog", "bl", "--help"])), Some(0));
}

/// bl-262a — **the rung runs in the store of the DIRECTORY it is invoked in.**
///
/// The invocation path here is the operator's own project (`found_project`
/// chdirs into it), which the world does not own — so §16.2's
/// one-store-per-project invariant resolves the HOST's bundle, the very store
/// the operator's own `bl` in that directory resolves. Returns it, the anchor
/// of every store and territory path below.
///
/// **This is the mirror image of what bl-81c9 asserted here, and deliberately.**
/// That ball's complaint stands and is unchanged — one command spelling must
/// not address two universes — but it is *satisfied* by this, not violated: for
/// this directory `yog bl` and a bare `bl` now name the same store, where
/// before they named two. What bl-81c9 fixed is the world FOLD, still whole:
/// the arm stands the process in the world for `LITANY_HOME`, `PATH` and
/// balls' config home before balls reads a byte, and the world's own balls
/// state still serves every directory the world owns (`marks::…`, which drives
/// exactly that). What this proves is the half the fold cannot decide: which
/// store a directory has.
fn the_directorys_own_store(tmp: &Path) -> std::path::PathBuf {
    let world_balls = tmp.join("data/yog/world/state/balls/clones");
    assert!(
        !world_balls.exists(),
        "a world clone was founded for an operator's own directory: {}",
        world_balls.display()
    );
    tmp.join("state/balls")
}

#[test]
fn the_bl_arm_runs_the_whole_rung_on_the_embedded_balls() {
    let tmp = tempfile::TempDir::new().unwrap();
    for var in INHERITED_GIT_ENV {
        // SAFETY: as `set` — one test, one thread.
        unsafe { std::env::remove_var(var) };
    }
    for var in [
        "BL_BINARY",
        "LITANY_BINARY",
        "BZ_BINARY",
        "YOG_NAME",
        "BALLS_CLOCK",
        // The six balls lets cross the delivery boundary as author input:
        // exporting one satisfies the commit for the wrong reason, hiding a
        // repository that carries no identity of its own (bl-ff85).
        "GIT_AUTHOR_NAME",
        "GIT_AUTHOR_EMAIL",
        "GIT_AUTHOR_DATE",
        "GIT_COMMITTER_NAME",
        "GIT_COMMITTER_EMAIL",
        "GIT_COMMITTER_DATE",
    ] {
        // SAFETY: as `set` — the arm must run the default (self-multiplex)
        // resolution, not a machine-local override or identity.
        unsafe { std::env::remove_var(var) };
    }
    let bin = tmp.path().join("bin");
    fs::create_dir(&bin).unwrap();
    set("BL_DELIVERY_BINARY", &plugin_wrapper(&bin, "bl-delivery"));
    set("BL_TRACKER_BINARY", &plugin_wrapper(&bin, "bl-tracker"));
    set("HOME", &tmp.path().join("home"));
    set("XDG_DATA_HOME", &tmp.path().join("data"));
    set("XDG_STATE_HOME", &tmp.path().join("state"));
    set("USER", Path::new("rung-actor"));
    set("GIT_CONFIG_GLOBAL", &fixture_gitconfig(tmp.path()));
    set("GIT_CONFIG_SYSTEM", Path::new("/dev/null"));

    // bl-52ed — **help reads the interface, never the world**, and this is the
    // first thing the binary does because every verb below materializes the
    // world it must not have touched. Before this, `yog bl --help` wrote six
    // shims under the fresh anchor — and on a read-only root failed with yog's
    // converge error *instead of* printing help.
    probes_answer_and_found_nothing(&tmp.path().join("data").join("yog"));

    let proj = found_project(tmp.path());

    // The W12 slice/exit plumbing the unit tests can no longer drive without
    // writing shims under the ambient anchor: balls' own exits ride back.
    assert_eq!(dispatch(&argv(&["yog", "bl", "--skill"])), Some(0));
    assert_eq!(dispatch(&argv(&["yog", "bl"])), Some(2));
    assert_eq!(dispatch(&argv(&["yog", "bl", "no-such-verb"])), Some(2));
    // bl-2930: the former W9 refusals are balls' own verbs — their `--skill`
    // docs print (exit 0), no yog guard interposed.
    for verb in ["prime", "sync", "install"] {
        assert_eq!(dispatch(&argv(&["yog", "bl", "--skill", verb])), Some(0));
    }

    // `prime` — founds the checkout AND binds the plugin chain: the arm hands
    // balls `world/tools/bl` as the running executable, so the seed's sibling
    // rule finds the `bl-delivery`/`bl-tracker` shims beside it.
    assert_eq!(
        dispatch(&argv(&["yog", "bl", "prime", "--as", "seam"])),
        Some(0)
    );
    let tools = tmp.path().join("data/yog/world/tools");
    for shim in ["bl", "litany", "bz", "bl-delivery", "bl-tracker"] {
        assert!(tools.join(shim).is_file(), "shim {shim} converged");
    }
    let balls_state = the_directorys_own_store(tmp.path());
    let clone = sole_child(&balls_state.join("clones"));
    assert!(clone.join("config").is_dir(), "landing founded");
    for plugin in ["bl-delivery", "bl-tracker"] {
        // The landing checkout (`<clone>/config`) holds the config tree, so
        // the gitignored binding symlinks live at `config/plugins/bin/`.
        let bound = fs::read_link(clone.join("config/config/plugins/bin").join(plugin)).unwrap();
        assert_eq!(
            bound,
            tools.join(plugin),
            "{plugin} bound to the world shim"
        );
    }

    // `create` — one ball, sealed into the store checkout.
    assert_eq!(
        dispatch(&argv(&["yog", "bl", "create", "rung ball", "--as", "seam"])),
        Some(0)
    );
    let mut balls_md: Vec<_> = fs::read_dir(clone.join("tasks/tasks"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    assert_eq!(balls_md.len(), 1, "one sealed ball: {balls_md:?}");
    let task = balls_md.remove(0);
    let id = task.file_stem().unwrap().to_str().unwrap().to_string();

    board::the_board_reads_the_directorys_own_store(&proj, &id);
    // `claim` — cuts the code worktree through the bound bl-delivery shim (a
    // real subprocess chain: symlink → shim → yog's plugin arm).
    assert_eq!(
        dispatch(&argv(&["yog", "bl", "claim", &id, "--as", "seam"])),
        Some(0)
    );
    // The worktree lands in the SAME bundle as the store — the half an `Edge`
    // alone could never fix: `bl-delivery` is a real subprocess that folds
    // `$XDG_STATE_HOME` out of its OWN env (bl-81c9), so the arm stands the
    // process in whichever space the invocation directory resolves. Store and
    // territory are one fact, and since bl-262a that fact is the directory's.
    let territory = balls_state.join("plugins/bl-delivery");
    let worktree = territory.join(proj.strip_prefix("/").unwrap()).join(&id);
    assert!(
        worktree.join("README.md").is_file(),
        "worktree materialized at {}",
        worktree.display()
    );

    // `close` — delivers the worktree's diff as the tagged squash on `main`
    // and archives the ball (absence is the record).
    fs::write(worktree.join("work.txt"), "delivered\n").unwrap();
    assert_eq!(
        dispatch(&argv(&["yog", "bl", "close", &id, "--as", "seam"])),
        Some(0)
    );
    // The delivery tag, and the AUTHOR that signed the squash: the seeded
    // repository-local identity — the half of bl-ff85 a dev box can check,
    // where a populated passwd entry would still let git guess one.
    let (name, email) = IDENT;
    let head = git(&proj, &["log", "-1", "--format=%s%n%an <%ae>", "main"]);
    assert!(head.contains(&format!("[{id}]")), "delivery tag: {head}");
    assert!(
        head.contains(&format!("{name} <{email}>")),
        "author: {head}"
    );
    assert!(!task.exists(), "closed ball's file is gone");
    let delivered = git(&proj, &["show", "main:work.txt"]);
    assert_eq!(delivered, "delivered\n");

    // The same rung again in an agent's OWN space (§16.3), which owns its
    // worktrees as well as its store — and it runs before the anchor check,
    // which deliberately leaves `$XDG_DATA_HOME` unusable.
    marks::an_own_space_owns_its_worktrees(tmp.path(), &proj, &balls_state);

    an_unusable_anchor_fails_every_verb_but_a_probe(tmp.path());
}
