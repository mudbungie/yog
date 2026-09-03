//! The §11 inspector family's derivations, driven against a tempdir workspace
//! and a hand-built snapshot (bl-6233). Each read below is one call into a
//! module with its own tables; what is pinned here is the part that had no home
//! before — which tail is folded, whose liveness the steps read, what a named
//! path is allowed to open, and where the spine's children come from.

use std::path::Path;

/// Config-frozen-at's own drive (bl-13f9) — its own file, because it is the
/// one read here that needs a real config lineage on disk rather than a
/// tempdir and a hand-built snapshot.
mod governing;

use super::*;
use crate::boundary::tests::{agent, snapshot};
use crate::git_tree::{AgentState, Stream};
use tempfile::TempDir;

/// A caller's clock long past any step these tests write — the §7.2 in-flight
/// window (bl-776a) has elapsed, so a wound the world holds is stated.
const AFTER_THE_WINDOW: i64 = 4_000_000_000;

const AGENT: &str = "c-1";
const CHILD: &str = "c-1-w-1";

/// A workspace on disk with one committed message for `AGENT`.
fn world() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let messages = dir.path().join("agents").join(AGENT).join("messages");
    std::fs::create_dir_all(&messages).unwrap();
    std::fs::write(messages.join("001-user.md"), b"is this thing on?\n").unwrap();
    dir
}

/// A snapshot whose one agent wears `state` and is mid-sentence.
fn snap_at(ws: &Path, state: AgentState) -> Snapshot {
    let mut row = agent(AGENT, state, 100);
    row.stream = Stream {
        text: Some("half a thought".to_owned()),
        ..Stream::default()
    };
    snapshot(ws, "alba", vec![row], vec![])
}

/// The ruling this ball had to make: a headless answer folds the in-flight tail
/// exactly as the window does, so the two seats never describe one moment
/// differently — and folds nothing once the step settles, because a settled
/// step's trailing text is already committed and would paint twice.
#[test]
fn the_live_tail_is_folded_only_while_a_call_is_in_flight() {
    let dir = world();
    let (ws, other) = (dir.path(), Path::new("/nowhere"));
    let flying = snap_at(ws, AgentState::InFlight);
    let tail = transcript(&flying, ws, AGENT, AFTER_THE_WINDOW);
    assert_eq!(tail.entries.len(), 2, "the committed half plus the tail");
    assert_eq!(
        tail.entries[1].kind,
        crate::transcript::EntryKind::Streaming {
            thinking: String::new(),
            text: "half a thought".to_owned(),
        }
    );
    // Settled, unknown to the snapshot, and in an underived workspace: three
    // ways to have no tail, and all three are the committed half alone.
    for (snap, at, agent_id) in [
        (snap_at(ws, AgentState::Quiescent), ws, AGENT),
        (snap_at(ws, AgentState::InFlight), ws, "who"),
        (snap_at(other, AgentState::InFlight), ws, AGENT),
    ] {
        assert!(live_tail(&snap, at, agent_id).is_none());
        assert_eq!(
            transcript(&snap, at, agent_id, AFTER_THE_WINDOW),
            crate::transcript::build(at, agent_id),
            "nothing beyond the committed half"
        );
    }
}

/// The steps view reads the agent's liveness off the world rather than taking
/// it as a parameter: a driver at work is still filling its newest step, so
/// that step's silence is a call in flight and not a §7.3 wound.
#[test]
fn the_steps_read_their_liveness_off_the_snapshot() {
    let dir = world();
    let ws = dir.path();
    let step = ws.join("steps").join(AGENT).join("001");
    std::fs::create_dir_all(&step).unwrap();
    std::fs::write(step.join("response.json"), b"").unwrap();
    let driven = steps(&snap_at(ws, AgentState::Live), ws, AGENT, AFTER_THE_WINDOW);
    assert_eq!(driven.steps.len(), 1);
    assert!(!driven.steps[0].wound.wounded(), "a driver is filling it");
    let abandoned = steps(
        &snap_at(ws, AgentState::Stopped),
        ws,
        AGENT,
        AFTER_THE_WINDOW,
    );
    assert!(abandoned.steps[0].wound.wounded(), "nobody is driving it");
    // An agent the snapshot never carried reads as stopped, which is what an
    // untracked tree's newest step honestly is.
    assert!(
        steps(
            &snap_at(Path::new("/nowhere"), AgentState::Live),
            ws,
            AGENT,
            AFTER_THE_WINDOW,
        )
        .steps[0]
            .wound
            .wounded()
    );
}

/// The listing, and the containment rule beside it: a named path is resolved
/// against the listing this same answer built, so the read can open nothing it
/// did not enumerate.
#[test]
fn files_opens_only_what_its_own_listing_named() {
    let dir = world();
    let ws = dir.path();
    let work = ws.join("agents").join(AGENT);
    std::fs::write(work.join("goal.md"), b"ship it").unwrap();
    let (view, none) = files(ws, AGENT, None, None);
    let crate::files_view::FilesView::Present { entries, .. } = &view else {
        unreachable!("the worktree is on disk")
    };
    assert!(entries.iter().any(|e| e.rel_path == "goal.md"));
    assert!(none.is_none(), "no path named, no bytes read");
    assert_eq!(
        files(ws, AGENT, Some("goal.md"), None).1,
        Some(crate::files_view::Preview::Text("ship it".to_owned()))
    );
    // A path outside the listing, a directory, and a torn-down worktree: each
    // answers no preview rather than opening something the listing never named.
    assert!(files(ws, AGENT, Some("../../etc/passwd"), None).1.is_none());
    assert!(files(ws, AGENT, Some("messages"), None).1.is_none());
    let (absent, preview) = files(ws, "gone", Some("goal.md"), None);
    assert_eq!(absent, crate::files_view::FilesView::AbsentWorktree);
    assert!(preview.is_none());
}

/// **`at` names the tree, and the same read answers either one** (REMOTE §9.7,
/// bl-44e9): with it, the listing is that commit's blobs and a named path's
/// bytes come out of `git show`; without it, the live worktree. One derivation
/// with a parameter rather than two reads — which is what makes the window's
/// pinned Files tab and a headless `/files --at <commit>` one implementation.
#[test]
fn files_reads_the_commits_tree_when_one_is_named() {
    let fx = crate::git_tree::tests::fixture::Fixture::new();
    let conv = "20260427T120000Z-aaaa";
    fx.build_agent(conv, "walk the rail");
    let at = format!("agents/{conv}");

    let (pinned, bytes) = files(&fx.path, conv, Some("goal.md"), Some(&at));
    let crate::files_view::FilesView::Present { entries, .. } = &pinned else {
        unreachable!("a real commit lists")
    };
    assert!(entries.iter().any(|e| e.rel_path == "goal.md"));
    assert_eq!(
        bytes,
        Some(crate::files_view::Preview::Text("walk the rail".to_owned())),
        "the bytes come out of that commit's tree"
    );
    // The containment rule is the same rule over either tree: a path the pinned
    // listing does not name opens nothing.
    assert!(
        files(&fx.path, conv, Some("../etc/passwd"), Some(&at))
            .1
            .is_none()
    );
    // A commit git does not have lists nothing, exactly as a torn-down worktree
    // does — the general path with the tree absent.
    assert_eq!(
        files(&fx.path, conv, None, Some("deadbeef")).0,
        crate::files_view::FilesView::AbsentWorktree
    );
}

/// The spine's inputs come off the snapshot — the parent's own commits and its
/// descent-id children — which is the gather the frame used to do for itself.
#[test]
fn the_rail_gathers_its_children_off_the_snapshot() {
    let dir = world();
    let ws = dir.path();
    let snap = snapshot(
        ws,
        "alba",
        vec![
            agent(AGENT, AgentState::Live, 100),
            agent(CHILD, AgentState::Quiescent, 90),
        ],
        vec![],
    );
    let (view, tx) = (
        steps(&snap, ws, AGENT, AFTER_THE_WINDOW),
        transcript(&snap, ws, AGENT, AFTER_THE_WINDOW),
    );
    let spine = rail(&snap, ws, AGENT, &view, &tx);
    assert!(spine.notches.is_empty(), "no steps on disk, no notches");
    // A workspace the snapshot does not carry gathers nothing at all — the
    // general path with no inputs, never a branch of its own.
    assert_eq!(
        rail(&snap, Path::new("/nowhere"), AGENT, &view, &tx),
        crate::rail::Rail::default()
    );
}

/// The §3.3 ladder over the conversation *root*, and its last rung: a selection
/// the snapshot does not carry is its own root and wears its own id.
#[test]
fn the_speaker_is_the_roots_display_name_or_the_id_itself() {
    let mut root = agent(AGENT, AgentState::Live, 100);
    root.name = Some("koi".to_owned());
    let agents = vec![root, agent(CHILD, AgentState::Quiescent, 90)];
    assert_eq!(
        speaker(&agents, CHILD),
        "koi",
        "the child speaks as its root"
    );
    assert_eq!(speaker(&agents, "unheard-of"), "unheard-of");
}
