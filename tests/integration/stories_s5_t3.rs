//! STORIES **S5-T3** brazen-validate-rejects: a staged buffer whose
//! `bz --config <temp> --dump-config` exits non-zero never lands — the
//! destination file is byte-identical afterwards, no temp survives, the stderr
//! renders, and the draft is kept (STORIES S5.3, DESIGN §9.1).
//!
//! "A malformed brazen config cannot land — `bz` itself rejects the staged file
//! and the draft survives in the box." The validation is deliberately *not*
//! yog's: yog owns no TOML judgement, it stages bytes and asks the tool that
//! will have to read them.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use tempfile::tempdir;
use yog::config_edit::RealFileIo;
use yog::config_edit::brazen::{
    Applied, BrazenEditor, BrazenPaths, BzOutcome, BzRunner, ProviderRow,
};

/// The good config on disk before the operator types anything.
const ON_DISK: &str = "[providers.anthropic]\nkey_env = \"ANTHROPIC_API_KEY\"\n";
/// What the operator typed: not TOML at all.
const MALFORMED: &str = "[providers.anthropic\nkey_env =\n";
/// bz's verdict on it, which is the only judgement in this flow.
const VERDICT: &str = "invalid TOML at line 1: expected `]`";

/// A `bz` that refuses whatever it is handed, recording the config path it was
/// asked to validate so the test can prove it judged the **staged temp**, not
/// the destination.
struct RefusingBz {
    seen: std::cell::Cell<Option<std::path::PathBuf>>,
}

impl BzRunner for RefusingBz {
    fn dump_config_at(&self, config: &Path) -> BzOutcome {
        self.seen.set(Some(config.to_path_buf()));
        BzOutcome {
            success: false,
            stdout: String::new(),
            stderr: VERDICT.to_owned(),
        }
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

/// Every file in `dir`, sorted — the "no temp survives" reading.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// STORIES **S5-T3** brazen-validate-rejects.
#[test]
fn s5_t3_a_config_bz_refuses_never_lands_and_the_draft_survives() {
    let home = tempdir().unwrap();
    let dir = home.path().join("brazen");
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("config.toml");
    std::fs::write(&config, ON_DISK).unwrap();

    let io = RealFileIo;
    let paths = BrazenPaths {
        config: config.clone(),
        credentials_dir: dir.join("credentials"),
        models_cache_dir: dir.join("cache"),
    };
    let mut editor = BrazenEditor::load(paths, &io).unwrap();
    editor.set_draft(MALFORMED.to_owned());

    let bz = RefusingBz {
        seen: std::cell::Cell::new(None),
    };
    let applied = editor.apply(&bz, &io);

    // The stderr renders verbatim — the operator is told what bz objected to,
    // not that "something went wrong".
    assert_eq!(
        applied,
        Applied::Rejected {
            stderr: VERDICT.to_owned()
        }
    );

    // The destination is byte-identical: a refused Apply is a no-op on disk.
    assert_eq!(std::fs::read_to_string(&config).unwrap(), ON_DISK);
    // And no temp survives it — the staged bytes are cleaned up on the reject
    // path, not left beside the config for the next reader to find.
    assert_eq!(entries(&dir), ["config.toml"], "no staging debris");
    // bz judged the STAGED file, never the destination: the whole point of
    // staging is that the candidate is validated before it is anywhere real.
    let judged = bz.seen.take().expect("bz was asked");
    assert_ne!(judged, config, "the temp was judged, not the live file");
    assert_eq!(judged.parent(), config.parent(), "staged beside its target");
    assert!(!judged.exists(), "and gone afterwards");

    // The draft is kept in the box: the operator's typing is not thrown away
    // because the tool said no. Applying again reaches bz again with the same
    // bytes, which is only possible if they survived.
    let again = editor.apply(&bz, &io);
    assert_eq!(
        again,
        Applied::Rejected {
            stderr: VERDICT.to_owned()
        },
        "the draft survived the first refusal"
    );
    assert_eq!(std::fs::read_to_string(&config).unwrap(), ON_DISK);
}
