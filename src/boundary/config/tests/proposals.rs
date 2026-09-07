//! **The §9.6 learning-loop pair at the chokepoint** (§9.6; REMOTE §9.22,
//! bl-dd88): the read answered off a real workspace's refs, and the settle
//! spawned as the workspace-bound `litany proposal` it is.
//!
//! Both go through the same `answer`/`dispatch` a deposit and a wire request
//! enter, so what is proven is the gesture rather than a private helper. The
//! read's derivation is real git — freshness is two commits compared, and a
//! fixture that faked either would be asserting on its own arithmetic — and
//! the settle's spawn is a recorded script, because litany's compare-and-swap
//! is litany's to prove and what this owes is that the right argv reaches it.

use super::{ask, deps_at, fire, quiet, script, seeing};
use crate::boundary::config::{Read, Write};
use crate::boundary::reply::{ConfigAnswer, Reply};
use crate::boundary::{Action, Query};
use crate::proposals::{Settle, Verdict};
use crate::test_support::workspace::{seed_proposal, seed_workspace_config};
use std::path::Path;
use tempfile::TempDir;

/// A workspace on disk with one staged proposal, and deps that can address it.
fn staged(root: &Path) -> (TempDir, crate::boundary::dispatch::Deps) {
    let ws = tempfile::tempdir().expect("tmp");
    seed_workspace_config(ws.path(), &[("workflow.yaml", "events: {}\n")]);
    seed_proposal(
        ws.path(),
        "20260906T090000Z-r001",
        "config/default",
        "souls: the apk cache is worth keeping",
        &[("souls/worker.md", "keep it\n")],
    );
    let deps = seeing(&quiet(root), &[ws.path()]);
    (ws, deps)
}

/// **The beat this ball exists for.** A staged proposal is readable from the
/// one chokepoint every seat enters — the listing, and the proposal whole when
/// the read names it. Before this there was no gesture at all.
#[test]
fn the_staged_proposals_answer_at_the_chokepoint() {
    let root = tempfile::tempdir().expect("tmp");
    let (ws, deps) = staged(root.path());
    let name = crate::naming::leaf(ws.path());

    let Ok(Reply::Config(ConfigAnswer::Proposals(listing))) = ask(
        &deps,
        &Query::Config(Read::Proposals {
            workspace: name.clone(),
            id: None,
        }),
    ) else {
        panic!("a proposals read answers the family's carrier");
    };
    assert_eq!(listing.rows.len(), 1, "{:?}", listing.rows);
    assert!(listing.rows[0].fresh, "the lineage still stands there");
    assert_eq!(listing.whole, None, "the bare read names none");

    let Ok(Reply::Config(ConfigAnswer::Proposals(one))) = ask(
        &deps,
        &Query::Config(Read::Proposals {
            workspace: name,
            id: Some("20260906T090000Z-r001".to_owned()),
        }),
    ) else {
        panic!("naming one answers the same carrier");
    };
    assert!(
        one.whole.is_some_and(|w| w.contains("apk cache")),
        "the proposal itself, message and diff"
    );
}

/// A workspace whose repository holds no `proposal/*` ref answers an empty
/// listing — the general path with no input, never a refusal.
#[test]
fn a_workspace_with_nothing_staged_answers_emptily() {
    let root = tempfile::tempdir().expect("tmp");
    let ws = tempfile::tempdir().expect("tmp");
    seed_workspace_config(ws.path(), &[("workflow.yaml", "events: {}\n")]);
    let deps = seeing(&quiet(root.path()), &[ws.path()]);
    let Ok(Reply::Config(ConfigAnswer::Proposals(view))) = ask(
        &deps,
        &Query::Config(Read::Proposals {
            workspace: crate::naming::leaf(ws.path()),
            id: None,
        }),
    ) else {
        panic!("an empty listing is still a listing");
    };
    assert!(view.rows.is_empty());
}

/// A workspace whose repository cannot be read refuses in git's own words
/// rather than answering "nothing staged" — the same forgiveness split the
/// §9.3 browse draws, and the reason a defective workspace is never silently
/// an empty one.
#[test]
fn an_unreadable_workspace_refuses_rather_than_reading_empty() {
    let root = tempfile::tempdir().expect("tmp");
    let ws = tempfile::tempdir().expect("tmp");
    let deps = seeing(&quiet(root.path()), &[ws.path()]);
    assert!(
        ask(
            &deps,
            &Query::Config(Read::Proposals {
                workspace: crate::naming::leaf(ws.path()),
                id: None,
            }),
        )
        .is_err()
    );
}

/// **The settle is the workspace-bound `litany proposal` it claims to be**: the
/// argv the act spawns is the verb, the workspace, the id and the verdict's own
/// flag — which is what leaves litany's compare-and-swap the one home for the
/// act itself.
#[test]
fn the_settle_spawns_litany_proposal_with_the_verdicts_flag() {
    let root = tempfile::tempdir().expect("tmp");
    let bin = tempfile::tempdir().expect("tmp");
    let litany = script(bin.path(), "litany", "printf '%s\\n' \"$*\"\nexit 0\n");
    let ws = tempfile::tempdir().expect("tmp");
    let deps = seeing(
        &deps_at(root.path(), &litany, Path::new("/definitely/not/a/bl-xyz")),
        &[ws.path()],
    );
    for (verdict, flag) in [(Verdict::Accept, "--accept"), (Verdict::Reject, "--reject")] {
        let reply = fire(
            &deps,
            &Action::Config(Write::Proposal(Settle {
                workspace: crate::naming::leaf(ws.path()),
                id: "20260906T090000Z-r001".to_owned(),
                verdict,
            })),
        )
        .expect("the verb ran");
        let Reply::Outcome(outcome) = reply else {
            panic!("a §8.2 verb answers its captured run");
        };
        assert!(outcome.ok(), "{outcome:?}");
        assert!(outcome.stdout.contains("proposal"), "{}", outcome.stdout);
        assert!(
            outcome.stdout.contains("20260906T090000Z-r001"),
            "{}",
            outcome.stdout
        );
        assert!(outcome.stdout.contains(flag), "{}", outcome.stdout);
    }
}

/// litany's own refusal rides back verbatim — a stale proposal, an ambiguous
/// one and an unknown id are all sentences only litany can write, and the
/// settle's whole job is to carry them rather than to re-derive them. It is the
/// captured run and **not** an `Err`, exactly as every other §8.2 verb's
/// non-zero exit is: the refusal is the product, and a trail row carries it.
#[test]
fn litanys_refusal_is_the_answer() {
    let root = tempfile::tempdir().expect("tmp");
    let bin = tempfile::tempdir().expect("tmp");
    let litany = script(
        bin.path(),
        "litany",
        "printf 'proposal x is stale\\n' 1>&2\nexit 1\n",
    );
    let ws = tempfile::tempdir().expect("tmp");
    let deps = seeing(
        &deps_at(root.path(), &litany, Path::new("/definitely/not/a/bl-xyz")),
        &[ws.path()],
    );
    let reply = fire(
        &deps,
        &Action::Config(Write::Proposal(Settle {
            workspace: crate::naming::leaf(ws.path()),
            id: "20260906T090000Z-r001".to_owned(),
            verdict: Verdict::Accept,
        })),
    )
    .expect("a captured run, however it exited");
    let Reply::Outcome(outcome) = reply else {
        panic!("a §8.2 verb answers its captured run");
    };
    assert!(!outcome.ok(), "{outcome:?}");
    assert!(outcome.stderr.contains("stale"), "{}", outcome.stderr);
}
