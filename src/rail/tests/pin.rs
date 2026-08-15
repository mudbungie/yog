//! **S10-T3 notch-pin**: one selection folds the whole inspector to one
//! commit — the transcript cut to what that call read, the budget folded to
//! that point — and no selection leaves every tab on today's read.

use super::{chat, commit, step, steps};
use crate::rail::{build, pin, transcript_as_of};
use crate::transcript::{Entry, EntryKind, Transcript};

fn rail() -> crate::rail::Rail {
    build(
        "storeroom",
        &[commit("aaaa1111", 10), commit("bbbb2222", 20)],
        &steps(vec![
            step("001", Some("aaaa1111"), 5),
            step("002", Some("bbbb2222"), 7),
            step("003", None, 11),
        ]),
        &chat(3),
        &[],
    )
}

fn delivered(name: &str) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: b"hello".to_vec(),
        kind: EntryKind::Delivered {
            sender: "user".to_owned(),
            epitaph: None,
            body: "hello".to_owned(),
        },
    }
}

fn spoken(name: &str) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: b"[]".to_vec(),
        kind: EntryKind::Model {
            model_id: "m".to_owned(),
            blocks: vec![],
            usage: crate::transcript::Usage::new(),
        },
    }
}

/// A pinned notch names its commit, states the budget as of that point, and
/// carries the cut its chat seat decided — so the line in the chat and the
/// prefix behind it can never disagree.
///
/// **The fold is the BUILD's, and the pin only selects** (REMOTE §9.7,
/// bl-44e9): the running rollup is on every notch of the answer, so what a seat
/// resolving a pin does is read one field off the notch it picked. Both halves
/// are asserted, because a pin that summed the prefix itself would pass the
/// second alone.
#[test]
fn a_pinned_notch_names_its_commit_and_states_the_budget_as_of_it() {
    let rail = rail();
    let budgets: Vec<u64> = rail.notches.iter().map(|n| n.budget).collect();
    assert_eq!(
        budgets,
        [5, 12, 23],
        "every notch carries the spend up to and including itself"
    );
    let pinned = pin(&rail, Some(1)).expect("notch 1 pins");
    assert_eq!(pinned.commit, "bbbb2222");
    assert_eq!(pinned.short, "bbbb222");
    assert_eq!(pinned.tokens, 12);
    assert_eq!(pinned.cut, 3);
}

/// Three ways a pin declines, all one answer — today's read: nothing selected,
/// an index the spine no longer has, and a notch whose step recorded no commit
/// (there is no tree behind it to pin to).
#[test]
fn a_pin_with_no_tree_behind_it_declines() {
    let rail = rail();
    assert!(pin(&rail, None).is_none());
    assert!(pin(&rail, Some(99)).is_none());
    assert!(pin(&rail, Some(2)).is_none());
}

/// The transcript as of a notch is what that call read: everything ahead of
/// that call's own output, and none of what it went on to produce.
#[test]
fn the_transcript_cuts_at_what_that_call_read() {
    let tx = Transcript {
        entries: vec![
            delivered("001-user.md"),
            spoken("002-model.json"),
            delivered("003-user.md"),
            delivered("004-peer.md"),
            spoken("005-model.json"),
        ],
    };
    assert_eq!(transcript_as_of(&tx, 1).entries.len(), 1);
    let second = transcript_as_of(&tx, 4);
    assert_eq!(second.entries.len(), 4);
    // Two drains with no call between them are ONE run under one notch: the
    // second delivered entry does not open a notch of its own.
    assert_eq!(
        second.entries.get(3).map(|e| e.name.clone()),
        Some("004-peer.md".to_owned())
    );
}

/// A cut the transcript cannot honour reads whole — the general path with an
/// out-of-range input, not an arm of its own.
#[test]
fn a_cut_the_transcript_cannot_honour_reads_whole() {
    let tx = Transcript {
        entries: vec![delivered("001-user.md"), spoken("002-model.json")],
    };
    assert_eq!(transcript_as_of(&tx, 9).entries.len(), 2);
    assert_eq!(transcript_as_of(&Transcript::default(), 3).entries.len(), 0);
}

/// The cut is a prefix of the same entries, byte for byte — which is what lets
/// the Raw toggle keep showing the pinned tree's bytes with no `git show`.
#[test]
fn the_cut_keeps_the_entries_bytes_unaltered() {
    let tx = Transcript {
        entries: vec![delivered("001-user.md"), spoken("002-model.json")],
    };
    let cut = transcript_as_of(&tx, 1);
    assert_eq!(
        cut.entries.first().map(|e| e.raw.clone()),
        Some(b"hello".to_vec())
    );
}
