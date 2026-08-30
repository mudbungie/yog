//! **The pairing bl-1802 corrected**: which chat row each notch's rule paints
//! above, and how far back its pin cuts.
//!
//! The premise these tests retire — shipped in bl-929d and bl-98da alike — was
//! that the i-th maximal run of delivered `.md` entries feeds the i-th step. It
//! does not: a litany step is one model call and a tool loop is many of them
//! behind one delivered run, so the two sets were never the same set. What
//! pairs one for one is a **model-output entry per completed step**, which is
//! what every case below asserts.

use super::{chat, commit, seat, step, steps};
use crate::git_tree::Framing;
use crate::rail::{Rail, build, pin};
use crate::steps_view::{StepSummary, StepsView};
use crate::transcript::{Block, Entry, EntryKind, Transcript, Usage};

fn entry(name: &str, kind: EntryKind) -> Entry {
    Entry {
        name: name.to_owned(),
        raw: b"x".to_vec(),
        kind,
    }
}

fn delivered(name: &str) -> Entry {
    entry(
        name,
        EntryKind::Delivered {
            sender: "user".to_owned(),
            epitaph: None,
            body: "hi".to_owned(),
        },
    )
}

fn spoke(name: &str) -> Entry {
    entry(
        name,
        EntryKind::Model {
            model_id: "opus".to_owned(),
            blocks: vec![Block::ToolUse {
                id: "t1".to_owned(),
                name: "bash".to_owned(),
                input_summary: String::new(),
            }],
            usage: Usage::new(),
        },
    )
}

fn tool(name: &str) -> Entry {
    entry(
        name,
        EntryKind::ToolResult {
            tool_use_id: "t1".to_owned(),
            content: "ok".to_owned(),
            is_error: false,
        },
    )
}

fn framed(seq: &str, oid: &str, framing: Framing) -> StepSummary {
    StepSummary {
        framing,
        ..step(seq, Some(oid), 0)
    }
}

fn spine(tx: &Transcript, view: StepsView) -> Rail {
    build("root", &[], &view, tx, &[])
}

/// The bug, stated as a test. A turn that calls two tools is **three** steps
/// behind **one** delivered run; the old ordinal alignment handed the second
/// delivered run the second step's commit, which is the read state of a call
/// three messages earlier. Each step's rule now sits above the run of entries
/// that call read and nobody before it had — the tool results included.
#[test]
fn a_tool_loop_gives_every_call_its_own_rule_not_the_next_drains() {
    let tx = Transcript {
        entries: vec![
            delivered("001-user.md"),
            spoke("002-opus.json"),
            tool("003-tool.json"),
            spoke("004-opus.json"),
            tool("005-tool.json"),
            spoke("006-opus.json"),
            delivered("007-user.md"),
            spoke("008-opus.json"),
        ],
    };
    let rail = spine(
        &tx,
        steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
            step("003", Some("cccc3333"), 0),
            step("004", Some("dddd4444"), 0),
        ]),
    );
    let rules = rail.rules();
    assert_eq!(rules.get("tx/001-user.md#0"), Some(&0));
    assert_eq!(rules.get("tx/003-tool.json#0"), Some(&1));
    assert_eq!(rules.get("tx/005-tool.json#0"), Some(&2));
    // The second delivered run belongs to step 004 — the call that read it —
    // and never to step 002, which had answered three entries earlier.
    assert_eq!(rules.get("tx/007-user.md#0"), Some(&3));
    assert_eq!(rules.len(), 4, "one rule per step: {rules:?}");
}

/// Two drains with no call between them are one run under one rule: both
/// batches entered the same prompt, and a line apiece would claim a boundary no
/// model call observed.
#[test]
fn two_drains_with_no_call_between_them_share_one_rule() {
    let tx = Transcript {
        entries: vec![
            delivered("001-user.md"),
            spoke("002-opus.json"),
            delivered("003-user.md"),
            delivered("004-peer.md"),
            spoke("005-opus.json"),
        ],
    };
    let rail = spine(
        &tx,
        steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
        ]),
    );
    let rules = rail.rules();
    assert_eq!(rules.len(), 2, "{rules:?}");
    assert_eq!(rules.get("tx/003-user.md#0"), Some(&1));
}

/// The pin's cut is the same walk's other half: everything ahead of that
/// call's own output, so a pinned notch shows exactly what the call read.
#[test]
fn the_pin_cuts_to_what_that_call_read() {
    let tx = chat(3);
    let rail = spine(
        &tx,
        steps(vec![
            step("001", Some("aaaa1111"), 5),
            step("002", Some("bbbb2222"), 7),
            step("003", Some("cccc3333"), 9),
        ]),
    );
    assert_eq!(pin(&rail, Some(0)).map(|p| p.cut), Some(1));
    assert_eq!(pin(&rail, Some(1)).map(|p| p.cut), Some(3));
    let second = crate::rail::transcript_as_of(&tx, 3);
    let names: Vec<&str> = second.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["001-user.md", "002-opus.json", "003-user.md"]);
    assert_eq!(seat(1), "tx/003-user.md#0");
}

/// A call that sealed nothing and is still running marks the tail of the chat:
/// its read state is everything committed so far, which is where a child
/// dispatched right now hangs its card.
#[test]
fn the_running_call_marks_the_tail_of_the_chat() {
    let tx = Transcript {
        entries: vec![
            delivered("001-user.md"),
            spoke("002-opus.json"),
            tool("003-tool.json"),
        ],
    };
    let rail = spine(
        &tx,
        steps(vec![
            step("001", Some("aaaa1111"), 0),
            framed("002", "bbbb2222", Framing::Killed),
        ]),
    );
    assert_eq!(rail.rules().get("tx/003-tool.json#0"), Some(&1));
    assert_eq!(pin(&rail, Some(1)).map(|p| p.cut), Some(3));
}

/// A call that sealed nothing and was then superseded left no output to sit
/// above, so it has no seat and no gesture can reach it — the revival's own
/// rule is the next line down, carrying the commit that includes the delivery
/// which revived the branch.
#[test]
fn a_superseded_call_that_sealed_nothing_has_no_seat() {
    let tx = Transcript {
        entries: vec![
            delivered("001-user.md"),
            delivered("002-user.md"),
            spoke("003-opus.json"),
        ],
    };
    let rail = spine(
        &tx,
        steps(vec![
            framed("001", "aaaa1111", Framing::Killed),
            step("002", Some("bbbb2222"), 0),
        ]),
    );
    let rules = rail.rules();
    assert_eq!(rules.len(), 1, "{rules:?}");
    assert_eq!(rules.get("tx/001-user.md#0"), Some(&1));
    assert!(pin(&rail, Some(0)).is_none(), "no seat, no pin");
}

/// A step the transcript has no output for yet — the call is in flight and the
/// entries it will seal are not committed — takes no seat, which is bl-929d's
/// *absence of a commit = no line* generalized: absence of a place = no rule.
#[test]
fn a_step_the_chat_has_not_caught_up_with_takes_no_seat() {
    let empty = spine(
        &Transcript::default(),
        steps(vec![step("001", Some("aaaa1111"), 0)]),
    );
    assert!(empty.rules().is_empty());
    let ahead = spine(
        &chat(1),
        steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
        ]),
    );
    let rules = ahead.rules();
    assert_eq!(rules.len(), 1, "{rules:?}");
    assert_eq!(rules.get(&seat(0)), Some(&0));
    assert_eq!(commit("aaaa1111", 1).oid, "aaaa1111");
}
