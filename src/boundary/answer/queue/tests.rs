//! The decision queue's tables (STORIES S14-T1…T5): the queue is the roster's own
//! subsequence and the strip's own count, a row says why it is asking, and an
//! acknowledgement writes exactly the watermarks the window writes.

use super::*;
use crate::binding::{Workspace, WorkspaceKind};
use crate::boundary::tests::{agent, snapshot};
use crate::git_tree::GitTree;
use crate::ui_state::SeenKind;
use tempfile::TempDir;

fn ws_a() -> PathBuf {
    PathBuf::from("/names/alba")
}

fn ws_b() -> PathBuf {
    PathBuf::from("/names/koi")
}

/// A writable `ui.json` — the queue's answer is a function of the watermarks,
/// so the acknowledgement tables need a document that actually records.
fn writable() -> (TempDir, UiState) {
    let dir = tempfile::tempdir().unwrap();
    let ui = UiState::open(dir.path().join("ui.json"));
    (dir, ui)
}

/// One agent asking for attention four different ways.
fn shouting(id: &str, ts: i64) -> Agent {
    let mut a = agent(id, AgentState::Stopped, ts);
    a.notify_oid = Some("n".repeat(40));
    a.budget_oid = Some("b".repeat(40));
    a.conflicted_oid = Some("c".repeat(40));
    a.preview = Some("which branch do you want?\nand a second line".to_owned());
    a
}

/// One undelivered deposit — the §6 rule-5 fixture.
fn mail() -> crate::inboxview::InboxEntry {
    crate::inboxview::InboxEntry {
        name: "user-001.md".to_owned(),
        raw: b"hi".to_vec(),
        deposit: crate::inboxview::parse_deposit(b"hi"),
    }
}

/// Two workspaces, so the roster's across-workspace order is exercised.
fn world(a: Vec<Agent>, b: Vec<Agent>) -> Snapshot {
    let mut snap = snapshot(&ws_a(), "alba", a, vec![]);
    snap.workspaces.push(Workspace {
        path: ws_b(),
        kind: WorkspaceKind::Named {
            name: "koi".to_owned(),
        },
    });
    snap.trees.insert(
        ws_b(),
        GitTree {
            commits: vec![],
            agents: b,
        },
    );
    snap
}

/// The queue *is* the roster filtered — never a second walk with its own order —
/// and its length is the §6 strip's own total, so the number the window paints
/// and the rows a headless seat is handed can never disagree.
#[test]
fn the_queue_is_the_rosters_attention_bearing_subsequence_and_the_strips_count() {
    let snap = world(
        vec![
            agent("c-1", AgentState::Live, 100),
            shouting("c-2", 90),
            agent("c-3", AgentState::Quiescent, 80),
        ],
        vec![shouting("k-1", 70)],
    );
    let (_dir, ui) = writable();
    let rows = queue(&snap, &ui, 200);
    let order: Vec<&str> = rows.iter().map(|r| r.agent.as_str()).collect();
    // `/names/alba` sorts before `/names/koi`; within a workspace the §6 rank
    // puts attention first.
    assert_eq!(order, ["c-2", "c-3", "k-1"]);
    assert_eq!(rows[0].workspace, ws_a());
    assert_eq!(rows[2].workspace, ws_b());

    let seen = |k, w: &str, a: &str, o: &str| ui.is_seen(k, w, a, o);
    let keyed = [
        (ws_a().to_string_lossy().into_owned(), vec![]),
        (ws_b().to_string_lossy().into_owned(), vec![]),
    ];
    let wss: Vec<(&str, &[Agent])> = keyed
        .iter()
        .map(|(k, _): &(String, Vec<Agent>)| {
            (
                k.as_str(),
                snap.trees
                    .get(Path::new(k))
                    .map_or(&[][..], |t| t.agents.as_slice()),
            )
        })
        .collect();
    assert_eq!(
        rows.len(),
        attention::strip_total(&wss, &seen),
        "the queue's length is the strip's number"
    );
    let roster_order: Vec<String> = roster(&snap, &ui).into_iter().map(|k| k.agent_id).collect();
    assert!(
        order
            .iter()
            .all(|id| roster_order.contains(&(*id).to_owned())),
        "every queued row is a roster entry"
    );
}

/// A row says where it is, what it is called, why it is asking and what it last
/// said — enough to answer it without another read.
#[test]
fn a_row_carries_its_address_its_reason_and_what_it_last_said() {
    let mut waiting = shouting("c-2", 90);
    waiting.pending = vec![mail()];
    let snap = world(vec![waiting], vec![]);
    let (_dir, ui) = writable();
    let rows = queue(&snap, &ui, 200);
    let row = &rows[0];
    assert_eq!(row.workspace, ws_a());
    assert_eq!(row.agent, "c-2");
    assert_eq!(row.display, "which branch do you want?");
    assert_eq!(row.state, AgentState::Stopped);
    assert!(!row.uncertain);
    assert_eq!(row.age_secs, 110);
    assert_eq!(row.pending, 1);
    assert_eq!(row.preview, "which branch do you want?");
    assert_eq!(
        row.signals,
        vec![
            AttentionKind::Notify,
            AttentionKind::Stopped,
            AttentionKind::Budget,
            AttentionKind::Conflicted,
            AttentionKind::Mail,
        ],
        "all five §6 signals, in badge order"
    );
}

/// The I0 convergence proof, from the headless side: `seen` writes exactly the
/// oids [`attention::evidence`] names — the same list the window's focus tick
/// writes — and the row leaves the queue.
#[test]
fn an_acknowledgement_writes_the_windows_own_watermarks_and_the_row_leaves() {
    let snap = world(vec![shouting("c-2", 90)], vec![]);
    let (_dir, mut ui) = writable();
    assert_eq!(queue(&snap, &ui, 200).len(), 1);

    mark_seen(&snap, &mut ui, &ws_a(), "c-2").unwrap();
    let key = ws_key(&ws_a());
    let evidence = attention::evidence(&shouting("c-2", 90));
    assert_eq!(
        evidence.len(),
        4,
        "mail carries no oid and cannot be quieted"
    );
    for (kind, oid) in &evidence {
        assert!(
            ui.is_seen(*kind, &key, "c-2", oid),
            "{kind:?} unacknowledged"
        );
    }
    assert!(!ui.is_seen(SeenKind::Notify, &key, "c-2", "z".repeat(40).as_str()));
    assert!(
        queue(&snap, &ui, 200).is_empty(),
        "answered, so it is off the queue"
    );
}

/// Rule 5 is not a watermark: undelivered mail self-clears when a driver reads
/// it, and no acknowledgement may pretend otherwise.
#[test]
fn an_acknowledgement_does_not_quiet_undelivered_mail() {
    let mut mailed = agent("c-2", AgentState::Quiescent, 90);
    mailed.pending = vec![mail()];
    let snap = world(vec![mailed], vec![]);
    let (_dir, mut ui) = writable();
    mark_seen(&snap, &mut ui, &ws_a(), "c-2").unwrap();
    let rows = queue(&snap, &ui, 200);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].signals, vec![AttentionKind::Mail]);
}

/// A gesture is an instruction: an acknowledgement aimed at a conversation yog
/// cannot see refuses by name rather than reporting a silent success.
#[test]
fn an_acknowledgement_aimed_at_nothing_refuses_by_name() {
    let snap = world(vec![agent("c-1", AgentState::Live, 10)], vec![]);
    let (_dir, mut ui) = writable();
    assert_eq!(
        mark_seen(&snap, &mut ui, &ws_a(), "ghost"),
        Err("no conversation \"ghost\" in /names/alba".to_owned())
    );
    assert_eq!(
        mark_seen(&snap, &mut ui, Path::new("/nowhere"), "c-1"),
        Err("no conversation \"c-1\" in /nowhere".to_owned())
    );
}

/// A workspace with no derived tree contributes nothing — the general path with
/// no inputs, not a special case.
#[test]
fn an_underived_workspace_is_simply_absent_from_the_roster() {
    let mut snap = world(vec![shouting("c-2", 90)], vec![]);
    snap.workspaces.push(Workspace {
        path: PathBuf::from("/names/unread"),
        kind: WorkspaceKind::Foreign,
    });
    let (_dir, ui) = writable();
    assert_eq!(roster(&snap, &ui).len(), 1);
    assert_eq!(queue(&snap, &ui, 200).len(), 1);
}
