//! STORIES **S4-T2** assign-move-release: the §8.2 late-binding verbs spawn the
//! exact argv (assign = `bl claim --as <to>`, move = `bl unclaim --as <from>`
//! then `bl claim --as <to>`, release = `bl unclaim --as <name>`), every outcome
//! lands in `ops.jsonl`, and the enablement predicates refuse exactly what the
//! underlying `bl` verb would (§8.2, §3.2, §3.5, §15 M6 Z4).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::actions::{assign_enabled, move_enabled, unclaim_enabled, verbs};
use yog::cli_outbound::Cli;
use yog::opslog;
use yog::projects::join::JoinState;

/// STORIES **S4-T2** assign-move-release.
#[test]
fn s4_t2_assign_move_release_argv_and_enablement() {
    let (bin, state, project) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let bl = Recorder::new(bin.path(), "bl");
    let cli = Cli::new(bl.path());
    let (sr, p) = (state.path(), project.path());

    // Assign a ready ball to a workspace (§3.2): `bl claim <id> --as <name>`.
    verbs::assign(&cli, sr, "T1", p, "bl-1", "cobalt-gecko").unwrap();
    // Move it: the source releases (`--as from`), the target claims (`--as to`).
    verbs::reassign(&cli, sr, "T2", p, "bl-1", "cobalt-gecko", "amber-toad").unwrap();
    // Release: `bl unclaim <id> --as <name>` (the bound workspace name).
    verbs::unclaim(&cli, sr, "T3", p, "bl-1", "amber-toad").unwrap();

    let argv: Vec<Vec<String>> = bl.invocations().into_iter().map(|i| i.argv).collect();
    assert_eq!(argv[0], ["claim", "bl-1", "--as", "cobalt-gecko"], "assign");
    assert_eq!(
        argv[1],
        ["unclaim", "bl-1", "--as", "cobalt-gecko"],
        "move: from"
    );
    assert_eq!(argv[2], ["claim", "bl-1", "--as", "amber-toad"], "move: to");
    assert_eq!(
        argv[3],
        ["unclaim", "bl-1", "--as", "amber-toad"],
        "release"
    );

    // Every attempt is a durable ops line (§4.2): one per spawn (move = two).
    let ops = opslog::tail(sr, 16);
    assert_eq!(ops.len(), 4, "assign + move ×2 + release, all logged");
    assert!(ops.iter().all(|e| e.argv[0].ends_with("bl") && e.exit == 0));

    // Enablement refuses what `bl` would (§3.5): assign only a ready ball;
    // release/move only a ball this yog owns (Bound).
    assert!(assign_enabled(JoinState::ReadyStartable));
    assert!(!assign_enabled(JoinState::Bound));
    assert!(!assign_enabled(JoinState::ClaimedElsewhere));
    assert!(move_enabled(JoinState::Bound));
    assert!(!move_enabled(JoinState::ReadyStartable));
    assert!(!move_enabled(JoinState::ClaimedElsewhere));
    assert!(unclaim_enabled(JoinState::Bound));
    assert!(!unclaim_enabled(JoinState::ReadyStartable));
}
