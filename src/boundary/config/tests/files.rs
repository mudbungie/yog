//! Every §9 config *destination*, driven through the chokepoint (bl-3f46): the
//! `bz`-validated brazen write, the provider-gated lernie-global one, the
//! workflow name gate, the clock's own file, and the staged lineage commit —
//! plus the two verdict folds that name every refusal.

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
fn provider_rows_are_asked_of_brazen_at_the_gesture_never_stored() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    assert!(
        deps.provider_rows().iter().any(|r| r == "acme"),
        "{:?}",
        deps.provider_rows()
    );
    // The same `Deps` answers differently once the file does — which is the
    // whole claim: the rows are brazen's fact, read at the moment of use, and
    // the headless "empty table gates nothing" interim is retired.
    fs::write(
        crate::test_support::wall_paths(root.path()).config,
        ACME.replace("acme", "zinc"),
    )
    .unwrap();
    assert!(deps.provider_rows().iter().any(|r| r == "zinc"));
    assert!(!deps.provider_rows().iter().any(|r| r == "acme"));
}

#[test]
fn a_brazen_apply_lands_only_what_bz_accepts() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let good = ACME.replace("acme", "zinc");
    let dest = crate::test_support::wall_paths(root.path()).config;
    assert_eq!(
        fire(&deps, &applying(brazen_file(), &good)),
        Ok(Reply::Applied {
            file: dest.display().to_string()
        })
    );
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

#[test]
fn a_lernie_global_apply_is_gated_on_brazens_effective_rows() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let good = "models:\n  m-1:\n    provider: acme\n";
    assert_eq!(
        fire(&deps, &applying(ConfigFile::LernieModels, good)),
        Ok(Reply::Applied {
            file: root.path().join("lernie/models.yaml").display().to_string()
        })
    );
    assert_eq!(
        fs::read_to_string(root.path().join("lernie/models.yaml")).unwrap(),
        good
    );
    // A model declared on a row brazen does not have is the §9.2 refusal —
    // the failure an operator would otherwise only meet mid-conversation.
    let err = fire(
        &deps,
        &applying(
            ConfigFile::LernieModels,
            "models:\n  m-2:\n    provider: nope\n",
        ),
    )
    .unwrap_err();
    assert!(err.contains("brazen has no provider row for"), "{err}");
    assert!(err.contains("m-2"), "{err}");
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
    // A workflow declares no models, so the provider gate has nothing to say —
    // the general path with nothing to judge, not an exemption.
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
        Ok(Reply::Applied {
            file: root
                .path()
                .join("lernie/workflows/review.yaml")
                .display()
                .to_string()
        })
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
        Ok(Reply::Applied {
            file: dest.display().to_string()
        })
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
    let deps = deps_at(root.path(), &lernie, Path::new("/no/bl"));
    let ws = root.path().join("ws");
    let text = "roles:\n  worker:\n    provider: acme\n";
    let reply = fire(
        &deps,
        &applying(
            ConfigFile::Branch {
                workspace: ws.clone(),
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
    let deps = quiet(root.path());
    // The staging root's own path is a file, so no nonce dir can be made.
    fs::write(deps.world.yog_stage_root(), b"in the way").unwrap();
    let err = fire(
        &deps,
        &applying(
            ConfigFile::Branch {
                workspace: root.path().join("ws"),
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
