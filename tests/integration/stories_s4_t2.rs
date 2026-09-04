//! STORIES **S4-T2** assign-release: the §8.2 late-binding verbs spawn the exact
//! argv (assign = `bl claim --as <to>`, release = `bl unclaim --as <name>`) and
//! every outcome lands in `ops.jsonl` (§8.2, §3.2, §3.5, §15 M6 Z4).
//!
//! **Whether the verb is offered is no longer asked here** (bl-33e9): the gate
//! is a fold over the §3.5 `JoinState` that `Query::WorkspaceBalls` already
//! answers on every row, so REMOTE §9.4 leaves it to the seat and `bl` itself
//! refuses at the fire. What this beat pins is the argv and the trail.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::actions::verbs;
use yog::cli_outbound::Cli;
use yog::opslog;

/// STORIES **S4-T2** assign-release.
#[test]
fn s4_t2_assign_release_argv_and_trail() {
    let (bin, state, project) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let bl = Recorder::new(bin.path(), "bl");
    let cli = Cli::new(bl.path());
    let (sr, p) = (state.path(), project.path());

    // Assign a ready ball to a workspace (§3.2): `bl claim <id> --as <name>`.
    verbs::assign(&cli, sr, "T1", p, "bl-1", "cobalt-gecko").unwrap();
    // Release: `bl unclaim <id> --as <name>` (the bound workspace name).
    verbs::unclaim(&cli, sr, "T2", p, "bl-1", "cobalt-gecko").unwrap();

    let argv: Vec<Vec<String>> = bl.invocations().into_iter().map(|i| i.argv).collect();
    assert_eq!(argv[0], ["claim", "bl-1", "--as", "cobalt-gecko"], "assign");
    assert_eq!(
        argv[1],
        ["unclaim", "bl-1", "--as", "cobalt-gecko"],
        "release"
    );

    // Every attempt is a durable ops line (§4.2): one per spawn.
    let ops = opslog::tail(sr, 16);
    assert_eq!(ops.len(), 2, "assign + release, both logged");
    assert!(ops.iter().all(|e| e.argv[0].ends_with("bl") && e.exit == 0));
}
