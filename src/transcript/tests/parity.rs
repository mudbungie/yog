//! Two claims about *when* the §11 transcript hides things, both asserted on
//! the laid galley by [`super::legible`]'s probe (bl-7654).
//!
//! - **Direct model output arrives open.** The knob was always right
//!   (`AutoExpand::default().responses`); what was never asserted is that the
//!   turn rollup — which folds a segment's machinery into one shut aggregate —
//!   never folds the answer away with it.
//! - **The pre-commit render means what the committed one does.** bl-54f7
//!   (`0ef853e`) made the live tail project two rows keeping their committed
//!   counterparts' `RowClass` *specifically* so the fold knobs mean one thing
//!   on either side of the commit. That intent was stated and never checked.

use std::collections::HashSet;

use super::legible::{SIZES, answer, long, run, seen, whole};
use super::render::{entry, tx};
use crate::transcript::{AutoExpand, Block, Entry, EntryKind, Row, Transcript, Usage, rows};

/// A turn holding one of every row class: a delivered message, a thinking
/// block, a tool call, its result, and the model's answer — the fixture the
/// whole-surface sweep in [`super::legible`] reads too, so the invariant is
/// asserted over the vocabulary rather than over one row kind.
pub(super) fn mixed(payload: &str) -> Transcript {
    tx(vec![
        Entry {
            name: "001-user.md".into(),
            raw: b"x".to_vec(),
            kind: EntryKind::Delivered {
                sender: "user".into(),
                epitaph: None,
                body: payload.to_owned(),
            },
        },
        entry(EntryKind::Model {
            model_id: "opus".into(),
            usage: Usage::default(),
            blocks: vec![
                Block::Thinking(payload.to_owned()),
                Block::ToolUse {
                    id: "toolu_1".into(),
                    name: "Read".into(),
                    input_summary: payload.to_owned(),
                },
            ],
        }),
        Entry {
            name: "003-tool.json".into(),
            raw: b"{}".to_vec(),
            kind: EntryKind::ToolResult {
                tool_use_id: "toolu_1".into(),
                content: payload.to_owned(),
                is_error: false,
            },
        },
        Entry {
            name: "004-opus.json".into(),
            raw: b"{}".to_vec(),
            kind: EntryKind::Model {
                model_id: "opus".into(),
                usage: Usage::default(),
                blocks: vec![Block::Text(payload.to_owned())],
            },
        },
    ])
}

/// **Finding 4 — direct model output arrives open**, through the rollup rather
/// than in spite of it. The machinery really does roll up here (asserted, or
/// the answer was never behind anything), and the answer is still on the glass
/// entire under the shipped defaults.
#[test]
fn a_model_answer_arrives_open_behind_a_rolled_up_turn() {
    let payload = long();
    let t = mixed(&payload);
    for (w, h) in SIZES {
        let painted = seen(&t, AutoExpand::default(), true, w, h);
        assert!(
            painted.iter().any(|s| s.text.contains("inference call")),
            "the machinery must really roll up, or this proves nothing: {:?}",
            painted.iter().map(|s| &s.text).collect::<Vec<_>>()
        );
        assert!(
            painted.iter().filter(|s| s.text == payload).count() >= 1,
            "the answer arrives open and entire at {w}x{h}: {:?}",
            painted
                .iter()
                .map(|s| s.text.chars().count())
                .collect::<Vec<_>>()
        );
    }
}

/// The default knob is what the ruling says it is — read here rather than in
/// prose, so a flip of it fails at the sentence that claims it.
#[test]
fn the_conversation_arrives_expanded_by_default() {
    assert!(AutoExpand::default().responses);
    assert!(!AutoExpand::default().others);
}

/// The live rows and the committed rows for one payload, paired by the prefix
/// each wears — the labels differ on purpose (`live:` vs the §3.3 speaker),
/// which is why the pairing is stated rather than searched for.
///
/// `pairs` names which rows to take, because a shut turn rolls its *thinking*
/// row out of the committed projection entirely (§11's rollup) while the live
/// one has no turn to roll up: the answer is the row both sides always have.
fn pair(auto: AutoExpand, pairs: &[(String, String)]) -> Vec<(Row, Row)> {
    let payload = long();
    let live = tx(vec![entry(EntryKind::Streaming {
        thinking: "reasoning".into(),
        text: payload.clone(),
    })]);
    let committed = tx(vec![entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![
            Block::Thinking("reasoning".into()),
            Block::Text(payload.clone()),
        ],
    })]);
    let folds = HashSet::new();
    let speaker = super::rows::SPEAKER;
    let (a, b) = (
        rows(&live, speaker, auto, &folds),
        rows(&committed, speaker, auto, &folds),
    );
    let of = |set: &[Row], prefix: &str| {
        set.iter()
            .find(|r| r.prefix == prefix)
            .unwrap_or_else(|| panic!("no {prefix} row in {set:?}"))
            .clone()
    };
    pairs
        .iter()
        .map(|(live, committed)| (of(&a, live), of(&b, committed)))
        .collect()
}

/// The answer pair: the live tail's `live:` row against the committed turn's
/// §3.3 speaker row — the one row both sides always have.
fn answer_pair() -> (String, String) {
    ("live:".to_owned(), format!("{}:", super::rows::SPEAKER))
}

/// Both rows the live tail projects, against the two a committed model turn
/// has (§7.2 the thinking ruling): reasoning, then the answer.
fn both() -> Vec<(String, String)> {
    vec![
        ("thinking:".to_owned(), "thinking:".to_owned()),
        answer_pair(),
    ]
}

/// **Finding 5(a) — the same payload has the same class and the same fold on
/// either side of the commit.** These are what the §11 knobs are asked about,
/// so a payload that answered to a different knob while streaming would change
/// shape the frame it committed.
#[test]
fn a_live_row_keeps_its_committed_counterparts_class_and_fold() {
    let open = AutoExpand {
        responses: true,
        others: true,
    };
    for (live, committed) in pair(open, &both()) {
        assert_eq!(live.class, committed.class, "{live:?} vs {committed:?}");
        assert_eq!(
            live.body.is_empty(),
            committed.body.is_empty(),
            "fold availability differs: {live:?} vs {committed:?}"
        );
        assert_eq!(live.preview, committed.preview, "the preview split differs");
        assert!(
            live.expanded && committed.expanded,
            "both open on the knobs"
        );
    }
}

/// **The one thing they may differ on**, kept true rather than assumed: with
/// the knobs shut, `Tone::Live` still auto-expands the streaming rows — the
/// live answer is the show (`rows::expanded_for`) — while the committed rows
/// obey the knobs. Asserted so the parity above can never be "fixed" by taking
/// that away.
#[test]
fn a_live_row_is_open_even_when_the_knobs_are_shut() {
    let shut = AutoExpand {
        responses: false,
        others: false,
    };
    for (live, committed) in pair(shut, &[answer_pair()]) {
        assert!(live.expanded, "the live row is the show: {live:?}");
        assert!(
            !committed.expanded,
            "the committed row folds: {committed:?}"
        );
    }
}

/// **Finding 5(b) — and it reads the same.** The projection agreeing is not
/// evidence about the glass: same glyphs, same ink, both whole, at both window
/// sizes, with the knobs open so the two are in the state they are meant to
/// share.
#[test]
fn the_live_tail_paints_as_its_committed_counterpart_does() {
    let payload = long();
    let open = AutoExpand {
        responses: true,
        others: true,
    };
    let live = tx(vec![entry(EntryKind::Streaming {
        thinking: "reasoning".into(),
        text: payload.clone(),
    })]);
    let committed = answer(&payload);
    for (w, h) in SIZES {
        let (l, c) = (
            seen(&live, open, true, w, h),
            seen(&committed, open, true, w, h),
        );
        let (lt, ct) = (run(&l, "abcdefghij"), run(&c, "abcdefghij"));
        assert_eq!(lt.text, ct.text, "same glyphs on either side of the commit");
        assert_eq!(lt.ink, ct.ink, "same ink on either side of the commit");
        assert!(
            whole(&lt) && whole(&ct),
            "and whole on both sides at {w}x{h}: live {:.0}/{:.0}, committed {:.0}/{:.0}",
            lt.shown.width(),
            lt.laid.width(),
            ct.shown.width(),
            ct.laid.width()
        );
    }
}
