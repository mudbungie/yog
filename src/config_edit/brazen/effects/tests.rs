//! The production runner's two halves (§16.7 W10): the **in-process** read
//! verbs, driven against a hermetic `BRAZEN_CONFIG` so the linked brazen reads
//! a temp file and never the operator's own config; and the one remaining
//! **spawn** (`--list-models`), driven by a recorder script the way
//! `cli_outbound`/`lock_probe` are. No network: `--dump-config` and
//! `--list-providers` are offline by construction, and the recorder never is a
//! real `bz`.

use super::*;
use crate::test_support::spawn_guard;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// A runner whose linked-brazen reads fold through a hermetic env standing in
/// the wall at `wall` (§16.2 as amended — the wall, not an ambient config, is
/// what names brazen's file). The `Cli` is a path that cannot spawn — these
/// cases never take the spawn branch.
fn in_process(wall: &Path) -> RealBzRunner {
    RealBzRunner::new(
        Cli::new("/definitely/not/a/real/bz-xyz"),
        Env::from_pairs([(crate::world::wall::YOG_WALL, wall.display().to_string())]),
    )
}

/// A tempdir holding a wall whose `brazen/config.toml` is `body`, plus the
/// wall root — what a focused workspace's fold resolves to.
fn config_file(body: &str) -> (TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let wall = dir.path().join("wall");
    let path = crate::config_edit::brazen::BrazenPaths::in_wall(&wall).config;
    fs::create_dir_all(path.parent().expect("the wall's brazen dir")).unwrap();
    fs::write(&path, body).unwrap();
    (dir, wall)
}

/// Write an executable recorder that logs argv to `log`, prints canned
/// stdout/stderr, and exits `code`. Returns the script path; the caller
/// holds `SPAWN_LOCK` across write+spawn (the ETXTBSY discipline).
fn recorder(dir: &Path, log: &Path, code: i32) -> PathBuf {
    let path = dir.join("bz");
    let body = format!(
        "#!/bin/sh\nfor a in \"$@\"; do printf '%s\\n' \"$a\" >> {}; done\n\
         printf 'OUT'\nprintf 'ERR' 1>&2\nexit {}\n",
        log.display(),
        code
    );
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

#[test]
fn resolve_keeps_the_bz_logical_name_and_the_world_env() {
    // `Binary::Bz` is self-multiplexed (§16.7 W10), so the physical target is
    // yog's own exe — but the LOGICAL name the ops log records stays `bz`.
    let world = Env::from_pairs([("HOME", "/h"), ("XDG_DATA_HOME", "/d")]);
    let runner = RealBzRunner::resolve(&world);
    assert_eq!(runner.cli.binary(), Path::new("bz"));
    // The world env rode along: the in-process reads fold through it.
    assert!(
        runner
            .env
            .pairs()
            .contains(&("HOME".to_string(), "/h".to_string()))
    );
}

#[test]
fn dump_config_effective_reads_the_linked_crate_in_process() {
    // A well-formed row dumps successfully — no spawn, no `bz` on PATH.
    let (_dir, path) = config_file(
        "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\nbase_url = \"https://acme.test\"\nauth = \"none\"\n",
    );
    let out = in_process(&path).dump_config_effective();
    assert!(out.success, "stderr: {}", out.stderr);
    assert!(out.stdout.contains("acme"), "stdout: {}", out.stdout);
}

#[test]
fn dump_config_at_gates_a_malformed_draft_with_brazens_own_stderr() {
    // The Apply gate (§9.1): a temp file brazen refuses is a non-success
    // outcome whose stderr is brazen's verbatim — the `Applied::Rejected` text.
    let (_dir, wall) = config_file("this is not toml = = =\n");
    let draft = crate::config_edit::brazen::BrazenPaths::in_wall(&wall).config;
    // `dump_config_at` names the file explicitly, so the runner's own wall is
    // deliberately a different (absent) one — the flag must win.
    let runner = in_process(Path::new("/definitely/not/a/wall"));
    let out = runner.dump_config_at(&draft);
    assert!(!out.success);
    assert!(!out.stderr.is_empty());
    assert!(out.stdout.is_empty());
}

#[test]
fn providers_lists_the_effective_table_including_the_built_in_rows() {
    // The listing keeps the defaults operand the dump drops (§5.1 #21), so the
    // built-in rows and the file's own row both appear, in routing order.
    let (_dir, path) = config_file(
        "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\nbase_url = \"https://acme.test\"\nauth = \"none\"\n",
    );
    let rows = in_process(&path).providers();
    let named = |n: &str| rows.iter().find(|p| p.name == n).cloned();
    assert_eq!(
        named("acme").map(|p| p.auth),
        Some("none".to_owned()),
        "rows: {rows:?}"
    );
    // A built-in api-keyed row: the `auth` column rides through the real
    // projection, which is what makes it the login-capability answer (§8.3).
    assert_eq!(
        named("anthropic").map(|p| p.auth),
        Some("api_key".to_owned()),
        "rows: {rows:?}"
    );
}

#[test]
fn a_malformed_config_yields_no_provider_rows() {
    // brazen exits non-zero and prints nothing to stdout, so the fold is empty
    // — the login surface simply offers no rows, never a panic.
    let (_dir, path) = config_file("= not toml\n");
    assert_eq!(in_process(&path).providers(), Vec::new());
}

#[test]
fn list_models_still_spawns_and_carries_its_argv() {
    let g = spawn_guard();
    let dir = tempdir().unwrap();
    let log = dir.path().join("argv");
    let bin = recorder(dir.path(), &log, 3);
    let runner = RealBzRunner::new(Cli::new(bin), Env::from_pairs([("HOME", "/h")]));
    let out = runner.list_models("openai");
    drop(g);
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "--list-models\n--provider\nopenai\n--json\n"
    );
    assert!(!out.success);
    assert_eq!(out.stderr, "ERR");
}

#[test]
fn list_models_spawn_failure_is_a_nonsuccess_outcome() {
    let _g = spawn_guard();
    let runner = in_process(Path::new("/definitely/not/a/wall"));
    let out = runner.list_models("openai");
    assert!(!out.success);
    assert!(!out.stderr.is_empty());
}
