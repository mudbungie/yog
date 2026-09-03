//! The **free-function derivations** the query chokepoint is built out of
//! (§8.5) — the descent forest a seat folds, a conversation's bound ball, a
//! workspace's attention/running rollup, and the mint's occupied name set.
//! Split from [`super`] at §12's budget on the seam that file already had:
//! these are claims about one derivation over a hand-built snapshot, and the
//! beats left there are claims about `answer` **dispatching** to them.

use super::{ui, ws};
use crate::boundary::answer::*;
use crate::boundary::tests::{agent, bound_row, snapshot};
use crate::git_tree::AgentState;
use crate::projects::join::JoinState;
use std::path::{Path, PathBuf};

/// REMOTE §9.7's altitude ruling (bl-44e9): the answer is the whole descent
/// forest with its per-row rollups, and the all-collapsed list every seat used
/// to be handed is the **root subset** a seat selects out of it.
#[test]
fn conversations_answer_the_whole_forest_and_the_seat_folds_it() {
    let snap = snapshot(
        &ws(),
        "alba",
        vec![
            agent("c-1", AgentState::Live, 100),
            agent("c-1-w-1", AgentState::Quiescent, 90),
            agent("c-2", AgentState::Stopped, 50),
        ],
        vec![],
    );
    let rows = conversations(&snap, &ui(), &ws(), 200);
    assert_eq!(
        rows.iter().map(|r| r.root_id.as_str()).collect::<Vec<_>>(),
        ["c-1", "c-1-w-1", "c-2"],
        "every member of the forest, in paint order"
    );
    assert_eq!(
        rows[0].members, 2,
        "the rollup is the subtree's, not the fold's"
    );
    // No expanded set crosses, so the answer carries none: a seat holding no
    // fold selects the roots, which is the list this query used to answer.
    let collapsed = crate::nav::convs::visible(&rows, &std::collections::HashSet::new());
    assert_eq!(
        collapsed
            .iter()
            .map(|r| r.root_id.as_str())
            .collect::<Vec<_>>(),
        ["c-1", "c-2"]
    );
    assert!(conversations(&snap, &ui(), Path::new("/other"), 200).is_empty());
}

#[test]
fn conv_ball_reads_the_join_or_renders_the_stray_id() {
    let project = PathBuf::from("/proj");
    let snap = snapshot(
        &ws(),
        "alba",
        vec![],
        vec![bound_row(&project, "bl-1", &ws(), "alba")],
    );
    let hit = conv_ball(&snap, "bl-1");
    assert_eq!(hit.state, Some(JoinState::Bound));
    assert_eq!(hit.title.as_deref(), Some("title of bl-1"));
    let miss = conv_ball(&snap, "bl-9");
    assert_eq!(miss.id, "bl-9");
    assert_eq!(miss.state, None);
}

#[test]
fn workspace_stats_roll_up_attention_and_running() {
    let mut waiting = agent("c-2", AgentState::Quiescent, 10);
    waiting.notify_oid = Some("n".repeat(40));
    let snap = snapshot(
        &ws(),
        "alba",
        vec![agent("c-1", AgentState::InFlight, 100), waiting],
        vec![],
    );
    let (attention, agents, running) = workspace_stats(&snap, &ui(), &ws());
    assert_eq!(agents, 2);
    assert!(running, "an InFlight member runs");
    assert_eq!(attention, 1, "the notify mark begs attention");
    assert_eq!(
        workspace_stats(&snap, &ui(), Path::new("/other")),
        (0, 0, false),
        "an underived workspace contributes zeros"
    );
}
#[test]
fn names_in_reads_the_name_fact_children_included() {
    // The mint's occupied set (§3.3, bl-08f2): each agent's name_fact — the
    // litany-stored blob, with the legacy goal stamp while pre-0.0.4 roots
    // live. A named descent child occupies too: litany refuses a taken name at
    // fire, so the mint must see everything litany would.
    let mut named = agent("c-1", AgentState::Live, 1);
    named.name = Some("pale-otter".into());
    let mut legacy = agent("c-2", AgentState::Live, 2);
    legacy.goal_name = Some("brave-fox".into());
    let mut child = agent("c-1-x1", AgentState::Live, 3);
    child.name = Some("quiet-heron".into());
    let snap = snapshot(
        &ws(),
        "alba",
        vec![named, legacy, child, agent("c-3", AgentState::Live, 4)],
        vec![],
    );
    assert_eq!(
        names_in(&snap, &ws()),
        ["pale-otter", "brave-fox", "quiet-heron"]
    );
    assert!(names_in(&snap, Path::new("/other")).is_empty());
}

/// **A bound conversation's deliverable is not in the listing, and the answer
/// says where it is** (bl-1015). A path or ball rung seeds litany's cwd mark
/// at creation (DESIGN §3.3, *"the one channel"*), so every tool step of every
/// turn runs at the target while `/files` walks the agent worktree — which
/// held goal and soul and no work product, with nothing anywhere saying the
/// work had gone elsewhere.
///
/// Both "here" cases collapse to `None`, which is why the reply carries a path
/// exactly when there is somewhere else to name: no mark at all (a bare start,
/// whose tools run in the listed worktree) and a mark that names that very
/// worktree (an agent that `cd`ed home).
#[test]
fn a_bound_conversation_names_the_directory_its_work_lands_in() {
    let dir = tempfile::tempdir().expect("tmp");
    let ws = dir.path().join("ws");
    let repo = ws.join("repo.git");
    std::fs::create_dir_all(&repo).expect("repo");
    assert_eq!(inspector::working_dir(&ws, "amber-1"), None);
    let git = |args: &[&str]| {
        crate::git_env::output(crate::git_env::git().arg("--git-dir").arg(&repo).args(args))
            .expect("git")
    };
    assert!(git(&["init", "--bare", "-b", "main"]).status.success());
    let mark = |at: &Path| {
        let blob = dir.path().join("cwd");
        std::fs::write(&blob, format!("{}\n", at.display())).expect("write");
        let out = git(&["hash-object", "-w", "--", &blob.display().to_string()]);
        let oid = String::from_utf8(out.stdout)
            .expect("oid")
            .trim()
            .to_owned();
        assert!(
            git(&["update-ref", "refs/litany/cwd/amber-1", &oid])
                .status
                .success()
        );
    };

    let bound = PathBuf::from("/home/u/proj");
    mark(&bound);
    assert_eq!(inspector::working_dir(&ws, "amber-1"), Some(bound));

    let home = crate::files_view::agent_worktree(&ws, "amber-1");
    mark(&home);
    assert_eq!(
        inspector::working_dir(&ws, "amber-1"),
        None,
        "a mark naming the listed worktree is the case where the listing is the answer"
    );
}

/// **The settled-failure notice, at the chokepoint** (bl-015b). A conversation
/// refused at its first model call answered one committed entry and nothing
/// else; the answerer now folds the §7.3 wound on as a virtual trailing entry,
/// so the pane the operator reads names the provider row to sign in to.
#[test]
fn a_stopped_conversation_answers_its_wound_as_a_trailing_entry() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    let id = "20260101T000000Z-c1";
    let messages = ws.join("agents").join(id).join("messages");
    std::fs::create_dir_all(&messages).unwrap();
    std::fs::write(messages.join("001-user.md"), b"go").unwrap();
    // The live shape of a refusal: brazen speaks it in band on stdout, so the
    // step settles Failed with an auth-class error and no `meta.json`.
    let step = ws.join("steps").join(id).join("001");
    std::fs::create_dir_all(&step).unwrap();
    std::fs::write(
        step.join("response.json"),
        b"{\"type\":\"error\",\"message\":\"no credential for this provider\"}\n{\"type\":\"end\"}\n",
    )
    .unwrap();

    let snap = snapshot(ws, "alba", vec![agent(id, AgentState::Stopped, 10)], vec![]);
    let answered = inspector::transcript(&snap, ws, id);
    assert_eq!(answered.entries.len(), 2, "the message AND the notice");
    let said = String::from_utf8(answered.entries[1].raw.clone()).unwrap();
    assert!(
        said.contains("credentials"),
        "it says what happened: {said}"
    );
}

/// The two folds are one question asked at two moments, and the live one wins:
/// a wound is claimed only of a step nobody is driving, so an in-flight call
/// answers its tail and never a notice.
#[test]
fn an_in_flight_conversation_answers_its_tail_and_no_notice() {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path();
    let id = "20260101T000000Z-c1";
    let messages = ws.join("agents").join(id).join("messages");
    std::fs::create_dir_all(&messages).unwrap();
    std::fs::write(messages.join("001-user.md"), b"go").unwrap();
    let mut live = agent(id, AgentState::InFlight, 10);
    live.stream = crate::git_tree::Stream {
        text: Some("half".into()),
        thinking: None,
        last_delta: Some(crate::git_tree::Delta::Text),
    };
    let snap = snapshot(ws, "alba", vec![live], vec![]);
    let answered = inspector::transcript(&snap, ws, id);
    assert_eq!(answered.entries.len(), 2);
    assert!(matches!(
        answered.entries[1].kind,
        crate::transcript::EntryKind::Streaming { .. }
    ));
}
