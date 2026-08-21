//! World-seed tests (DESIGN §16.6 W3): the pure [`seeded`] marker probe and the
//! [`ensure_seeded`] converger — the argv landing and the **standing** world
//! `LERNIE_HOME` (§16.6 W2) reaching the child (seed sets no per-call env), the
//! seeded-skip proven (a bogus binary is never run), the ops entry written, and
//! the non-zero / spawn-failure error arms. Fork-based, so the spawning tests
//! hold the crate-wide `SPAWN_LOCK` (the ETXTBSY discipline).

use super::{SeedError, ensure_seeded, seeded};
use crate::cli_outbound::Cli;
use crate::opslog::{self, OpEntry, Origin};
use crate::world::{Layout, layout_under};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::{TempDir, tempdir};

/// A hermetic world: a dir for the fake `lernie`, a state root for `ops.jsonl`,
/// and the yog data-root anchor the world layout derives from.
struct World {
    bin: TempDir,
    state: TempDir,
    yog: TempDir,
}

impl World {
    fn new() -> Self {
        Self {
            bin: tempdir().unwrap(),
            state: tempdir().unwrap(),
            yog: tempdir().unwrap(),
        }
    }

    /// The world layout anchored on this world's yog data root.
    fn layout(&self) -> Layout {
        layout_under(self.yog.path())
    }

    /// Materialize the seeded marker — `<LERNIE_HOME>/models.yaml`.
    fn seed_marker(&self) {
        let lernie = self.layout().lernie;
        fs::create_dir_all(&lernie).unwrap();
        fs::write(lernie.join("models.yaml"), b"models: {}\n").unwrap();
    }

    /// The logged `ops.jsonl` entries, oldest-first.
    fn ops(&self) -> Vec<OpEntry> {
        opslog::tail(self.state.path(), 16)
    }
}

/// Write an executable `lernie` (0755) with the given shell `body`.
fn fake_lernie(dir: &Path, body: &str) -> std::path::PathBuf {
    let path = dir.join("lernie");
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

#[test]
fn seeded_is_true_iff_models_yaml_is_present() {
    let w = World::new();
    assert!(!seeded(&w.layout()), "a fresh world has no models.yaml");
    w.seed_marker();
    assert!(seeded(&w.layout()), "the founded marker reads as seeded");
}

#[test]
fn ensure_seeded_skips_a_seeded_world() {
    // Marker present → no spawn, nothing logged. A bogus binary proves the skip:
    // ensure_seeded would error (Io) if it ran it.
    let w = World::new();
    w.seed_marker();
    let lernie = Cli::new("/definitely/not/a/real/lernie");
    let primed = ensure_seeded(
        &lernie,
        w.state.path(),
        "TS",
        &w.layout(),
        Origin::Conversation,
    )
    .unwrap();
    assert!(!primed, "a seeded world is skipped");
    assert!(w.ops().is_empty(), "the skip runs and logs nothing");
}

#[test]
fn ensure_seeded_primes_with_lernie_home_and_logs() {
    let w = World::new();
    let report = w.bin.path().join("home");
    // The fake records the `LERNIE_HOME` it received (printf builtin only — no
    // fork to race the coverage ptrace engine) and exits 0, product-less.
    let body = format!(
        "#!/bin/sh\nprintf '%s' \"$LERNIE_HOME\" > '{}'\n",
        report.display()
    );
    let layout = w.layout();
    // seed sets NO per-call env (§16.6 W3 collapse); LERNIE_HOME must ride the
    // standing world env `lernie` carries (as it does in production via
    // `Cli::resolve_in_world`). Stand it here on the fake binary.
    let lernie = Cli::new(fake_lernie(w.bin.path(), &body)).with_env(vec![(
        "LERNIE_HOME".to_owned(),
        layout.lernie.to_string_lossy().into_owned(),
    )]);
    let primed =
        ensure_seeded(&lernie, w.state.path(), "TS", &layout, Origin::Conversation).unwrap();
    assert!(primed, "an unseeded world is primed");
    // The standing env landed: the child saw LERNIE_HOME = the world's lernie home.
    let got = fs::read_to_string(&report).unwrap();
    assert_eq!(Path::new(&got), layout.lernie);
    // The argv landed and the outcome is logged (cwd inherits, logged blank).
    let e = w.ops();
    assert_eq!(e.len(), 1);
    assert_eq!(&e[0].argv[1..], &["prime"]);
    assert_eq!(e[0].exit, 0);
    assert_eq!(e[0].cwd, "");
    assert!(e[0].stderr.is_empty(), "a clean prime logs no error");
}

#[test]
fn ensure_seeded_errors_on_a_nonzero_prime() {
    let w = World::new();
    let body = "#!/bin/sh\nprintf '%s\\n' 'prime boom' 1>&2\nexit 3\n";
    let lernie = Cli::new(fake_lernie(w.bin.path(), body));
    let err = ensure_seeded(
        &lernie,
        w.state.path(),
        "TS",
        &w.layout(),
        Origin::Conversation,
    )
    .unwrap_err();
    let SeedError::Prime(out) = err else {
        panic!("expected Prime, got {err:?}");
    };
    assert_eq!(out.exit, 3);
    assert!(out.stderr.contains("prime boom"));
    // A completed-but-non-zero prime is still logged (§8.2).
    assert_eq!(w.ops().len(), 1);
    // The Display carries the exit + stderr.
    assert!(SeedError::Prime(out).to_string().contains("exit 3"));
}

#[test]
fn ensure_seeded_surfaces_a_spawn_failure() {
    let w = World::new();
    let lernie = Cli::new("/definitely/not/a/real/lernie");
    let err = ensure_seeded(
        &lernie,
        w.state.path(),
        "TS",
        &w.layout(),
        Origin::Conversation,
    )
    .unwrap_err();
    assert!(matches!(err, SeedError::Io(_)), "a missing binary is Io");
    assert!(!err.to_string().is_empty(), "the Io error renders");
    // §4.2 as amended: the spawn failure is a rendered fact — a synthetic line
    // with the intended argv and the error in stderr (no longer un-logged).
    let ops = w.ops();
    assert_eq!(ops.len(), 1, "a spawn failure appends a synthetic ops line");
    assert_eq!(ops[0].exit, crate::opslog::SYNTHETIC_EXIT);
    assert_eq!(ops[0].argv[0], "/definitely/not/a/real/lernie");
    assert!(!ops[0].stderr.is_empty());
}
