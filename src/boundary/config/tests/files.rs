//! Every §9 config *destination*, driven through the chokepoint (bl-3f46): the
//! `bz`-validated brazen write, the unjudged lernie-global one (bl-3ffa retired
//! its provider gate), the workflow-name gate, the clock's own file, and the
//! staged lineage commit — plus the two verdict folds that name every refusal.

use super::{ACME, applying, brazen_file, deps_at, fire, quiet, script};
use crate::actions::verbs::Outcome;
use crate::boundary::config::{ConfigFile, write};
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::brazen::Applied;
use crate::config_edit::lernie_global::Saved;
use crate::test_support::{spawn_guard, world_under};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn a_brazen_apply_lands_only_what_bz_accepts() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let good = ACME.replace("acme", "zinc");
    let dest = crate::test_support::wall_paths(root.path()).config;
    assert_eq!(
        fire(&deps, &applying(brazen_file(), &good)),
        Ok(Reply::Applied)
    );
    // The receipt says only that it landed (REMOTE §8, bl-ccf7); *where* is the
    // destination's own fact, and the file on disk is what proves it.
    assert_eq!(fs::read_to_string(&dest).unwrap(), good);
    // A draft bz refuses never lands, and its own stderr is the reason.
    let err = fire(&deps, &applying(brazen_file(), "not toml = = =\n")).unwrap_err();
    assert!(err.contains("bz refused the draft"), "{err}");
    assert_eq!(fs::read_to_string(&dest).unwrap(), good, "left untouched");
}

#[test]
fn a_brazen_apply_that_cannot_write_says_so() {
    let root = tempdir().unwrap();
    // A world whose brazen config sits under a directory that does not exist:
    // the read is absence (a value), the staged write is a real failure.
    let deps = Deps {
        world: world_under(&root.path().join("gone")),
        ..quiet(root.path())
    };
    let err = fire(
        &deps,
        &applying(brazen_file(), &ACME.replace("acme", "zinc")),
    )
    .unwrap_err();
    assert!(!err.is_empty(), "the io error rides back verbatim");
}

/// The §9.2 destination lands the bytes it is handed and judges none of them
/// (bl-3ffa): the gate that read `models.<id>.provider` is retired with the
/// field's last reader, so a row brazen has never heard of is the operator's
/// business, exactly as it is in `vi`.
#[test]
fn a_lernie_global_apply_lands_the_text_it_is_handed() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let text = "models:\n  m-1:\n    context_window: 200000\n";
    assert_eq!(
        fire(&deps, &applying(ConfigFile::LernieModels, text)),
        Ok(Reply::Applied)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("lernie/models.yaml")).unwrap(),
        text
    );
    // A legacy entry naming a row brazen does not have lands too — refusing it
    // would refuse an Apply that is correcting the one line anything reads.
    let legacy = "models:\n  m-2:\n    provider: nope\n    context_window: 400000\n";
    assert_eq!(
        fire(&deps, &applying(ConfigFile::LernieModels, legacy)),
        Ok(Reply::Applied)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("lernie/models.yaml")).unwrap(),
        legacy
    );
}

#[test]
fn a_workflow_destination_must_name_one_file_and_lands_under_workflows() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let err = fire(
        &deps,
        &applying(
            ConfigFile::LernieWorkflow {
                name: "a/b".to_owned(),
            },
            "events: {}\n",
        ),
    )
    .unwrap_err();
    assert_eq!(err, "name must be a single file, no path");
    // A safe name lands under `workflows/`, on the same unjudged pipeline every
    // other file destination runs.
    assert_eq!(
        fire(
            &deps,
            &applying(
                ConfigFile::LernieWorkflow {
                    name: "review".to_owned(),
                },
                "events: {}\n",
            ),
        ),
        Ok(Reply::Applied)
    );
    assert_eq!(
        fs::read_to_string(root.path().join("lernie/workflows/review.yaml")).unwrap(),
        "events: {}\n"
    );
}

#[test]
fn a_cadence_apply_writes_the_clocks_own_file() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let text = "sweep_secs: 3\n";
    let dest = write::cadence_path(&deps.world);
    assert_eq!(
        fire(&deps, &applying(ConfigFile::Cadence, text)),
        Ok(Reply::Applied)
    );
    assert_eq!(fs::read_to_string(&dest).unwrap(), text);
}

#[test]
fn an_unreadable_file_refuses_before_anything_is_staged() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    // A directory where the file should be: the read is a real error, not
    // absence, so the editor refuses to load.
    fs::create_dir_all(root.path().join("lernie/models.yaml")).unwrap();
    assert!(fire(&deps, &applying(ConfigFile::LernieModels, "x: 1\n")).is_err());
}

#[test]
fn a_lineage_apply_stages_the_text_and_drives_lernie_config() {
    let g = spawn_guard();
    let root = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let log = bin.path().join("log");
    // The recorder copies the staged file out, so the test can prove the text
    // reached the shim's source dir — which is what lernie's $EDITOR reads.
    let lernie = script(
        bin.path(),
        "lernie",
        &format!(
            "cat \"$YOG_EDIT_SRC/providers.yaml\" > {}\nprintf 'ok'\nexit 0\n",
            log.display()
        ),
    );
    let ws = root.path().join("sphere");
    let deps = super::seeing(
        &deps_at(root.path(), &lernie, Path::new("/no/bl")),
        &[ws.as_path()],
    );
    let text = "roles:\n  worker:\n    provider: acme\n";
    let reply = fire(
        &deps,
        &applying(
            ConfigFile::Branch {
                workspace: crate::naming::leaf(&ws),
                lineage: "default".to_owned(),
                origin: EditOrigin::Advance,
                path: "providers.yaml".to_owned(),
            },
            text,
        ),
    );
    drop(g);
    assert_eq!(
        reply,
        Ok(Reply::Outcome(Outcome {
            exit: 0,
            stdout: "ok".to_owned(),
            stderr: String::new(),
        }))
    );
    assert_eq!(fs::read_to_string(&log).unwrap(), text);
    // The drive's own ops row is the audit half (§4.2).
    let logged = crate::opslog::tail(&deps.state_root, 10);
    assert_eq!(logged.len(), 1);
    assert_eq!(
        &logged[0].argv[1..],
        ["config", &ws.display().to_string(), "default"]
    );
}

#[test]
fn a_lineage_apply_that_cannot_stage_says_so() {
    let root = tempdir().unwrap();
    let ws = root.path().join("sphere");
    let deps = super::seeing(&quiet(root.path()), &[ws.as_path()]);
    // The staging root's own path is a file, so no nonce dir can be made.
    fs::write(deps.world.yog_stage_root(), b"in the way").unwrap();
    let err = fire(
        &deps,
        &applying(
            ConfigFile::Branch {
                workspace: crate::naming::leaf(&ws),
                lineage: "default".to_owned(),
                origin: EditOrigin::Orphan,
                path: "workflow.yaml".to_owned(),
            },
            "events: {}\n",
        ),
    )
    .unwrap_err();
    assert!(!err.is_empty(), "{err}");
}

#[test]
fn the_two_verdict_folds_name_every_refusal() {
    assert_eq!(write::applied(Applied::Ok), Ok(()));
    assert_eq!(
        write::applied(Applied::Conflict),
        Err(write::CONFLICT.to_owned())
    );
    assert_eq!(
        write::applied(Applied::Io {
            error: "disk".to_owned()
        }),
        Err("disk".to_owned())
    );
    assert_eq!(
        write::saved(Saved::Conflict),
        Err(write::CONFLICT.to_owned())
    );
    assert_eq!(
        write::saved(Saved::Io {
            error: "disk".to_owned()
        }),
        Err("disk".to_owned())
    );
}
