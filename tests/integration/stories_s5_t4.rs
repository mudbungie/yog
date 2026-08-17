//! STORIES **S5-T4** hash-guard: a file that changed on disk after load makes
//! Apply refuse rather than blind-LWW the concurrent edit; after a reload the
//! same Apply lands (STORIES S5.3, DESIGN §9.1, §9.2).
//!
//! **The row's premise drifted: there are two doors with this guard, not
//! three.** STORIES says "asserted once per editor (brazen, lernie-global,
//! config branch) — one discipline, three doors". The first two share
//! `config_edit::pipeline`'s snapshot guard, asserted below. The **config
//! branch has no such guard and must not grow one**: yog never writes
//! `config/*` at all — `lernie config` is that tree's only lawful writer (§9.3)
//! and owns its own concurrency — so yog stages a draft and hands it over
//! (S5-T5). Adding a hash guard on this side would be yog claiming an
//! authority over a file it does not own.

#![allow(clippy::unwrap_used)]

use tempfile::tempdir;
use yog::config_edit::RealFileIo;
use yog::config_edit::brazen::{
    Applied, BrazenEditor, BrazenPaths, BzOutcome, BzRunner, ProviderRow,
};
use yog::config_edit::lernie_global::{Editor, Saved};

/// A `bz` that validates anything — this row is about the guard, not the gate.
struct PermissiveBz;

impl BzRunner for PermissiveBz {
    fn dump_config_at(&self, _config: &std::path::Path) -> BzOutcome {
        self.dump_config_effective()
    }
    fn dump_config_effective(&self) -> BzOutcome {
        BzOutcome {
            success: true,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
    fn providers(&self) -> Vec<ProviderRow> {
        Vec::new()
    }
    fn list_models(&self, _provider: &str) -> BzOutcome {
        self.dump_config_effective()
    }
}

/// STORIES **S5-T4** hash-guard — door 1: brazen's `config.toml` (§9.1).
#[test]
fn s5_t4_brazen_apply_refuses_a_file_that_moved_and_lands_after_a_reload() {
    let home = tempdir().unwrap();
    let config = home.path().join("config.toml");
    std::fs::write(&config, "a = 1\n").unwrap();

    let io = RealFileIo;
    let mut editor = BrazenEditor::load(
        BrazenPaths {
            config: config.clone(),
            credentials_dir: home.path().join("creds"),
            models_cache_dir: home.path().join("cache"),
        },
        &io,
    )
    .unwrap();
    editor.set_draft("a = 2\n".to_owned());

    // The other instance — or vi — writes the file underneath the editor.
    std::fs::write(&config, "a = 99\n").unwrap();

    // Apply refuses. It does NOT overwrite: the other writer's bytes stand.
    assert_eq!(editor.apply(&PermissiveBz, &io), Applied::Conflict);
    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        "a = 99\n",
        "a refused Apply never touches the file"
    );

    // Reload re-diffs against what is actually there; the same Apply then lands.
    editor.reload(&io).unwrap();
    editor.set_draft("a = 2\n".to_owned());
    assert_eq!(editor.apply(&PermissiveBz, &io), Applied::Ok);
    assert_eq!(std::fs::read_to_string(&config).unwrap(), "a = 2\n");
}

/// STORIES **S5-T4** hash-guard — door 2: lernie's global raw-text files (§9.2).
/// The same pipeline, so the same refusal, reached through a different editor.
#[test]
fn s5_t4_lernie_global_apply_refuses_a_file_that_moved_and_lands_after_a_reload() {
    let home = tempdir().unwrap();
    let models = home.path().join("models.yaml");
    std::fs::write(&models, "models: []\n").unwrap();

    let io = RealFileIo;
    let mut editor = Editor::load(models.clone(), &io).unwrap();
    editor.set_draft("models: [mine]\n".to_owned());

    std::fs::write(&models, "models: [theirs]\n").unwrap();

    // The guard is the only thing that can refuse here — this editor judges no
    // content at all since bl-3ffa (§9.2).
    assert_eq!(editor.apply(&io), Saved::Conflict);
    assert_eq!(
        std::fs::read_to_string(&models).unwrap(),
        "models: [theirs]\n"
    );

    editor.reload(&io).unwrap();
    editor.set_draft("models: [mine]\n".to_owned());
    assert_eq!(editor.apply(&io), Saved::Ok);
    assert_eq!(
        std::fs::read_to_string(&models).unwrap(),
        "models: [mine]\n"
    );
}

/// The guard's other face on door 2: creating a file that appeared underneath
/// you is the *same* refusal, not a special case — `loaded` is `None`, so any
/// file now present is a mismatch.
#[test]
fn s5_t4_creating_a_file_that_now_exists_is_the_same_refusal() {
    let home = tempdir().unwrap();
    let path = home.path().join("workflows").join("new.yaml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let io = RealFileIo;
    let mut editor = Editor::seeded(path.clone(), b"steps: []\n");
    assert!(editor.is_new());

    std::fs::write(&path, "someone: else\n").unwrap();
    assert_eq!(
        editor.apply(&io),
        Saved::Conflict,
        "must-not-exist is the empty case of the same guard"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "someone: else\n");
}
