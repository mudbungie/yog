//! The cohort join, over a trail built the way the start flow writes one.

use std::path::{Path, PathBuf};

use balls::delivery_path::attempt_path;
use balls::layout::Xdg;

use super::{members, worktrees};
use crate::opslog::{OpEntry, Origin};

const PROJECT: &str = "/dev/proj";
const WS: &str = "/w/workspaces/cobalt-gecko";

fn layout() -> Xdg {
    Xdg::with(Path::new("/home/u"), None, Some("/home/u/.local/state"))
}

/// balls' own placement of one handle's worktree — what a fire binds.
fn candidate(handle: &str) -> String {
    attempt_path(&layout(), PROJECT, handle)
        .to_string_lossy()
        .into_owned()
}

/// One fire row exactly as `start::execute_prompt` logs it: the logical
/// `litany` argv0, `--name`, the typed `--cwd` binding, then ws and goal.
fn fire(conversation: &str, binding: &str) -> OpEntry {
    OpEntry {
        ts: "TS".to_owned(),
        argv: [
            "litany",
            "prompt",
            "--name",
            conversation,
            "--cwd",
            binding,
            WS,
            "Ball bl-1f2a: do it",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect(),
        cwd: WS.to_owned(),
        exit: crate::opslog::DETACHED_EXIT,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::Balls,
    }
}

#[test]
fn every_fire_bound_to_a_candidate_is_a_member_and_nothing_else_is() {
    let mut trail = vec![
        fire("amber-1", &candidate("at-0badcafe")),
        fire("basil-2", &candidate("at-12345678")),
    ];
    // A fire bound to the ordinary `work/<id>` claim is not a candidate.
    trail.push(fire(
        "cedar-3",
        "/w/balls/plugins/bl-delivery/dev/proj/bl-1f2a",
    ));
    // A fire that bound nothing at all (the bare rung) names no directory.
    let mut bare = fire("dill-4", "x");
    bare.argv.retain(|w| w != "--cwd" && w != "x");
    trail.push(bare);
    // A candidate of ANOTHER project reproduces no path under this one.
    trail.push(fire(
        "elder-5",
        "/home/u/.local/state/balls/attempts/dev/other/at-99999999",
    ));
    // A row that is not a fire at all.
    let mut claim = fire("fig-6", &candidate("at-abcdef01"));
    claim.argv = ["bl", "claim", "bl-1f2a", "--as", "cobalt-gecko"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    trail.push(claim);
    // A row too short to carry a verb.
    let mut stub = fire("gorse-7", &candidate("at-aaaaaaaa"));
    stub.argv.truncate(1);
    trail.push(stub);
    // A fire in ANOTHER workspace, which is another seat's cohort.
    let mut elsewhere = fire("hazel-8", &candidate("at-bbbbbbbb"));
    elsewhere.cwd = "/w/workspaces/other".to_owned();
    trail.push(elsewhere);

    let found = members(&trail, &layout(), Path::new(PROJECT), Path::new(WS));
    let names: Vec<&str> = found.iter().map(|m| m.conversation.as_str()).collect();
    assert_eq!(names, ["amber-1", "basil-2"]);
    assert_eq!(found[0].handle, "at-0badcafe");
    assert_eq!(found[1].worktree, PathBuf::from(candidate("at-12345678")));
}

#[test]
fn a_re_fire_onto_one_candidate_is_one_member_and_the_last_row_wins() {
    let trail = vec![
        fire("amber-1", &candidate("at-0badcafe")),
        fire("basil-2", &candidate("at-12345678")),
        fire("cedar-3", &candidate("at-0badcafe")),
    ];
    let found = members(&trail, &layout(), Path::new(PROJECT), Path::new(WS));
    let names: Vec<&str> = found.iter().map(|m| m.conversation.as_str()).collect();
    assert_eq!(names, ["basil-2", "cedar-3"]);
}

#[test]
fn a_workspace_that_never_fanned_has_no_cohort_and_no_worktrees() {
    let trail = vec![fire(
        "amber-1",
        "/w/balls/plugins/bl-delivery/dev/proj/bl-1f2a",
    )];
    assert!(members(&trail, &layout(), Path::new(PROJECT), Path::new(WS)).is_empty());
    assert!(worktrees(&trail, &layout(), Path::new(PROJECT), Path::new(WS)).is_empty());
}

#[test]
fn the_worktree_view_is_the_members_bindings_and_only_those() {
    let trail = vec![
        fire("amber-1", &candidate("at-0badcafe")),
        fire("basil-2", &candidate("at-12345678")),
    ];
    assert_eq!(
        worktrees(&trail, &layout(), Path::new(PROJECT), Path::new(WS)),
        vec![
            PathBuf::from(candidate("at-0badcafe")),
            PathBuf::from(candidate("at-12345678")),
        ],
    );
}
