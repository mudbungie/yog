//! The confinement backend: the platform switch, the probe's three verdicts,
//! the refusal's wording, the wrapper argv, the derived writable set — its
//! fourth member the bound project the claim trail names (bl-34b1), and the one
//! rule that drops a member no longer on disk — and, where the box itself has
//! the backend, the real sandbox's support boundary (writes clamped to that same
//! derived set, env passed through). The real-backend test skips by the same
//! derivation the product spends, and every refusal arm is covered without it.

use super::*;
use crate::actions::verbs::collect;
use crate::cli_outbound::Cli;
use crate::opslog::{OpEntry, Origin};
use crate::test_support::spawn_guard;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

/// A world whose state and data roots sit under `scratch`, so
/// [`writable`]'s two derivations — the world root off `XDG_DATA_HOME`, the
/// claim trail off `XDG_STATE_HOME` — both land in a throwaway tree.
fn world_at(scratch: &Path) -> crate::xdg::Env {
    crate::xdg::Env::from_pairs([
        (
            "XDG_STATE_HOME",
            scratch.join("state").to_string_lossy().into_owned(),
        ),
        (
            "XDG_DATA_HOME",
            scratch.join("data").to_string_lossy().into_owned(),
        ),
    ])
}

/// The one yog-owned fact the bound project derives from: a `bl claim` row this
/// workspace's leaf stamped, run in `project` (§3.2's claimant join).
fn claim(world: &crate::xdg::Env, project: &Path, claimant: &str) {
    crate::opslog::append(
        &world.yog_state_root(),
        &OpEntry {
            ts: "TS".to_owned(),
            argv: ["bl", "claim", "bl-1234", "--as", claimant]
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
            cwd: project.to_string_lossy().into_owned(),
            exit: 0,
            stdout: String::new(),
            stderr: String::new(),
            origin: Origin::Balls,
        },
    )
    .unwrap();
}

#[test]
fn the_backend_is_linux_only_and_every_other_os_is_named() {
    assert_eq!(backend_for("linux"), Ok(BACKEND));
    let err = backend_for("macos").expect_err("no macOS backend is wired");
    assert!(err.contains("macos"), "{err}");
    assert!(backend_for("windows").is_err());
}

#[test]
fn the_probe_believes_a_zero_exit_and_names_both_unavailabilities() {
    let dir = tempdir().unwrap();
    let guard = spawn_guard();
    assert_eq!(
        probe(&script(dir.path(), "ok", "#!/bin/sh\nexit 0\n")),
        Ok(())
    );
    let err = probe(&script(
        dir.path(),
        "no",
        "#!/bin/sh\necho 'setting up uid map: Permission denied' >&2\nexit 1\n",
    ))
    .expect_err("a refusing kernel is unavailability");
    assert!(err.contains("exit 1"), "{err}");
    assert!(err.contains("Permission denied"), "{err}");
    let gone = probe(Path::new("/no/such/bwrap")).expect_err("an absent backend");
    drop(guard);
    assert!(gone.contains("could not run"), "{gone}");
}

#[test]
fn the_refusal_names_the_workspace_the_policy_file_and_the_why() {
    let text = refusal(
        Path::new("/w/alba"),
        "no confinement backend is wired for redox",
    );
    assert!(text.contains("/w/alba"), "{text}");
    assert!(
        text.contains(super::super::policy::CAPABILITY_YAML),
        "{text}"
    );
    assert!(text.contains("redox"), "{text}");
    assert!(text.contains("confinement: required"), "{text}");
}

#[test]
fn a_workspace_stating_no_policy_gates_nothing_and_wraps_nothing() {
    let ws = Path::new("/no/such/workspace");
    assert_eq!(gate(ws), Ok(()));
    let world = crate::xdg::Env::from_pairs([("XDG_DATA_HOME", "/data")]);
    assert_eq!(wrapper(&world, ws), Vec::<String>::new());
}

#[test]
fn the_argv_is_the_shape_plus_the_derived_writable_set_then_a_terminator() {
    let words = argv(&[
        PathBuf::from("/w/alba"),
        PathBuf::from("/data/yog/world"),
        PathBuf::from("/p/proj"),
    ]);
    let expect: Vec<String> = [
        "bwrap",
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--bind",
        "/tmp",
        "/tmp",
        "--bind",
        "/w/alba",
        "/w/alba",
        "--bind",
        "/data/yog/world",
        "/data/yog/world",
        "--bind",
        "/p/proj",
        "/p/proj",
        "--",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(words, expect);
}

/// The set's third member (bl-34b1): a workspace that claimed a ball through
/// yog gets the project the claim ran in, derived off the ops trail alone — the
/// one fact a revived driver still has — and a workspace that claimed nothing
/// gets yog's own two places and no more.
#[test]
fn the_writable_set_derives_the_bound_project_from_the_claim_trail() {
    let dir = tempdir().unwrap();
    let scratch = dir.path();
    let world = world_at(scratch);
    let ws = scratch.join("ws");
    let project = scratch.join("project");
    let world_root = scratch.join("data").join("yog").join("world");
    for d in [&ws, &project, &world_root] {
        fs::create_dir_all(d).unwrap();
    }
    assert_eq!(writable(&world, &ws), vec![ws.clone(), world_root.clone()]);
    claim(&world, &project, "ws");
    assert_eq!(
        writable(&world, &ws),
        vec![ws.clone(), world_root.clone(), project.clone()]
    );
    // …and the claimant is the workspace's own leaf (§3.2), so another
    // workspace's claim is not this one's project.
    let other = scratch.join("other");
    fs::create_dir_all(&other).unwrap();
    assert_eq!(writable(&world, &other), vec![other, world_root]);
}

/// A bind source that is not there is not bound: an orphaned project (§3.5)
/// narrows the set instead of failing every birth on `bwrap`'s own refusal.
#[test]
fn a_project_that_is_gone_drops_out_of_the_set_rather_than_breaking_the_spawn() {
    let dir = tempdir().unwrap();
    let scratch = dir.path();
    let world = world_at(scratch);
    let ws = scratch.join("ws");
    let world_root = scratch.join("data").join("yog").join("world");
    for d in [&ws, &world_root] {
        fs::create_dir_all(d).unwrap();
    }
    claim(&world, &scratch.join("burned"), "ws");
    assert_eq!(writable(&world, &ws), vec![ws, world_root]);
}

/// The support boundary, proven against the real backend where the box has
/// one: a write inside the bound set lands on the host — the workspace **and
/// the project the claim trail names, which is what a drone's own `bl close`
/// needs** (bl-34b1) — a write beside the project is refused by the read-only
/// rebind, and the composed env rides through. The set is the product's own
/// [`writable`] over a real trail, not a hand-built list, so what the sandbox
/// is proven to permit is what a birth actually spends. Skips by [`available`]
/// — the product's own derivation — on a box without `bwrap`; the scratch sits
/// under the crate's own `target/` so "outside the binds" is a host-writable
/// place the sandbox must still refuse.
#[test]
fn the_real_backend_clamps_writes_to_the_bound_set_and_passes_env_through() {
    let guard = spawn_guard();
    if available().is_err() {
        drop(guard);
        return;
    }
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("confine-scratch-{}", std::process::id()));
    let world = world_at(&scratch);
    let ws = scratch.join("ws");
    let project = scratch.join("project");
    let world_root = scratch.join("data").join("yog").join("world");
    for d in [&ws, &project, &world_root] {
        fs::create_dir_all(d).unwrap();
    }
    claim(&world, &project, "ws");
    let set = writable(&world, &ws);
    assert_eq!(set, vec![ws.clone(), world_root, project.clone()]);
    let sh = format!(
        "touch {ws}/inside.txt || echo IN_FAIL; touch {project}/delivered.txt || echo \
         PROJECT_FAIL; touch {scratch}/outside.txt 2>/dev/null && echo OUT_WROTE; printf \
         'env=%s' \"$YOG_CONFINE_PROOF\"",
        ws = ws.display(),
        project = project.display(),
        scratch = scratch.display(),
    );
    let cli = Cli::new("/bin/sh").and_wrapper(argv(&set));
    let outcome = collect(cli.run_env(&[("YOG_CONFINE_PROOF", "held")], &["-c", &sh])).unwrap();
    drop(guard);
    assert_eq!(outcome.exit, 0, "{}", outcome.stderr);
    assert!(!outcome.stdout.contains("IN_FAIL"), "{}", outcome.stdout);
    assert!(
        !outcome.stdout.contains("PROJECT_FAIL"),
        "{}",
        outcome.stdout
    );
    assert!(!outcome.stdout.contains("OUT_WROTE"), "{}", outcome.stdout);
    assert!(outcome.stdout.contains("env=held"), "{}", outcome.stdout);
    assert!(ws.join("inside.txt").exists());
    assert!(project.join("delivered.txt").exists());
    assert!(!scratch.join("outside.txt").exists());
    fs::remove_dir_all(&scratch).unwrap();
}
