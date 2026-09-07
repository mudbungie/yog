//! What a staged proposal reads as, and how the answer is said (bl-dd88).
//!
//! Real git throughout for the derivation: freshness is two commits compared,
//! and a fixture that faked either would be asserting on its own arithmetic.

use super::*;
use crate::test_support::workspace::{
    seed_proposal, seed_workspace_config, seed_workspace_lineage,
};
use serde_json::json;
use tempfile::TempDir;

/// A workspace with a `config/default` lineage and nothing staged.
fn workspace() -> TempDir {
    let dir = tempfile::tempdir().expect("tmp");
    seed_workspace_config(dir.path(), &[("workflow.yaml", "events: {}\n")]);
    dir
}

/// **The beat this ball exists for.** A reviewer's staged patch is readable
/// from the boundary: what it is called, what it is parented on, how big it is
/// and — the fact an operator settles on — whether the lineage still stands
/// where the reviewer read it.
#[test]
fn a_staged_proposal_is_listed_with_the_lineage_it_would_move() {
    let ws = workspace();
    seed_proposal(
        ws.path(),
        "20260906T090000Z-r001",
        "config/default",
        "notes: record what the span taught",
        &[("souls/worker.md", "you learned a thing\n")],
    );
    let view = read(ws.path(), None).expect("a listing");
    assert_eq!(view.rows.len(), 1, "{:?}", view.rows);
    let row = &view.rows[0];
    assert_eq!(row.id, "20260906T090000Z-r001");
    assert_eq!(row.lineages, vec!["default".to_owned()]);
    assert!(row.fresh, "the lineage still stands where it was read");
    assert!(row.diffstat.contains("1 file changed"), "{}", row.diffstat);
    assert_eq!(row.subject, "notes: record what the span taught");
    assert_eq!(row.parent.len(), 8, "the parent, abbreviated");
    assert_eq!(view.whole, None, "the bare read names no proposal");
}

/// **Stale is derived, not stored.** The lineage advancing under a staged
/// proposal is the only thing that changes, and the row reads it the next time
/// it is asked — which is what makes it impossible to list one fresh and then
/// accept it on a field that has gone stale.
#[test]
fn a_lineage_that_moved_leaves_its_proposal_stale() {
    let ws = workspace();
    seed_proposal(
        ws.path(),
        "20260906T090000Z-r001",
        "config/default",
        "notes: one",
        &[("souls/worker.md", "first\n")],
    );
    assert!(read(ws.path(), None).expect("fresh").rows[0].fresh);

    // Somebody else advanced the lineage: the proposal's parent is no longer a
    // head, and nothing about the proposal itself changed.
    seed_proposal(
        ws.path(),
        "advance",
        "config/default",
        "config: move the lineage on",
        &[("workflow.yaml", "events: {}\n# moved\n")],
    );
    let repo = ws.path().join("repo.git").display().to_string();
    let status = crate::git_env::status(crate::git_env::git().args([
        "-C",
        &repo,
        "branch",
        "-f",
        "config/default",
        "proposal/advance",
    ]))
    .expect("advance the lineage");
    assert!(status.success());

    let view = read(ws.path(), None).expect("a listing");
    let row = view
        .rows
        .iter()
        .find(|r| r.id == "20260906T090000Z-r001")
        .expect("still staged");
    assert!(!row.fresh, "its parent is nobody's head now");
    assert!(row.lineages.is_empty(), "which is the same fact, as a pool");
}

/// A proposal parented on a commit two lineages stand on lists both — the
/// choice litany's accept refuses to make for the operator, said as the pool it
/// is rather than as a flag that would have to be read twice.
#[test]
fn a_parent_two_lineages_stand_on_lists_both() {
    let ws = workspace();
    seed_workspace_lineage(ws.path(), "spare");
    seed_proposal(
        ws.path(),
        "20260906T090000Z-r001",
        "config/default",
        "notes: one",
        &[("souls/worker.md", "first\n")],
    );
    let mut lineages = read(ws.path(), None).expect("a listing").rows[0]
        .lineages
        .clone();
    lineages.sort();
    assert_eq!(lineages, vec!["default".to_owned(), "spare".to_owned()]);
}

/// Naming one answers it **whole** — message and diff — beside the listing the
/// seat already holds, so a settle needs no second read to be decided.
#[test]
fn naming_one_answers_it_whole_beside_the_listing() {
    let ws = workspace();
    seed_proposal(
        ws.path(),
        "20260906T090000Z-r001",
        "config/default",
        "notes: record what the span taught",
        &[("souls/worker.md", "you learned a thing\n")],
    );
    let view = read(ws.path(), Some("20260906T090000Z-r001")).expect("one whole");
    assert_eq!(view.rows.len(), 1, "the listing rides along");
    let whole = view.whole.expect("the proposal itself");
    assert!(
        whole.contains("notes: record what the span taught"),
        "{whole}"
    );
    assert!(whole.contains("you learned a thing"), "{whole}");
}

/// A workspace with nothing staged answers an empty listing — the general path
/// with no input. An id naming nothing refuses instead, because a read that
/// named one thing must not answer emptiness for a thing that is not there.
#[test]
fn nothing_staged_lists_nothing_and_a_bad_id_refuses() {
    let ws = workspace();
    assert_eq!(
        read(ws.path(), None).expect("empty"),
        ProposalView::default()
    );
    assert!(read(ws.path(), Some("nobody")).is_err());
}

/// The verdict vocabulary is closed and round-trips: the word the line spells,
/// the word the wire carries and the `litany proposal` flag the act spends are
/// three readings of one table.
#[test]
fn the_verdict_vocabulary_round_trips_and_refuses_a_third_word() {
    for verdict in [Verdict::Accept, Verdict::Reject] {
        assert_eq!(Verdict::parse(verdict.word()), Some(verdict));
        assert!(verdict.flag().starts_with("--"));
        assert!(verdict.flag().ends_with(verdict.word()));
    }
    assert_eq!(Verdict::parse("maybe"), None);
    assert_eq!(Verdict::parse(""), None);
}

/// The answer's spelling round-trips both readings, and `whole`'s absence
/// survives it — absent is the fact, and a null would be a second spelling.
#[test]
fn the_answer_round_trips_and_absence_stays_absence() {
    let listing = ProposalView {
        rows: vec![ProposalRow {
            id: "20260906T090000Z-r001".into(),
            lineages: vec!["default".into()],
            parent: "9f2c1ab4".into(),
            fresh: true,
            diffstat: "1 file changed, 4 insertions(+)".into(),
            subject: "notes: record what the span taught".into(),
        }],
        whole: None,
    };
    let value = wire::reply(&listing);
    assert_eq!(value["kind"], wire::KIND);
    assert!(
        value.get("whole").is_none(),
        "the bare listing names no proposal"
    );
    let o = value.as_object().expect("an object");
    assert_eq!(wire::view_of(o).expect("decodes"), listing);

    let whole = ProposalView {
        whole: Some("commit …\n+ a line\n".into()),
        ..listing
    };
    let value = wire::reply(&whole);
    let o = value.as_object().expect("an object");
    assert_eq!(wire::view_of(o).expect("decodes"), whole);
}

/// The decoder is strict on every row field: each is a derivation the engine
/// made in one pass and a seat cannot re-make, so a missing one is a codec that
/// has drifted rather than a row to be guessed at.
#[test]
fn the_decoder_refuses_a_row_it_cannot_read() {
    let bad = json!({"kind": wire::KIND, "ok": true, "rows": ["not an object"]});
    assert!(wire::view_of(bad.as_object().expect("an object")).is_err());
    let missing = json!({"kind": wire::KIND, "ok": true,
                         "rows": [{"id": "a", "lineages": [], "parent": "p", "fresh": true}]});
    assert!(wire::view_of(missing.as_object().expect("an object")).is_err());
    let nothing = json!({"kind": wire::KIND, "ok": true});
    assert!(wire::view_of(nothing.as_object().expect("an object")).is_err());
}
