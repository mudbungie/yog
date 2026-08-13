//! STORIES **S4-T1** new-workspace-verb: the explicit New workspace verb
//! (§3.4/§11) raises a sphere wall under the **operator's own typed name** —
//! validated first (§3.1: shape, ≤32 bytes, the reserved `unknown`, and a leaf
//! collision under any of the three roots), then `lernie new <names-root>/<name>`
//! through the same start planner. A refused name spawns nothing at all: yog
//! never reaches the planner, so there is no half-raised wall to converge away
//! (DESIGN §3.1/§3.4/§8.1).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::{TempDir, tempdir};
use yog::binding::{names_root, roots, workspace_path};
use yog::cli_outbound::Cli;
use yog::names::{self, NameError};
use yog::start::{self, Deps, Payload, StartInputs};

/// Raise a workspace named `typed`: validate against the world's three roots
/// (§3.1) and, only if lawful, run the New-workspace start. Returns the
/// validated name and the `lernie` argv sequence (prime, new).
fn new_workspace(
    yog: &TempDir,
    lernie_data: &TempDir,
    typed: &str,
) -> Result<(String, Vec<Vec<String>>), NameError> {
    let name = names::validate(typed, &roots(yog.path(), lernie_data.path()))?;
    let (bin, state, balls, home) = (
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
    );
    let lernie = Recorder::new(bin.path(), "lernie").authoring_workspaces();
    let bl = Recorder::new(bin.path(), "bl");
    let deps = Deps {
        bl: Cli::new(bl.path()),
        lernie: Cli::new(lernie.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = StartInputs {
        // The raise's whole content: the operator's name under the flat names
        // root. There is nothing else to decide (§3.1 — the dir is the registry).
        workspace: workspace_path(yog.path(), &name),
        payload: Payload::Bare,
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        // A just-raised workspace has no conversations yet (§3.3): the occupied
        // set for the conversation mint is empty by construction, not by a case.
        conversation_names: Vec::new(),
    };
    let prepared = start::prepare(&deps, &inputs, "T").unwrap();
    assert_eq!(
        prepared.name, name,
        "the name is the leaf, not a second fact"
    );
    assert_eq!(prepared.workspace, workspace_path(yog.path(), &name));
    assert!(prepared.workspace.starts_with(names_root(yog.path())));
    assert!(bl.invocations().is_empty(), "a raise mutates no ball");
    let argv: Vec<Vec<String>> = lernie.invocations().into_iter().map(|i| i.argv).collect();
    Ok((name, argv))
}

/// STORIES **S4-T1** new-workspace-verb.
#[test]
fn s4_t1_new_workspace_takes_the_operators_typed_name() {
    let (yog, lernie_data) = (tempdir().unwrap(), tempdir().unwrap());

    // A lawful name raises the wall: seed first (§8.1 order), then `lernie new`
    // at the typed leaf — the pinned template already grants the worker role's
    // whole tool pool, so nothing runs after (§8.1, bl-7fc8).
    let (name, argv) = new_workspace(&yog, &lernie_data, "  ops  ").unwrap();
    assert_eq!(name, "ops", "surrounding whitespace forgiven, nothing else");
    assert_eq!(argv[0], ["prime"], "seed first (§8.1)");
    assert_eq!(argv[1][0], "new");
    assert!(
        argv[1][1].ends_with("workspaces/ops"),
        "`lernie new` targets the operator's own name under the names root",
    );
    assert_eq!(argv.len(), 2, "nothing runs after `new`");

    // Refused shapes never reach the planner (§3.1: no suffixing, no
    // prompt-loop — the operator retypes).
    for typed in ["", "Ops!", "two words", "-ops", &"a".repeat(33)] {
        assert!(
            new_workspace(&yog, &lernie_data, typed).is_err(),
            "{typed:?} is not a lawful sphere label"
        );
    }
    assert_eq!(
        new_workspace(&yog, &lernie_data, "unknown").unwrap_err(),
        NameError::Reserved,
        "bl's unstamped-claim fallback would false-join every claim (§3.1)",
    );

    // Collision: an existing leaf owns its name (the dir's existence *is* the
    // registration, §3.1), so retyping it is refused outright.
    std::fs::create_dir_all(workspace_path(yog.path(), "ops")).unwrap();
    assert_eq!(
        new_workspace(&yog, &lernie_data, "ops").unwrap_err(),
        NameError::Taken("ops".to_owned()),
    );
    // …and so is a name equal to a leaf under **lernie's** roots, which are its
    // territory, not yog's, but occupy the same namespace all the same.
    std::fs::create_dir_all(lernie_data.path().join("workspaces/acme")).unwrap();
    assert_eq!(
        new_workspace(&yog, &lernie_data, "acme").unwrap_err(),
        NameError::Taken("acme".to_owned()),
    );
    // A different name is still free.
    assert!(new_workspace(&yog, &lernie_data, "dev").is_ok());
}
