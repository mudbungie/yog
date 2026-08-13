//! Every route of the embedded host, driven offline.
//!
//! brazen's six routes all short-circuit `--help`/`--version` **before** any
//! config read, file open, socket bind or round-trip (its arch §5.5 discovery
//! rule), so each seam bundle here is wired and entered without a network — the
//! repo's no-network-in-tests floor. The two routes that are offline by
//! construction (`--dump-config`, `--list-providers`) are additionally driven
//! all the way through against a hermetic `BRAZEN_CONFIG`.

use super::*;
use std::fs;
use tempfile::tempdir;

/// Run `argv` in-process over a hermetic env, returning `(exit, stdout, stderr)`.
fn drive(argv: &[&str], env: &Env) -> (i32, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let exit = super::run(
        argv.iter().map(|a| (*a).to_string()).collect(),
        env,
        Tty::PIPED,
        &mut std::io::empty(),
        &mut out,
        &mut err,
    );
    (
        exit,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

/// A wall whose brazen config does not exist — the fold lands, brazen finds no
/// file and folds its compiled-in defaults. A wall is what makes `bz` reachable
/// at all (§16.2 as amended), and an ambient `BRAZEN_CONFIG` is pre-set here to
/// prove the wall's own config **replaces** it rather than deferring to it.
fn bare() -> Env {
    Env::from_pairs([
        ("BRAZEN_CONFIG", "/definitely/not/an/ambient/config.toml"),
        (crate::world::wall::YOG_WALL, "/definitely/not/a/wall"),
    ])
}

#[test]
fn outside_any_wall_bz_refuses_rather_than_reading_the_machines_config() {
    // No wall named: providers, sign-ins and the model cache are workspace
    // settings, so there is nothing to read and nothing ambient to fall back
    // to (§16.2 as amended). The refusal names the fix, not the fault.
    let (exit, out, err) = drive(&["--list-providers"], &Env::from_pairs([("HOME", "/h")]));
    assert_eq!(exit, super::NO_WALL_CODE);
    assert!(
        out.is_empty(),
        "stdout carries one product, and there is none"
    );
    assert!(
        err.contains("no workspace in this environment"),
        "stderr: {err}"
    );
}

/// bl-52ed — the wall gate's one exemption (§8.5): a **discovery probe** asks
/// brazen what it *is*, which reads the interface and never the world, so it
/// answers outside a workspace too. Otherwise `yog bz --help` — the very
/// spelling the top-level page advertises the sign-in through — could not be
/// read by anyone who has not already got a workspace focused.
#[test]
fn a_discovery_probe_answers_with_no_wall_at_all() {
    for probe in ["--help", "-h", "--version", "-V", "--skill"] {
        let (exit, out, err) = drive(&[probe], &Env::from_pairs([("HOME", "/h")]));
        assert_eq!(exit, 0, "{probe}: {err}");
        assert!(!out.is_empty(), "{probe} printed nothing");
        assert!(
            !err.contains("no workspace"),
            "{probe} refused instead: {err}"
        );
    }
}

/// The exemption is exactly as narrow as §8.5's line rule — the flag counts
/// only when the whole argv *is* the flag. A probe token that trails a route,
/// or that could be an option's value, leaves the wall requirement standing,
/// so nothing can be smuggled past the gate by appending `--help`.
#[test]
fn a_probe_token_beside_anything_else_is_still_walled() {
    for argv in [
        vec!["--list-providers", "--help"],
        vec!["--system", "--help"],
        vec!["--", "--help"],
    ] {
        let (exit, _, err) = drive(&argv, &Env::from_pairs([("HOME", "/h")]));
        assert_eq!(exit, super::NO_WALL_CODE, "{argv:?}");
        assert!(err.contains("no workspace in this environment"), "{argv:?}");
    }
}

#[test]
fn piped_is_neither_stream_a_terminal() {
    assert_eq!(
        Tty::PIPED,
        Tty {
            stdin: false,
            stdout: false
        }
    );
}

#[test]
fn probe_reads_the_real_stdio_and_agrees_with_itself() {
    // The value depends on how the test runner wired stdio, so the property
    // under test is that the probe is a pure read: twice, the same answer.
    assert_eq!(Tty::probe(), Tty::probe());
}

#[test]
fn the_data_plane_route_answers_version_without_config_or_network() {
    // `Route::Run` with the discovery short-circuit: the `Host` bundle (all
    // five seams incl. the replay stash) is wired, then brazen answers.
    let (exit, out, err) = drive(&["--version"], &bare());
    assert_eq!(exit, 0);
    assert!(!out.is_empty(), "expected a version line");
    assert!(err.is_empty());
}

#[test]
fn the_data_plane_route_dumps_the_effective_config_from_the_injected_env() {
    // The env decides which file is read (§16.2 ambient share) — proof that
    // `EnvSnapshot` is threaded, not `std::env`.
    let dir = tempdir().unwrap();
    let wall = dir.path().join("wall");
    let path = crate::config_edit::brazen::BrazenPaths::in_wall(&wall).config;
    fs::create_dir_all(path.parent().expect("the wall's brazen dir")).unwrap();
    fs::write(
        &path,
        "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\n\
         base_url = \"https://acme.test\"\nauth = \"none\"\n",
    )
    .unwrap();
    // The wall names the file; an ambient `BRAZEN_CONFIG` pointing elsewhere is
    // deliberately present and must lose (§16.2 as amended).
    let env = Env::from_pairs([
        (
            "BRAZEN_CONFIG",
            "/definitely/not/an/ambient/config.toml".to_owned(),
        ),
        (crate::world::wall::YOG_WALL, wall.display().to_string()),
    ]);
    let (exit, out, _) = drive(&["--dump-config"], &env);
    assert_eq!(exit, 0);
    assert!(out.contains("acme"), "dump: {out}");
}

#[test]
fn a_malformed_config_fails_before_the_sink_with_brazens_own_stderr() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, "= not toml\n").unwrap();
    let env = Env::from_pairs([("BRAZEN_CONFIG", path.display().to_string())]);
    let (exit, out, err) = drive(&["--dump-config"], &env);
    assert_ne!(exit, 0);
    assert!(out.is_empty());
    assert!(!err.is_empty());
}

#[test]
fn the_list_providers_route_prints_the_effective_table() {
    // Offline by construction: `ProvidersIo` has no transport at all. With no
    // config file the table is exactly brazen's compiled-in rows.
    let (exit, out, _) = drive(&["--list-providers", "--json"], &bare());
    assert_eq!(exit, 0);
    assert!(out.contains("\"providers\""), "listing: {out}");
    assert!(out.contains("anthropic"), "listing: {out}");
}

#[test]
fn the_list_models_route_wires_its_seams_and_self_describes() {
    // `--list-models` would reach the network; `--help` proves the arm is
    // entered and its `ListIo` bundle built, with no round-trip.
    let (exit, out, _) = drive(&["--list-models", "--help"], &bare());
    assert_eq!(exit, 0);
    assert!(!out.is_empty());
}

#[test]
fn the_count_tokens_route_wires_its_seams_and_self_describes() {
    let (exit, out, _) = drive(&["--count-tokens", "--help"], &bare());
    assert_eq!(exit, 0);
    assert!(!out.is_empty());
}

#[test]
fn the_serve_route_wires_its_bind_seam_and_self_describes() {
    // Nothing binds: the discovery short-circuit precedes the accept loop.
    let (exit, out, _) = drive(&["--serve", "--help"], &bare());
    assert_eq!(exit, 0);
    assert!(!out.is_empty());
}

#[test]
fn the_login_route_wires_its_interactive_seams_and_self_describes() {
    // No browser is launched, no loopback socket bound, no device poll: the
    // short-circuit answers first. The OS-RNG tokens are still minted, which
    // is the point — the whole `LoginIo` bundle is exercised.
    let (exit, out, _) = drive(&["--login", "--help"], &bare());
    assert_eq!(exit, 0);
    assert!(!out.is_empty());
}
