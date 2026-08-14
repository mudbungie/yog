//! I3's temp naming, its recognizer and the §5.2 startup sweep — every arm
//! read in both directions: what the sweep takes, and what it must leave.

use super::{STALE_SECS, dirs, is_temp, stale, sweep, temp_in};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn the_temp_is_a_dotfile_beside_its_destination_and_is_recognized() {
    let tmp = temp_in(Path::new("/cfg"), "models.yaml");
    let name = tmp.file_name().unwrap().to_string_lossy().into_owned();
    assert_eq!(tmp.parent(), Some(Path::new("/cfg")));
    assert!(name.starts_with(".models.yaml.yog-tmp-"));
    assert!(name.ends_with(&std::process::id().to_string()));
    // The writer's own output is what the sweep recognizes — that is the point
    // of both living here (bl-e47c).
    assert!(is_temp(&name));
}

#[test]
fn is_temp_takes_only_what_this_module_writes() {
    assert!(is_temp(".ui.json.yog-tmp-1234"));
    assert!(is_temp(".cred.yog-tmp-0"));
    // not a dotfile
    assert!(!is_temp("ui.json.yog-tmp-1234"));
    // no destination name before the mark
    assert!(!is_temp(".yog-tmp-1234"));
    // no pid, or a pid that is not one
    assert!(!is_temp(".ui.json.yog-tmp-"));
    assert!(!is_temp(".notes.yog-tmp-backup"));
    assert!(!is_temp(".ui.json.yog-tmp-12a"));
    // the destination file itself, and an ordinary dotfile
    assert!(!is_temp("ui.json"));
    assert!(!is_temp(".gitignore"));
}

#[test]
fn stale_decides_on_the_24h_boundary() {
    let now = 1_000_000_000;
    let files = vec![
        (PathBuf::from("fresh"), now),
        (PathBuf::from("exactly-24h"), now - STALE_SECS),
        (PathBuf::from("stale"), now - STALE_SECS - 1),
    ];
    // Strictly more than 24 h is stale; exactly 24 h is kept.
    assert_eq!(stale(now, &files), vec![PathBuf::from("stale")]);
}

#[test]
fn sweep_removes_stale_temps_and_nothing_else() {
    let dir = tempdir().unwrap();
    let d = dir.path();
    let mine = temp_in(d, "ui.json");
    let other_pid = d.join(".config.toml.yog-tmp-999999");
    let dest = d.join("ui.json");
    let operators = d.join(".notes.yog-tmp-backup");
    let sub = d.join(".stage.yog-tmp-4242");
    for f in [&mine, &other_pid, &dest, &operators] {
        fs::write(f, b"x").unwrap();
    }
    // A *directory* wearing the name, and a dangling symlink wearing it: the
    // sweep deletes files, so neither may be touched.
    fs::create_dir(&sub).unwrap();
    let link = d.join(".linked.yog-tmp-7");
    std::os::unix::fs::symlink(d.join("gone"), &link).unwrap();

    // `now` far in the future ⇒ every temp on disk is older than 24 h.
    let swept = sweep(std::slice::from_ref(&d.to_path_buf()), i64::MAX / 2);

    assert_eq!(swept.len(), 2, "swept: {swept:?}");
    assert!(swept.contains(&mine) && swept.contains(&other_pid));
    assert!(!mine.exists() && !other_pid.exists());
    assert!(dest.exists(), "the destination is not a leftover");
    assert!(operators.exists(), "a name we do not write is not ours");
    assert!(sub.is_dir(), "a directory is never swept");
    assert!(link.symlink_metadata().is_ok(), "a symlink is never swept");
}

#[test]
fn sweep_keeps_a_fresh_temp_and_shrugs_at_a_missing_dir() {
    let dir = tempdir().unwrap();
    let fresh = temp_in(dir.path(), "ui.json");
    fs::write(&fresh, b"x").unwrap();
    // `now` at the epoch ⇒ nothing on disk is 24 h older than it.
    assert!(sweep(&[dir.path().to_path_buf()], 0).is_empty());
    assert!(fresh.exists());
    // A directory that is not there is a no-op, not an error: yog sweeps
    // destinations it has never written to yet on every boot.
    assert!(sweep(&[dir.path().join("nope")], i64::MAX / 2).is_empty());
}

#[test]
fn dirs_are_the_three_write_sites_destinations() {
    let root = tempdir().unwrap();
    let world = crate::test_support::world::world_under(root.path());
    let wall = crate::test_support::world::wall_paths(root.path());
    // The wall is discovered on disk, so it must exist to be swept.
    fs::create_dir_all(&wall.credentials_dir).unwrap();

    let found = dirs(&world);

    // `ui.json` (§4.1), the §9.2 config root and its workflows/, and the
    // wall's three brazen destinations (§9.1, §16.2).
    for expected in [
        world.yog_state_root(),
        world.lernie_config_root(),
        world.lernie_config_root().join("workflows"),
        wall.config.parent().unwrap().to_path_buf(),
        wall.credentials_dir.clone(),
        wall.models_cache_dir.clone(),
    ] {
        assert!(found.contains(&expected), "{expected:?} not in {found:?}");
    }
}

#[test]
fn dirs_without_a_wall_on_disk_are_the_world_ones_alone() {
    let root = tempdir().unwrap();
    let world = crate::test_support::world::world_under(root.path());
    // Nothing laid: the walls dir does not exist, so no wall contributes and
    // the fold is the three world destinations rather than an error.
    assert_eq!(dirs(&world).len(), 3);
}
