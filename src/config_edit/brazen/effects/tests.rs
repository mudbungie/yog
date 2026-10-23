//! The production runner's reads (§16.7 W10, bl-dff8), every one of them
//! **in-process**: driven against a hermetic wall so the linked brazen reads a
//! temp file and never the operator's own config. No network here —
//! `--dump-config` and `--list-providers` are offline by construction, and the
//! roster case names a provider row that does not exist, which brazen refuses
//! at config resolution, before any request is composed.

use super::*;
use std::fs;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// A runner whose linked-brazen reads fold through a hermetic env standing in
/// the wall at `wall` (§16.2 as amended — the wall, not an ambient config, is
/// what names brazen's file).
fn in_process(wall: &Path) -> RealBzRunner {
    RealBzRunner::new(Env::from_pairs([(
        crate::world::wall::YOG_WALL,
        wall.display().to_string(),
    )]))
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

#[test]
fn resolve_keeps_the_world_env_the_reads_fold_through() {
    let world = Env::from_pairs([("HOME", "/h"), ("XDG_DATA_HOME", "/d")]);
    let runner = RealBzRunner::resolve(&world);
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
fn providers_lists_the_routing_table_with_a_named_preset_filled_in() {
    // The listing keeps the defaults operand the dump drops (§5.1 #21), so a
    // built-in row the operator NAMES — one line, the preset fills the rest
    // through brazen's per-name merge — appears beside the file's own row, in
    // routing order. The preset is named rather than relied on, because since
    // brazen bl-6feb an untouched preset is not in the table at all; naming it
    // is what holds this fixture true on both sides of that pin.
    let (_dir, path) = config_file(
        "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\nbase_url = \"https://acme.test\"\nauth = \"none\"\n\n[[provider]]\nname = \"anthropic\"\n",
    );
    let rows = in_process(&path).providers();
    let named = |n: &str| rows.iter().find(|p| p.name == n).cloned();
    assert_eq!(
        named("acme").map(|p| p.auth),
        Some("none".to_owned()),
        "rows: {rows:?}"
    );
    // The named api-keyed preset: the `auth` column rides through the real
    // projection, which is what makes it the login-capability answer (§8.3),
    // and with no key anywhere its credential column is the one refusal
    // spelling — on a row the operator named, which is the only place it can
    // now appear.
    let anthropic = named("anthropic").expect("a named preset is in the table");
    assert_eq!(anthropic.auth, "api_key", "rows: {rows:?}");
    assert_eq!(anthropic.credential, crate::config_edit::brazen::MISSING);
}

/// The sign-in catalog is read in-process through the login route with no
/// target named, and it never leaves the machine: no browser, no loopback
/// bind, no credential written. Every row it answers is an oauth row — that is
/// the catalog's whole membership rule — and a brazen before 0.0.19, which
/// prints no catalog (exit 78), folds to no rows rather than an error, so this
/// holds on both sides of the pin.
#[test]
fn the_catalog_reads_in_process_and_answers_oauth_rows_only() {
    let (_dir, path) = config_file(
        "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\nbase_url = \"https://acme.test\"\nauth = \"none\"\n",
    );
    let rows = in_process(&path).catalog();
    assert!(
        rows.iter().all(|row| row.auth == "oauth2"),
        "a keyless or keyed row is not in the sign-in catalog: {rows:?}"
    );
    assert!(
        !rows.iter().any(|row| row.name == "acme"),
        "the file's keyless row cannot be signed into: {rows:?}"
    );
    // A malformed config answers no rows here too — brazen exits non-zero and
    // prints nothing to stdout, exactly as the table read folds.
    let (_dir, broken) = config_file("= not toml\n");
    assert_eq!(in_process(&broken).catalog(), Vec::new());
}

#[test]
fn a_malformed_config_yields_no_provider_rows() {
    // brazen exits non-zero and prints nothing to stdout, so the fold is empty
    // — the login surface simply offers no rows, never a panic.
    let (_dir, path) = config_file("= not toml\n");
    assert_eq!(in_process(&path).providers(), Vec::new());
}

/// The roster read is in-process too (bl-dff8), and it refuses in brazen's own
/// words: a provider row the effective table does not have is a config error,
/// raised before a request exists — which is also why this case is offline.
#[test]
fn list_models_reads_in_process_and_refuses_an_unknown_row() {
    let (_dir, wall) = config_file(
        "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\nbase_url = \"https://acme.test\"\nauth = \"none\"\n",
    );
    let out = in_process(&wall).list_models("not-a-row");
    assert!(!out.success);
    assert!(out.stderr.contains("not-a-row"), "stderr: {}", out.stderr);
    assert!(out.stdout.is_empty());
}
