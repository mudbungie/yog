//! The flag row's two directions and the fold that turns it into a §6 fact
//! (bl-6f2f).

use super::{Flag, YOG_FLAG, fold, latest, raised};
use crate::opslog::{OpEntry, OpRow, Origin};
use std::path::{Path, PathBuf};

/// The workspace every fixture here uses, as a path and as the §4.1 key a row's
/// `cwd` carries — one spelling, so a test cannot pass by writing the other.
const WS: &str = "/names/alba";

fn row(ts: &str, cwd: &str, argv: &str, stdout: &str) -> OpRow {
    OpRow {
        ts: ts.to_owned(),
        argv: argv.to_owned(),
        cwd: cwd.to_owned(),
        exit: 0,
        stdout: stdout.to_owned(),
        stderr: String::new(),
        origin: Origin::Conversation,
    }
}

#[test]
fn the_row_names_the_conversation_and_never_banners() {
    let entry: OpEntry = raised("7".to_owned(), Path::new(WS), "c-1", "look at this");
    assert_eq!(entry.argv, vec![YOG_FLAG.to_owned(), "c-1".to_owned()]);
    assert_eq!(entry.cwd, WS);
    assert_eq!(entry.exit, 0, "raising attention is not a failure");
    assert_eq!(entry.origin, Origin::Conversation);
    assert_eq!(entry.stdout, "look at this");
}

/// File order decides, so a second flag supersedes the first — which is what
/// makes an acknowledged flag stay acknowledged and a new one fire again.
#[test]
fn the_newest_flag_on_the_conversation_wins() {
    let rows = [
        row("1", WS, "yog-flag c-1", "first"),
        row("2", WS, "yog-flag c-1", "second"),
    ];
    assert_eq!(
        latest(&rows, WS, "c-1"),
        Some(Flag {
            at: "2".to_owned(),
            reason: "second".to_owned()
        })
    );
}

/// Everything that is not this conversation's flag: another agent's, another
/// workspace's, another pseudo-binary, and a row whose argv stops short.
#[test]
fn nothing_else_reads_as_a_flag_on_this_conversation() {
    let rows = [
        row("1", WS, "yog-flag c-2", "another agent"),
        row("2", "/names/other", "yog-flag c-1", "another workspace"),
        row("3", WS, "yog-monitor drifting c-1 sha m 1 2", "a verdict"),
        row("4", WS, "yog-flag", "no agent at all"),
    ];
    assert_eq!(latest(&rows, WS, "c-1"), None);
}

fn tree(agent: &str) -> crate::git_tree::GitTree {
    crate::git_tree::GitTree {
        agents: vec![crate::boundary::tests::agent(
            agent,
            crate::git_tree::AgentState::Quiescent,
            0,
        )],
        ..Default::default()
    }
}

/// The fold is what joins the two halves the defect left apart: the row is
/// written into yog's own trail, and the agent §6 reads is stamped from it.
#[test]
fn the_fold_stamps_the_agent_the_row_names() {
    let mut trees = std::collections::HashMap::new();
    trees.insert(PathBuf::from(WS), tree("c-1"));
    let rows = [row("9", WS, "yog-flag c-1", "please look")];
    let folded = fold(trees, &rows);
    let stamped = folded[Path::new(WS)].agents[0].flagged.clone();
    assert_eq!(
        stamped,
        Some(Flag {
            at: "9".to_owned(),
            reason: "please look".to_owned()
        })
    );
}

/// A world nobody has flagged walks the tail once and touches nothing — the
/// ordinary case, and the one every publish pays for.
#[test]
fn a_trail_with_no_flag_leaves_every_agent_alone() {
    let mut trees = std::collections::HashMap::new();
    trees.insert(PathBuf::from(WS), tree("c-1"));
    let rows = [row("1", WS, "yog-step derive", "")];
    let folded = fold(trees, &rows);
    assert_eq!(folded[Path::new(WS)].agents[0].flagged, None);
}

/// …and a flag that was later answered elsewhere clears by absence, because
/// the fold assigns rather than accumulates: the trail is the home, and a
/// stamped agent is a query over it, never a second copy that can go stale.
#[test]
fn the_fold_assigns_rather_than_accumulates() {
    let mut trees = std::collections::HashMap::new();
    let mut only = tree("c-1");
    only.agents[0].flagged = Some(Flag {
        at: "stale".to_owned(),
        reason: "from a previous pass".to_owned(),
    });
    trees.insert(PathBuf::from(WS), only);
    let rows = [row("1", WS, "yog-flag c-2", "somebody else")];
    let folded = fold(trees, &rows);
    assert_eq!(folded[Path::new(WS)].agents[0].flagged, None);
}
