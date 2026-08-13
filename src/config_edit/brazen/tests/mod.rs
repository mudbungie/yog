//! View-model transitions driven by a fake [`BzRunner`]/[`FileIo`] pair —
//! no live `bz`, no real disk. Every Apply terminal state (§9.1) and every
//! derived read has a test here.
//!
//! Split at the surface seam: this file is the **write path** — load, the RAM
//! draft, and every Apply terminal state. [`reads`] holds the read-only
//! derived surfaces (§5.1 rows 20/22/23), which share this fixture but touch
//! no pipeline.

mod reads;

use super::*;
use crate::test_support::FakeFs;
use std::path::PathBuf;
use std::sync::Mutex;

/// A runner that returns one preset outcome (and one preset provider table)
/// and logs each call.
struct FakeRunner {
    outcome: BzOutcome,
    providers: Vec<ProviderRow>,
    log: Mutex<Vec<String>>,
}

impl FakeRunner {
    fn ok() -> Self {
        Self::with(true, "", "")
    }
    fn with(success: bool, stdout: &str, stderr: &str) -> Self {
        Self {
            outcome: BzOutcome {
                success,
                stdout: stdout.into(),
                stderr: stderr.into(),
            },
            providers: Vec::new(),
            log: Mutex::new(Vec::new()),
        }
    }

    /// The same fake with a preset effective provider table (keyless rows —
    /// only the `name` column is read here).
    fn listing(names: &[&str]) -> Self {
        Self {
            providers: names
                .iter()
                .map(|n| ProviderRow {
                    name: (*n).to_string(),
                    auth: "none".to_owned(),
                })
                .collect(),
            ..Self::ok()
        }
    }

    /// The call log, locked (poison-immune).
    fn log(&self) -> std::sync::MutexGuard<'_, Vec<String>> {
        self.log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl BzRunner for FakeRunner {
    fn dump_config_at(&self, _config: &Path) -> BzOutcome {
        self.log().push("at".into());
        self.outcome.clone()
    }
    fn dump_config_effective(&self) -> BzOutcome {
        self.log().push("effective".into());
        self.outcome.clone()
    }
    fn providers(&self) -> Vec<ProviderRow> {
        self.log().push("providers".into());
        self.providers.clone()
    }
    fn list_models(&self, provider: &str) -> BzOutcome {
        self.log().push(format!("list:{provider}"));
        self.outcome.clone()
    }
}

fn cfg() -> PathBuf {
    PathBuf::from("/cfg/brazen/config.toml")
}

fn paths() -> BrazenPaths {
    BrazenPaths {
        config: cfg(),
        credentials_dir: PathBuf::from("/creds"),
        models_cache_dir: PathBuf::from("/cache/models"),
    }
}

fn loaded(fs: &FakeFs) -> BrazenEditor {
    BrazenEditor::load(paths(), fs).unwrap()
}

#[test]
fn load_missing_file_is_empty_draft() {
    let ed = loaded(&FakeFs::default());
    assert_eq!(ed.draft(), "");
}

#[test]
fn load_reads_existing_bytes_into_draft() {
    let ed = loaded(&FakeFs::seed(&cfg(), b"name = \"openai\"\n"));
    assert_eq!(ed.draft(), "name = \"openai\"\n");
}

#[test]
fn draft_mut_and_set_draft_edit_the_buffer() {
    let mut ed = loaded(&FakeFs::default());
    ed.draft_mut().push_str("x = 1");
    assert_eq!(ed.draft(), "x = 1");
    ed.set_draft("y = 2".into());
    assert_eq!(ed.draft(), "y = 2");
}

#[test]
fn apply_ok_validates_guards_and_renames() {
    let fs = FakeFs::seed(&cfg(), b"A");
    let mut ed = loaded(&fs);
    ed.set_draft("B".into());
    let runner = FakeRunner::ok();
    assert_eq!(ed.apply(&runner, &fs), Applied::Ok);
    // The draft is now the on-disk content.
    assert_eq!(fs.get(&cfg()), Some(b"B".to_vec()));
    // A repeat Apply succeeds — the loaded hash tracked the rename.
    ed.set_draft("C".into());
    assert_eq!(ed.apply(&runner, &fs), Applied::Ok);
    assert_eq!(fs.get(&cfg()), Some(b"C".to_vec()));
}

#[test]
fn apply_rejects_on_nonzero_bz_exit_keeping_draft() {
    let fs = FakeFs::seed(&cfg(), b"A");
    let mut ed = loaded(&fs);
    ed.set_draft("bad".into());
    let runner = FakeRunner::with(false, "", "MalformedFile at line 3");
    assert_eq!(
        ed.apply(&runner, &fs),
        Applied::Rejected {
            stderr: "MalformedFile at line 3".into()
        }
    );
    // Draft kept, real file untouched.
    assert_eq!(ed.draft(), "bad");
    assert_eq!(fs.get(&cfg()), Some(b"A".to_vec()));
}

#[test]
fn apply_refuses_on_concurrent_edit_with_conflict() {
    let fs = FakeFs::seed(&cfg(), b"A");
    let mut ed = loaded(&fs);
    ed.set_draft("B".into());
    // Another writer changes the file after load.
    fs.map().insert(cfg(), b"C".to_vec());
    assert_eq!(ed.apply(&FakeRunner::ok(), &fs), Applied::Conflict);
    // The concurrent content is preserved.
    assert_eq!(fs.get(&cfg()), Some(b"C".to_vec()));
}

#[test]
fn apply_maps_a_filesystem_error_to_io() {
    let fs = FakeFs {
        fail_write: true,
        ..FakeFs::default()
    };
    let mut ed = loaded(&fs);
    ed.set_draft("B".into());
    assert!(matches!(
        ed.apply(&FakeRunner::ok(), &fs),
        Applied::Io { .. }
    ));
}

#[test]
fn reload_recovers_the_on_disk_content_and_hash() {
    let fs = FakeFs::seed(&cfg(), b"A");
    let mut ed = loaded(&fs);
    ed.set_draft("stale".into());
    fs.map().insert(cfg(), b"D".to_vec());
    ed.reload(&fs).unwrap();
    assert_eq!(ed.draft(), "D");
    // After reload the guard matches the new content, so Apply lands.
    ed.set_draft("E".into());
    assert_eq!(ed.apply(&FakeRunner::ok(), &fs), Applied::Ok);
}

#[test]
fn refresh_follows_disk_only_while_the_draft_is_pristine() {
    let fs = FakeFs::seed(&cfg(), b"A");
    let mut ed = loaded(&fs);
    // An untouched draft follows an edit made outside yog (§9, bl-9130).
    fs.map().insert(cfg(), b"outside = 1".to_vec());
    assert!(ed.refresh(&fs).unwrap());
    assert_eq!(ed.draft(), "outside = 1");
    // Having re-read, the guard matches, so Apply lands with no reload.
    ed.set_draft("mine".into());
    assert_eq!(ed.apply(&FakeRunner::ok(), &fs), Applied::Ok);
    // An edited draft is left exactly as typed; the hash guard is its answer.
    ed.set_draft("unsaved".into());
    fs.map().insert(cfg(), b"theirs".to_vec());
    assert!(!ed.refresh(&fs).unwrap());
    assert_eq!(ed.draft(), "unsaved");
    assert_eq!(ed.apply(&FakeRunner::ok(), &fs), Applied::Conflict);
}
