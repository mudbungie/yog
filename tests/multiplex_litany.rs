//! End-to-end drive of the W11 litany multiplex arm (DESIGN §16.7) through
//! `yog::multiplex::dispatch` — `yog litany <argv…>` IS litany's thin exec
//! binding, proven against a real litany home on disk: `prime` seeds it, `new`
//! mints a workspace (the one-product stdout line), `config` runs the whole
//! `$EDITOR` hand-off both ways, and a prelude-bearing verb walks the pgid
//! branch on its way to its own error.
//!
//! **This test binary owns its process environment.** The arm is a *binding*:
//! it reads `LITANY_HOME` (through the linked litany), the ambient
//! `$XDG_DATA_HOME` anchor (the world tools dir the re-entry shims converge
//! into), `$EDITOR`, and the git config lookup the litany-minted repos commit
//! through — all process-global, none injectable through
//! `dispatch`'s argv surface. So this file mutates its own env, which is
//! lawful exactly here: every `tests/*.rs` is a separate process, and the one
//! `#[test]` below is this binary's only test, so nothing observes the
//! mutation concurrently. `set_var` is `unsafe` in edition 2024; the repo's
//! unsafe-location rule confines `unsafe` in `src/` (the scan target), and
//! this env control has no safe in-process alternative — which is also why
//! this coverage cannot live beside the arm's unit tests.
//!
//! **It is also the isolation proof (bl-81c9).** `LITANY_HOME` is set to a
//! scratch *ambient* root that must stay empty: the arm stands the process in
//! the world first (`world::inhabit`), so every verb below seeds and reads
//! `<anchor>/yog/world/litany` and the ambient root the operator's own `litany`
//! would use is never written. This file asserted the opposite until bl-81c9 —
//! the arm was resolving whatever `LITANY_HOME` the caller happened to carry,
//! which for a bare `yog litany` at a shell is the operator's own harness.

// clippy's allow-*-in-tests reaches `#[test]` fns, not the free fixture
// helpers of an integration-test crate — those unwrap freely like any test
// (the `tests/support` precedent).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use yog::multiplex::dispatch;

// git vars a hook-invoked test may inherit. Scrubbed from this binary's OWN env
// — the litany it drives runs in-process and spawns its own git — from
// `yog::git_env`'s list, the one the spawn sites use.
use yog::git_env::INHERITED as INHERITED_GIT_ENV;

fn set(key: &str, value: &Path) {
    // SAFETY: single-threaded — this binary runs exactly one #[test], and no
    // other thread exists to read the env concurrently (module doc).
    unsafe { std::env::set_var(key, value) };
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

/// The scratch **global** gitconfig: litany mints its own repos, so the only
/// place an identity can be planted is git's config lookup — and this test may
/// not depend on the developer's. Written into the scratch dir and named by
/// `GIT_CONFIG_GLOBAL`, it is both the identity (a CI runner has none: "empty
/// ident name" aborted every commit) and the wall that keeps every *other*
/// ambient global setting (`commit.gpgsign`, `core.hooksPath`, …) out.
fn fixture_gitconfig(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("gitconfig");
    fs::write(
        &path,
        "[user]\n\tname = Tester\n\temail = t@test.invalid\n[commit]\n\tgpgsign = false\n",
    )
    .unwrap();
    path
}

/// A scripted `$EDITOR` (the upstream `config_cli.rs` idiom): writes `content`
/// to `rel` inside the checkout it is handed as `"$1"`.
fn editor_writing(dir: &Path, rel: &str, content: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("scripted-editor.sh");
    fs::write(
        &path,
        format!("#!/bin/sh\nprintf '%s' '{content}' > \"$1/{rel}\"\n"),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn the_litany_arm_is_the_thin_binding_end_to_end() {
    let tmp = tempfile::TempDir::new().unwrap();
    for var in INHERITED_GIT_ENV {
        // SAFETY: as `set` — one test, one thread.
        unsafe { std::env::remove_var(var) };
    }
    for var in ["LITANY_BINARY", "BZ_BINARY"] {
        // SAFETY: as `set` — the shims must converge from the default
        // (self-multiplex) resolution, not a machine-local override.
        unsafe { std::env::remove_var(var) };
    }
    // The AMBIENT harness root — a conflicting one, deliberately: it stands for
    // the operator's own `~/.config/litany`, and nothing below may write it.
    let ambient_home = tmp.path().join("litany-home");
    set("LITANY_HOME", &ambient_home);
    set("XDG_DATA_HOME", &tmp.path().join("xdg-data"));
    // …and the nested one every verb must actually reach (§16.2's fold).
    let world_home = tmp.path().join("xdg-data/yog/world/litany");
    set("EDITOR", Path::new("true"));
    // git reads exactly one config file: the scratch fixture. `/dev/null` is
    // the empty system config (git skips a system file it cannot read as one).
    set("GIT_CONFIG_GLOBAL", &fixture_gitconfig(tmp.path()));
    set("GIT_CONFIG_SYSTEM", Path::new("/dev/null"));

    // Parse short-circuits return before any prelude, env read, world fold, or
    // disk write — which is why bl-81c9's fold sits *below* the parse and a
    // probe still touches nothing (`std::env` is proof: `LITANY_HOME` is still
    // the ambient one after both of these).
    assert_eq!(dispatch(&argv(&["yog", "litany", "--help"])), Some(0));
    assert_eq!(dispatch(&argv(&["yog", "litany", "no-such-verb"])), Some(2));
    assert_eq!(
        std::env::var("LITANY_HOME").unwrap(),
        ambient_home.display().to_string()
    );

    // `prime` — the full run path: the world folded into this process's env, no
    // preludes, both re-entry shims converged into the world tools dir under the
    // ambient anchor, Fx built over the locked stdio, `Outcome::Quiet` (silent
    // seed-if-absent success). It seeds the WORLD's harness root and leaves the
    // conflicting ambient one absent (bl-81c9).
    assert_eq!(dispatch(&argv(&["yog", "litany", "prime"])), Some(0));
    assert!(world_home.join("models.yaml").is_file());
    assert!(
        !ambient_home.exists(),
        "ambient harness root written: {}",
        ambient_home.display()
    );
    let tools = tmp.path().join("xdg-data/yog/world/tools");
    for shim in ["litany", "bz"] {
        let body = fs::read_to_string(tools.join(shim)).unwrap();
        assert!(body.contains(&format!("'{shim}' \"$@\"")), "shim: {body}");
    }

    // **Re-entry is a no-op** (bl-81c9): this process now stands in the world,
    // which is the state every *spawned* `yog litany` starts in. A second verb
    // re-folds the identical set — the harness root is unmoved and the tools dir
    // appears on `PATH` exactly once, never stacked.
    assert_eq!(dispatch(&argv(&["yog", "litany", "prime"])), Some(0));
    assert_eq!(
        std::env::var("LITANY_HOME").unwrap(),
        world_home.display().to_string()
    );
    let tools_s = tools.display().to_string();
    assert_eq!(
        std::env::var("PATH")
            .unwrap()
            .split(':')
            .filter(|p| *p == tools_s)
            .count(),
        1,
        "world tools dir stacked on PATH"
    );

    // `new` — `Outcome::Line`: the destination path is the one stdout product.
    let ws = tmp.path().join("ws");
    let ws_s = ws.display().to_string();
    assert_eq!(dispatch(&argv(&["yog", "litany", "new", &ws_s])), Some(0));
    assert!(ws.join("repo.git").is_dir());

    // `config` — the `$EDITOR` hand-off: a scripted editor edits the checkout
    // litany hands it, litany commits, exit 0. Both the arm's `$EDITOR`
    // resolution and the `sh -c` spawn are the upstream binding's, verbatim.
    let ed = editor_writing(tmp.path(), "providers.yaml", "roles: {}\n");
    set("EDITOR", &ed);
    assert_eq!(
        dispatch(&argv(&["yog", "litany", "config", &ws_s])),
        Some(0)
    );
    // …and a failing editor is a failed edit: the verb's uniform error, exit 1.
    set("EDITOR", Path::new("false"));
    assert_eq!(
        dispatch(&argv(&["yog", "litany", "config", &ws_s])),
        Some(1)
    );

    // A prelude-bearing verb (`dispatch`: pgid leadership — this process is
    // its own test binary, so taking a group is safe) walks the prelude loop,
    // then meets its own agent-id validation error: the uniform failure, 1.
    assert_eq!(
        dispatch(&argv(&[
            "yog", "litany", "dispatch", "worker", &ws_s, "bad/id", "--goal", "g",
        ])),
        Some(1),
    );

    // An unusable anchor is a failed converge: the re-entry shims are the
    // verb's precondition, so a tools dir that cannot exist reports and fails
    // (1) before the verb runs at all — no litany work on invalid targets.
    // Anchoring `$XDG_DATA_HOME` inside a regular file makes the dir
    // uncreatable (ENOTDIR), the one failure mode of `tools::ensure_shim`.
    let blocked = tmp.path().join("not-a-dir");
    fs::write(&blocked, "").unwrap();
    set("XDG_DATA_HOME", &blocked.join("xdg-data"));
    assert_eq!(dispatch(&argv(&["yog", "litany", "prime"])), Some(1));
}
