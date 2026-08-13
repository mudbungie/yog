//! Tests for the `--editor-apply` shim (§9.3): the pure copy fn (nested
//! dirs, only-staged files, symlink hygiene, empty staging) and the
//! [`run_shim`] exit-code mapping (both missing inputs, a copy error, and
//! the happy path).

use super::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use tempfile::tempdir;

/// Read a file under `root` as UTF-8, panicking with the path on absence.
fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|_| panic!("missing {rel}"))
}

#[test]
fn copies_nested_files_preserving_structure() {
    let dir = tempdir().unwrap();
    let staging = dir.path().join("stage");
    let checkout = dir.path().join("checkout");
    fs::create_dir_all(staging.join("souls")).unwrap();
    fs::write(staging.join("workflow.yaml"), b"wf").unwrap();
    fs::write(staging.join("souls/coder.md"), b"soul").unwrap();

    let written = copy_staged(&staging, &checkout).unwrap();

    assert_eq!(
        written,
        vec![
            PathBuf::from("souls/coder.md"),
            PathBuf::from("workflow.yaml")
        ]
    );
    assert_eq!(read(&checkout, "workflow.yaml"), "wf");
    assert_eq!(read(&checkout, "souls/coder.md"), "soul");
}

#[test]
fn never_deletes_unstaged_checkout_files() {
    // The invariant that protects lernie's freshly-refreshed descriptions/**:
    // a file present in the checkout but absent from staging survives, and a
    // staged file overwrites its checkout counterpart.
    let dir = tempdir().unwrap();
    let staging = dir.path().join("stage");
    let checkout = dir.path().join("checkout");
    fs::create_dir_all(checkout.join("descriptions")).unwrap();
    fs::write(checkout.join("descriptions/pool.md"), b"lernie-refreshed").unwrap();
    fs::write(checkout.join("workflow.yaml"), b"old").unwrap();
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("workflow.yaml"), b"new").unwrap();

    let written = copy_staged(&staging, &checkout).unwrap();

    assert_eq!(written, vec![PathBuf::from("workflow.yaml")]);
    assert_eq!(read(&checkout, "descriptions/pool.md"), "lernie-refreshed");
    assert_eq!(read(&checkout, "workflow.yaml"), "new");
}

#[cfg(unix)]
#[test]
fn skips_symlinks_in_staging() {
    let dir = tempdir().unwrap();
    let staging = dir.path().join("stage");
    let checkout = dir.path().join("checkout");
    fs::create_dir_all(&staging).unwrap();
    fs::write(dir.path().join("outside-secret"), b"secret").unwrap();
    fs::write(staging.join("providers.yaml"), b"pv").unwrap();
    // A symlink pointing outside staging must not be followed or reproduced.
    symlink(dir.path().join("outside-secret"), staging.join("link")).unwrap();

    let written = copy_staged(&staging, &checkout).unwrap();

    assert_eq!(written, vec![PathBuf::from("providers.yaml")]);
    assert!(!checkout.join("link").exists());
}

#[test]
fn empty_staging_copies_nothing() {
    let dir = tempdir().unwrap();
    let staging = dir.path().join("stage");
    let checkout = dir.path().join("checkout");
    fs::create_dir_all(&staging).unwrap();

    let written = copy_staged(&staging, &checkout).unwrap();

    assert!(written.is_empty());
    assert!(!checkout.exists());
}

#[test]
fn run_shim_ok_copies_and_returns_zero() {
    let dir = tempdir().unwrap();
    let staging = dir.path().join("stage");
    let checkout = dir.path().join("checkout");
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("version"), b"2").unwrap();

    let code = run_shim(
        Some(staging.display().to_string()),
        Some(checkout.display().to_string()),
    );

    assert_eq!(code, 0);
    assert_eq!(read(&checkout, "version"), "2");
}

#[test]
fn run_shim_missing_edit_src_is_nonzero() {
    assert_eq!(run_shim(None, Some("/some/checkout".into())), 1);
    // Empty reads as absent, too.
    assert_eq!(
        run_shim(Some(String::new()), Some("/some/checkout".into())),
        1
    );
}

#[test]
fn run_shim_missing_checkout_is_nonzero() {
    assert_eq!(run_shim(Some("/some/stage".into()), None), 1);
    assert_eq!(run_shim(Some("/some/stage".into()), Some(String::new())), 1);
}

#[test]
fn run_shim_copy_error_is_nonzero() {
    // A staging dir that does not exist makes the first read_dir fail.
    let dir = tempdir().unwrap();
    let code = run_shim(
        Some(dir.path().join("nonexistent").display().to_string()),
        Some(dir.path().join("checkout").display().to_string()),
    );
    assert_eq!(code, 1);
}
