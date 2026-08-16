//! The confinement backend: the platform switch, the probe's three verdicts,
//! the refusal's wording, the wrapper argv — and, where the box itself has the
//! backend, the real sandbox's support boundary (writes clamped to the bound
//! set, env passed through). The real-backend test skips by the same
//! derivation the product spends, and every refusal arm is covered without it.

use super::*;
use crate::actions::verbs::collect;
use crate::cli_outbound::Cli;
use crate::test_support::spawn_guard;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::tempdir;

fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
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
    let words = argv(Path::new("/data/yog/world"), Path::new("/w/alba"));
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
        "--",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(words, expect);
}

/// The support boundary, proven against the real backend where the box has
/// one: a write inside the bound set lands on the host, a write outside it is
/// refused by the read-only rebind, and the composed env rides through. Skips
/// by [`available`] — the product's own derivation — on a box without `bwrap`;
/// the scratch sits under the crate's own `target/` so "outside the binds" is
/// a host-writable place the sandbox must still refuse.
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
    let ws = scratch.join("ws");
    let world_root = scratch.join("world");
    fs::create_dir_all(&ws).unwrap();
    fs::create_dir_all(&world_root).unwrap();
    let sh = format!(
        "touch {ws}/inside.txt || echo IN_FAIL; touch {scratch}/outside.txt 2>/dev/null && \
         echo OUT_WROTE; printf 'env=%s' \"$YOG_CONFINE_PROOF\"",
        ws = ws.display(),
        scratch = scratch.display(),
    );
    let cli = Cli::new("/bin/sh").and_wrapper(argv(&world_root, &ws));
    let outcome = collect(cli.run_env(&[("YOG_CONFINE_PROOF", "held")], &["-c", &sh])).unwrap();
    drop(guard);
    assert_eq!(outcome.exit, 0, "{}", outcome.stderr);
    assert!(!outcome.stdout.contains("IN_FAIL"), "{}", outcome.stdout);
    assert!(!outcome.stdout.contains("OUT_WROTE"), "{}", outcome.stdout);
    assert!(outcome.stdout.contains("env=held"), "{}", outcome.stdout);
    assert!(ws.join("inside.txt").exists());
    assert!(!scratch.join("outside.txt").exists());
    fs::remove_dir_all(&scratch).unwrap();
}
