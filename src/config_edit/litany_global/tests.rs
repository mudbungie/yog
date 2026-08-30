//! View-model transitions driven by the shared in-memory [`FakeFs`] — no real
//! disk. Enumeration, name validation, load new-vs-existing, the copy-from and
//! must-not-exist affordances, and every [`Saved`] Apply outcome.
//!
//! Two tests went with the §9.2 provider gate (bl-3ffa): the refusal of a model
//! on a row brazen lacked, and the proof that the same gate ran clean over a
//! workflow file. Apply judges no content now, so there is no third outcome to
//! drive and no per-file difference to prove — every test here holds for every
//! file the editor opens.

use super::*;
use crate::test_support::FakeFs;

fn root() -> PathBuf {
    PathBuf::from("/cfg/litany")
}

fn lg() -> LitanyGlobal {
    LitanyGlobal { root: root() }
}

fn wf(name: &str) -> PathBuf {
    root().join("workflows").join(name)
}

#[test]
fn resolve_folds_root_from_env_and_litany_home_collapse() {
    let xdg = Env::from_pairs([("HOME", "/home/u"), ("XDG_CONFIG_HOME", "/home/u/.config")]);
    assert_eq!(
        LitanyGlobal::resolve(&xdg).models(),
        PathBuf::from("/home/u/.config/litany/models.yaml")
    );
    // LITANY_HOME collapses both roots onto one dir.
    let home = Env::from_pairs([("HOME", "/home/u"), ("LITANY_HOME", "/srv/litany")]);
    assert_eq!(
        LitanyGlobal::resolve(&home).workflows_dir(),
        PathBuf::from("/srv/litany/workflows")
    );
}

#[test]
fn workflows_lists_only_yaml_sorted_and_absent_is_empty() {
    // Missing workflows/ dir → empty list, not an error.
    assert!(lg().workflows(&FakeFs::default()).unwrap().is_empty());
    let fs = FakeFs::default();
    for (name, body) in [
        ("b.yaml", "2"),
        ("a.yaml", "1"),
        ("notes.txt", "x"),
        ("README", "y"),
    ] {
        fs.map().insert(wf(name), body.into());
    }
    // Also a stray file directly under the root — not a workflow.
    fs.map().insert(lg().models(), b"m".to_vec());
    assert_eq!(
        lg().workflows(&fs).unwrap(),
        vec![wf("a.yaml"), wf("b.yaml")]
    );
}

#[test]
fn new_workflow_validates_the_name() {
    assert_eq!(lg().new_workflow("deploy").unwrap(), wf("deploy.yaml"));
    assert_eq!(lg().new_workflow(""), Err(WorkflowNameError::Empty));
    assert_eq!(
        lg().new_workflow(".hidden"),
        Err(WorkflowNameError::DotLeading)
    );
    assert_eq!(lg().new_workflow("a/b"), Err(WorkflowNameError::Separator));
    assert_eq!(lg().new_workflow("a\\b"), Err(WorkflowNameError::Separator));
}

#[test]
fn load_distinguishes_existing_from_new() {
    let existing = FakeFs::seed(&lg().models(), b"provider: x\n");
    let ed = Editor::load(lg().models(), &existing).unwrap();
    assert_eq!(ed.draft(), "provider: x\n");
    assert!(!ed.is_new());
    assert_eq!(ed.path(), lg().models());
    // A missing models.yaml is a meaningful new-file edit, not an error.
    let ed = Editor::load(lg().models(), &FakeFs::default()).unwrap();
    assert_eq!(ed.draft(), "");
    assert!(ed.is_new());
}

#[test]
fn draft_mut_and_set_draft_edit_the_buffer() {
    let mut ed = Editor::load(lg().models(), &FakeFs::default()).unwrap();
    ed.draft_mut().push_str("k: v");
    assert_eq!(ed.draft(), "k: v");
    ed.set_draft("k: w".into());
    assert_eq!(ed.draft(), "k: w");
}

#[test]
fn apply_edits_an_existing_file_and_tracks_the_snapshot() {
    let fs = FakeFs::seed(&lg().models(), b"a");
    let mut ed = Editor::load(lg().models(), &fs).unwrap();
    ed.set_draft("b".into());
    assert_eq!(ed.apply(&fs), Saved::Ok);
    assert_eq!(fs.get(&lg().models()), Some(b"b".to_vec()));
    // A repeat Apply lands — the loaded snapshot tracked the rename.
    ed.set_draft("c".into());
    assert_eq!(ed.apply(&fs), Saved::Ok);
    assert_eq!(fs.get(&lg().models()), Some(b"c".to_vec()));
}

#[test]
fn seeded_creates_a_new_workflow_guarded_must_not_exist() {
    let fs = FakeFs::default();
    let path = lg().new_workflow("deploy").unwrap();
    let mut ed = Editor::seeded(path.clone(), b"steps: []\n");
    assert!(ed.is_new());
    assert_eq!(ed.draft(), "steps: []\n");
    assert_eq!(ed.apply(&fs), Saved::Ok);
    assert_eq!(fs.get(&path), Some(b"steps: []\n".to_vec()));
    // Now it exists; is_new flips.
    assert!(!ed.is_new());
    // A second seeded editor for the same name refuses — must-not-exist.
    let mut clash = Editor::seeded(path.clone(), b"other\n");
    assert_eq!(clash.apply(&fs), Saved::Conflict);
    assert_eq!(fs.get(&path), Some(b"steps: []\n".to_vec()));
}

#[test]
fn seeded_copies_from_existing_workflow_bytes() {
    // Copy-from-existing: seed a new draft from another file's loaded bytes.
    let src = FakeFs::seed(&wf("base.yaml"), b"shared: true\n");
    let bytes = src.get(&wf("base.yaml")).unwrap();
    let ed = Editor::seeded(wf("copy.yaml"), &bytes);
    assert_eq!(ed.draft(), "shared: true\n");
    assert!(ed.is_new());
}

#[test]
fn apply_refuses_a_concurrent_edit_and_reload_recovers() {
    let fs = FakeFs::seed(&lg().models(), b"a");
    let mut ed = Editor::load(lg().models(), &fs).unwrap();
    ed.set_draft("mine".into());
    // Another writer changes the file after load.
    fs.map().insert(lg().models(), b"theirs".to_vec());
    assert_eq!(ed.apply(&fs), Saved::Conflict);
    assert_eq!(fs.get(&lg().models()), Some(b"theirs".to_vec()));
    // Reload re-diffs against the concurrent content; Apply then lands.
    ed.reload(&fs).unwrap();
    assert_eq!(ed.draft(), "theirs");
    ed.set_draft("merged".into());
    assert_eq!(ed.apply(&fs), Saved::Ok);
}

#[test]
fn refresh_follows_disk_only_while_the_draft_is_pristine() {
    let fs = FakeFs::seed(&lg().models(), b"a");
    let mut ed = Editor::load(lg().models(), &fs).unwrap();
    fs.map().insert(lg().models(), b"outside".to_vec());
    assert!(ed.refresh(&fs).unwrap());
    assert_eq!(ed.draft(), "outside");
    // A typed draft is never discarded.
    ed.set_draft("mine".into());
    fs.map().insert(lg().models(), b"theirs".to_vec());
    assert!(!ed.refresh(&fs).unwrap());
    assert_eq!(ed.draft(), "mine");
    // Nor is a seeded new-file draft, whose text was authored, not read.
    let mut fresh = Editor::seeded(wf("new.yaml"), b"steps: []\n");
    fs.map().insert(wf("new.yaml"), b"squatter".to_vec());
    assert!(!fresh.refresh(&fs).unwrap());
    assert_eq!(fresh.draft(), "steps: []\n");
}

#[test]
fn apply_maps_a_filesystem_error_to_io() {
    let fs = FakeFs {
        fail_write: true,
        ..FakeFs::default()
    };
    let mut ed = Editor::load(lg().models(), &fs).unwrap();
    ed.set_draft("x".into());
    assert!(matches!(ed.apply(&fs), Saved::Io { .. }));
}
