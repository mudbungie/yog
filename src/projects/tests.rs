use super::*;
use std::fs;
use tempfile::tempdir;

/// Percent-encode a path's `/` so it round-trips through a single clone
/// basename (the only escape these fixtures need).
fn enc(path: &str) -> String {
    path.replace('/', "%2F")
}

#[test]
fn delivery_root_is_the_state_roots_bl_delivery_tree() {
    assert_eq!(
        delivery_root(Path::new("/s/balls/clones")),
        Some(PathBuf::from("/s/balls/plugins/bl-delivery")),
    );
    // A rootless clones dir has no state root, so nothing can be internal.
    assert_eq!(delivery_root(Path::new("/")), None);
}

#[test]
fn is_internal_matches_only_under_the_delivery_tree() {
    let d = PathBuf::from("/s/balls/plugins/bl-delivery");
    assert!(is_internal(
        Path::new("/s/balls/plugins/bl-delivery/home/u/p/bl-1"),
        Some(&d),
    ));
    assert!(!is_internal(Path::new("/home/u/p"), Some(&d)));
    // No delivery root ⇒ never internal.
    assert!(!is_internal(Path::new("/anything"), None));
}

#[test]
fn missing_clones_dir_enumerates_empty() {
    let dir = tempdir().unwrap();
    assert!(enumerate(&dir.path().join("absent")).is_empty());
}

#[test]
fn enumerate_decodes_flags_internal_and_skips_non_dirs() {
    let root = tempdir().unwrap();
    let clones = root.path().join("balls").join("clones");
    fs::create_dir_all(&clones).unwrap();
    let delivery = root.path().join("balls/plugins/bl-delivery");

    // A normal project clone and an internal (nested-delivery) one.
    fs::create_dir(clones.join(enc("/home/u/proj"))).unwrap();
    let internal_path = format!("{}/home/u/proj/bl-1", delivery.display());
    fs::create_dir(clones.join(enc(&internal_path))).unwrap();
    // A regular file (not a dir) is skipped.
    fs::write(clones.join("stray-file"), b"x").unwrap();

    let projects = enumerate(&clones);
    assert_eq!(projects.len(), 2, "non-dir entry skipped");
    let normal = projects.iter().find(|p| !p.internal).unwrap();
    assert_eq!(normal.path, PathBuf::from("/home/u/proj"));
    let internal = projects.iter().find(|p| p.internal).unwrap();
    assert_eq!(internal.path, PathBuf::from(internal_path));
    // Sorted by decoded path (deterministic roster order).
    let mut sorted = projects.clone();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(projects, sorted);
}

/// A clone dir whose basename is not valid UTF-8 has no `to_str`, so
/// [`enumerate`] skips it (mod.rs `to_str` guard). The fixture is
/// non-macOS-only: APFS enforces valid-UTF-8 filenames and refuses the
/// creation (EILSEQ), so the input is simply unconstructible on macOS; on
/// ext4 — where tarpaulin runs — the guard is still exercised, so coverage
/// is unaffected.
#[cfg(not(target_os = "macos"))]
#[test]
fn enumerate_skips_non_utf8_basenames() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let clones = tempdir().unwrap();
    fs::create_dir(clones.path().join(enc("/home/u/proj"))).unwrap();
    fs::create_dir(clones.path().join(OsStr::from_bytes(&[0x66, 0x80]))).unwrap();

    let projects = enumerate(clones.path());
    assert_eq!(projects.len(), 1, "non-UTF-8 basename skipped");
    assert_eq!(projects[0].path, PathBuf::from("/home/u/proj"));
}

/// A project's roster label is its **basename** (§11, bl-ac3d): the absolute
/// path used to be the label, and it sized the whole left panel to itself.
#[test]
fn a_label_is_the_basename() {
    let paths = [
        PathBuf::from("/home/u/dev/yog"),
        PathBuf::from("/tmp/scratch-9f2/lernie"),
    ];
    assert_eq!(labels(&paths), vec!["yog".to_owned(), "lernie".to_owned()]);
}

/// Two projects that end in the same name are not one project: the label
/// extends leftward — the shortest trailing run that tells them apart — so the
/// row never reads alike for two different mint targets. Only the colliding
/// pair pays; a unique neighbour keeps its bare basename.
#[test]
fn colliding_basenames_extend_leftward_until_they_differ() {
    let paths = [
        PathBuf::from("/home/u/work/yog"),
        PathBuf::from("/home/u/play/yog"),
        PathBuf::from("/home/u/play/lernie"),
    ];
    assert_eq!(
        labels(&paths),
        vec![
            "work/yog".to_owned(),
            "play/yog".to_owned(),
            "lernie".to_owned(),
        ]
    );
}

/// The run extends only as far as it must — two projects that differ at their
/// first component separate there, without the leading `/` a whole-path render
/// would carry. Nothing separates a path from itself, so the fallback is that
/// whole path: a set holding one project twice labels it alike, which is right.
#[test]
fn a_label_extends_no_further_than_it_must_and_a_repeat_labels_alike() {
    let paths = [PathBuf::from("/a/p"), PathBuf::from("/b/p")];
    assert_eq!(labels(&paths), vec!["a/p".to_owned(), "b/p".to_owned()]);
    let twice = [PathBuf::from("/a/p"), PathBuf::from("/a/p")];
    assert_eq!(labels(&twice), vec!["/a/p".to_owned(), "/a/p".to_owned()]);
}

/// A label that runs on elides at [`LABEL_MAX`] — the roster is a column of
/// names, and the full path is one hover away.
#[test]
fn a_runaway_label_elides() {
    let long = format!("/home/u/{}", "x".repeat(200));
    let got = labels(&[PathBuf::from(&long)]);
    assert_eq!(got[0].chars().count(), LABEL_MAX, "capped");
    assert!(got[0].ends_with('…'), "marked: {}", got[0]);
    assert!(got[0].starts_with("xxx"), "head kept: {}", got[0]);
    // At the cap exactly, nothing is elided.
    let exact = PathBuf::from(format!("/home/u/{}", "y".repeat(LABEL_MAX)));
    assert_eq!(labels(&[exact])[0], "y".repeat(LABEL_MAX));
}

/// An empty roster labels nothing (the general path with no inputs).
#[test]
fn no_projects_label_nothing() {
    assert!(labels(&[]).is_empty());
}

#[test]
fn visible_always_filters_internal_clones() {
    // bl-e3e7: there is no longer a toggle to reveal them. A nested-delivery
    // clone is a torn-down worktree's throwaway store, never a project.
    let projects = vec![
        Project {
            path: PathBuf::from("/a"),
            internal: false,
        },
        Project {
            path: PathBuf::from("/b"),
            internal: true,
        },
    ];
    let shown: Vec<_> = visible(&projects).iter().map(|p| &p.path).collect();
    assert_eq!(shown, vec![&PathBuf::from("/a")]);
}
