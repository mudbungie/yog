//! The join itself (§3.9): every agent-side column against a real project repo,
//! a real fan, and a real agent worktree — and the honest absences a row wears
//! when nothing bound it.

use std::path::PathBuf;

use super::{AGENT, BALL, CONV, NAME, bill, claimed_project, layout, named_agent, snap, trail};
use crate::app::Snapshot;
use crate::opslog::OpEntry;
use crate::science::{Attempt, Outcome, project};
use crate::workdiff::{Change, tests::Project};

mod absences;

/// One model turn as litany commits it: canonical blocks wrapped in `content`.
const SAID: &str = r#"{"content":[{"type":"thinking","thinking":"hmm"},
{"type":"text","text":"done, tests green"}]}"#;
/// One delivered message, envelope and all.
const VERDICT: &str = "---\nfrom: judge-one\ndeposited_at: t\n---\ncandidate B reads cleaner\n";

/// A project repo, a balls layout and a workspace directory, all under one
/// throwaway root — every path this projection reads is inside it.
pub(super) struct Lab {
    _dir: tempfile::TempDir,
    pub(super) project: Project,
    xdg: balls::layout::Xdg,
    balls_root: PathBuf,
    pub(super) ws: PathBuf,
}

impl Lab {
    pub(super) fn new() -> Lab {
        Lab::over(claimed_project())
    }

    /// The same lab over a project founded some other way — the base column's
    /// own reading needs a source that shares no history with its target.
    pub(super) fn over(project: Project) -> Lab {
        let dir = tempfile::tempdir().unwrap();
        let (xdg, balls_root) = layout(dir.path());
        let ws = dir.path().join("workspaces").join(NAME);
        std::fs::create_dir_all(&ws).unwrap();
        Lab {
            _dir: dir,
            project,
            xdg,
            balls_root,
            ws,
        }
    }

    pub(super) fn project_at(&self, snap: &Snapshot, entries: &[OpEntry]) -> Vec<Attempt> {
        project(snap, &self.ws, entries, &self.xdg, &self.balls_root)
    }

    /// The claim attempt's own worktree path, by balls' formula.
    pub(super) fn claim(&self, claimant: Option<&str>) -> PathBuf {
        crate::binding::work_worktree_path(&self.balls_root, &self.project.path, BALL, claimant)
    }
}

/// The whole row for the ordinary claim attempt: it is bound like any other
/// (N = 1 is not a case), it carries its frozen inputs, its step-record figures
/// come off the snapshot's bills, and its diff column is the work-diff row.
#[test]
fn the_claim_attempt_projects_every_column() {
    let lab = Lab::new();
    let claim = lab.claim(None);
    let entries = trail(&lab.ws, &lab.project.path, &[(CONV, &claim)]);
    let snap = snap(
        &lab.ws,
        &lab.project.path,
        vec![named_agent()],
        vec![bill(AGENT, "001", 7, 11), bill(AGENT, "002", 3, 4)],
    );
    super::worktree(
        &lab.ws,
        AGENT,
        "ship bl-1",
        // The model turn FIRST and the delivered message last, so the terminal
        // read walks back past a non-model entry to reach the answer — the
        // ordinary shape of a conversation somebody has written into since.
        &[("claude-opus.json", SAID), ("judge-one.md", VERDICT)],
    );
    let rows = lab.project_at(&snap, &entries);
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = &rows[0];
    // The composed diff column — the same row `Query::WorkDiff` answers.
    assert_eq!(row.diff.ball_id, BALL);
    assert_eq!(row.diff.handle, None);
    assert_eq!(row.diff.range(), Some(format!("main..work/{BALL}")));
    // The third OID: the commit both ends departed from, which is the target's
    // own tip here — the claim branched off it and it has not moved since.
    let Change::Diff { target_oid, .. } = &row.diff.change else {
        panic!("both ends resolve: {row:?}");
    };
    assert_eq!(row.base.as_ref(), Some(target_oid));
    // The binding, resolved through the §3.3 name ladder to an agent id.
    assert_eq!(row.conversation.as_deref(), Some(AGENT));
    // Frozen inputs: the goal from the worktree, the pins from the fire's argv.
    assert_eq!(row.goal.as_deref(), Some("ship bl-1"));
    assert_eq!(
        row.pins,
        [
            "instructions/00-AGENTS.md=/p/AGENTS.md",
            "instructions/01-AGENTS.md=/p/src/AGENTS.md"
        ]
    );
    // Step records, off the one published walk — never a second pass.
    assert_eq!(row.usage.input_tokens, 10);
    assert_eq!(row.wall_secs, 15);
    assert_eq!(row.steps, 2);
    // What it said, and what was said to it.
    assert_eq!(row.response.as_deref(), Some("done, tests green"));
    assert_eq!(row.verdicts.len(), 1);
    assert_eq!(row.verdicts[0].sender, "judge-one");
    assert!(row.verdicts[0].body.contains("reads cleaner"), "{row:?}");
    // Undelivered and unopposed: the ordinary standing of work in progress.
    assert_eq!(row.outcome, Outcome::Pending);
    // The workspace is no git repo here, so the freeze reads as unreadable
    // rather than as some other commit.
    assert_eq!(row.governing, None);
    // An intact record: no compaction to speak of (bl-fde5).
    assert_eq!(row.compacted, 0);
}

/// A compacted conversation's row says so (bl-fde5): the counter proves how
/// many entries litany's compactor deleted, and the projection states that
/// bound rather than letting a short verdict list read as the conversation's
/// whole history.
#[test]
fn a_compacted_conversation_marks_its_projection() {
    let lab = Lab::new();
    let claim = lab.claim(None);
    let entries = trail(&lab.ws, &lab.project.path, &[(CONV, &claim)]);
    let snap = snap(&lab.ws, &lab.project.path, vec![named_agent()], vec![]);
    // The compactor's leavings: the surviving record starts at 004, so entries
    // 001–003 — any verdicts among them included — are proven gone.
    let dir = lab.ws.join("agents").join(AGENT).join("messages");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("004-claude-opus.json"), SAID).unwrap();
    let rows = lab.project_at(&snap, &entries);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].compacted, 3, "entries 001–003 are proven deleted");
    assert!(
        rows[0].verdicts.is_empty(),
        "nothing surviving is guessed at"
    );
    assert_eq!(rows[0].response.as_deref(), Some("done, tests green"));
}

/// A fan's candidates each project their own row, bound by their own fire —
/// and a figure is per conversation tree, so nothing bleeds between them.
#[test]
fn each_candidate_carries_its_own_binding_and_figures() {
    let lab = Lab::new();
    let obligation = crate::fan::Obligation {
        project: "proj".to_owned(),
        ball: Some(BALL.to_owned()),
    };
    let candidates = crate::fan::open(&obligation, &lab.project.path, &lab.xdg, 2).unwrap();
    let entries = trail(
        &lab.ws,
        &lab.project.path,
        &[
            (CONV, &candidates[0].worktree),
            ("other-two", &candidates[1].worktree),
        ],
    );
    let snap = snap(
        &lab.ws,
        &lab.project.path,
        vec![named_agent()],
        vec![bill(AGENT, "001", 5, 9)],
    );
    super::worktree(&lab.ws, AGENT, "try it one way", &[]);
    let rows = lab.project_at(&snap, &entries);
    assert_eq!(rows.len(), 3, "the claim and both candidates: {rows:?}");
    // The claim row's worktree is not what either fire bound, so it is bound to
    // nothing at all — and says so rather than borrowing a candidate's facts.
    assert_eq!(rows[0].conversation, None);
    assert_eq!(rows[0].goal, None);
    assert!(rows[0].pins.is_empty());
    assert_eq!(rows[0].steps, 0);
    // The first candidate resolves to the named agent; the second's minted name
    // belongs to no derived branch yet, which is a reading and not an error.
    assert_eq!(
        rows[1].diff.handle.as_deref(),
        Some(candidates[0].handle.as_str())
    );
    assert_eq!(rows[1].conversation.as_deref(), Some(AGENT));
    assert_eq!(rows[1].usage.input_tokens, 5);
    assert_eq!(rows[1].wall_secs, 9);
    assert_eq!(rows[1].goal.as_deref(), Some("try it one way"));
    assert_eq!(rows[2].conversation, None);
    assert_eq!(rows[2].usage.input_tokens, 0);
    assert!(rows[2].verdicts.is_empty());
    assert_eq!(rows[2].response, None);
}
