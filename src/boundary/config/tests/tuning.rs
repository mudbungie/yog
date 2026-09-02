//! The §9.4 **tuning pair** through the chokepoint (bl-23bd): the same
//! real-git workspace and recorder `litany` [`knobs`](super::knobs) drives the
//! pick with, aimed at the other two writers of that role assignment.
//!
//! Its own file on the seam the family already has — a pointer gated against
//! brazen's live table beside a knob that is always lawful — and because
//! `knobs` reached §12's cap when the second family arrived.

use super::knobs::{pick, workspace};
use super::{deps_at, fire, quiet, script};
use crate::boundary::Action;
use crate::boundary::reply::Reply;
use crate::model_pick::{Effort, Tuning};
use crate::test_support::TEMPLATE_PROVIDERS;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn tune(tuning: crate::model_pick::Tuning) -> Action {
    Action::Tune(tuning)
}

/// bl-23bd. A tuning gesture is the pick's read → plan → commit, minus the
/// provider table: it stages the same file on the same lineage for the same
/// `litany config`, so the two fail the same way and an operator learns one
/// pipeline. Here the knob is written for the first time, which is the insert
/// arm the grammar had no primitive for.
#[test]
fn a_tuning_gesture_commits_the_same_file_the_pick_does() {
    let root = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let log = bin.path().join("log");
    let litany = script(
        bin.path(),
        "litany",
        &format!(
            "cat \"$YOG_EDIT_SRC/providers.yaml\" > {}\nexit 0\n",
            log.display()
        ),
    );
    let fx = workspace();
    let deps = super::seeing(
        &deps_at(root.path(), &litany, Path::new("/no/bl")),
        &[fx.path.as_path()],
    );
    let reply = fire(
        &deps,
        &tune(Tuning::Effort {
            workspace: crate::naming::leaf(&fx.path),
            role: "worker".to_owned(),
            level: Some(Effort::High),
        }),
    );
    assert!(
        matches!(&reply, Ok(Reply::Outcome(o)) if o.ok()),
        "{reply:?}"
    );
    let staged = fs::read_to_string(&log).unwrap();
    assert!(staged.contains("effort: high"), "{staged}");
    // The pointer it did not come to change is untouched, which is the whole
    // reason this is a sibling gesture and not a wider `/model` — and so is the
    // `tools:` line, which a whole-entry rewrite would have dropped.
    let untouched: Vec<&str> = TEMPLATE_PROVIDERS
        .lines()
        .filter(|l| l.trim_start().starts_with("provider:") || l.trim_start().starts_with("tools:"))
        .collect();
    assert!(!untouched.is_empty(), "the fixture states a pointer");
    for line in untouched {
        assert!(staged.contains(line), "{line} lost from {staged}");
    }
}

/// **No provider table is read at all**, which is what makes the capability a
/// control's question and never a write's: this workspace's wall names no rows,
/// so a pick here would refuse — and the knob lands.
#[test]
fn a_tuning_gesture_reads_no_provider_table_and_so_cannot_be_gated_by_one() {
    let root = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let log = bin.path().join("log");
    let litany = script(
        bin.path(),
        "litany",
        &format!(
            "cat \"$YOG_EDIT_SRC/providers.yaml\" > {}\nexit 0\n",
            log.display()
        ),
    );
    let fx = workspace();
    let deps = super::seeing(
        &deps_at(root.path(), &litany, Path::new("/no/bl")),
        &[fx.path.as_path()],
    );
    let reply = fire(
        &deps,
        &tune(Tuning::Priority {
            workspace: crate::naming::leaf(&fx.path),
            role: "compactor".to_owned(),
            on: true,
        }),
    );
    assert!(
        matches!(&reply, Ok(Reply::Outcome(o)) if o.ok()),
        "{reply:?}"
    );
    assert!(fs::read_to_string(&log).unwrap().contains("priority: true"));
    // The same wall refuses a pick, so the two gestures really do differ here.
    let err = fire(&deps, &pick("worker", "acme", "m-9", &fx.path)).unwrap_err();
    assert!(err.contains("no provider row `acme`"), "{err}");
}

/// A role the lineage does not declare refuses, and nothing is staged — the
/// grammar's own sentence, arriving through the executor unchanged.
#[test]
fn a_tuning_gesture_on_an_undeclared_role_refuses_at_the_executor() {
    let root = tempdir().unwrap();
    let fx = workspace();
    let deps = super::seeing(&quiet(root.path()), &[fx.path.as_path()]);
    let err = fire(
        &deps,
        &tune(Tuning::Effort {
            workspace: crate::naming::leaf(&fx.path),
            role: "nonesuch".to_owned(),
            level: None,
        }),
    )
    .unwrap_err();
    assert!(err.contains("nonesuch"), "{err}");
}

/// It needs a lineage it can read the assignment from, exactly as a pick does.
#[test]
fn a_tuning_gesture_needs_a_lineage_it_can_read() {
    let root = tempdir().unwrap();
    let deps = quiet(root.path());
    let err = fire(
        &deps,
        &tune(Tuning::Priority {
            workspace: crate::naming::leaf(root.path()),
            role: "worker".to_owned(),
            on: false,
        }),
    )
    .unwrap_err();
    assert!(!err.is_empty(), "{err}");
}
