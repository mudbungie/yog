//! The §11 one-line row projection: classification, the derived auto-state,
//! the explicit-override flip, and the preview/body split.

use crate::transcript::{
    AutoExpand, Block, Entry, EntryKind, Row, RowClass, Tone, Transcript, Usage, rows,
};
use std::collections::HashSet;

mod labels;

pub(super) fn entry(name: &str, kind: EntryKind) -> Entry {
    Entry {
        name: name.to_string(),
        raw: b"RAWBYTES".to_vec(),
        kind,
    }
}

pub(super) fn tx(entries: Vec<Entry>) -> Transcript {
    Transcript { entries }
}

/// The conversation's §3.3 display name — who a model turn is (bl-2335).
pub(super) const SPEAKER: &str = "shudder-storeroom";

/// Rows under the default knobs and no overrides.
pub(super) fn default_rows(t: &Transcript) -> Vec<Row> {
    rows(t, SPEAKER, AutoExpand::default(), &HashSet::new())
}

pub(super) fn model(blocks: Vec<Block>) -> Entry {
    entry(
        "002-opus.json",
        EntryKind::Model {
            model_id: "opus".into(),
            blocks,
            usage: Usage::default(),
        },
    )
}

#[test]
fn every_block_is_its_own_one_line_row() {
    let t = tx(vec![
        entry(
            "001-user.md",
            EntryKind::Delivered {
                sender: "user".into(),
                epitaph: None,
                body: "do the thing".into(),
            },
        ),
        model(vec![
            Block::Text("on it".into()),
            Block::ToolUse {
                id: "toolu_1".into(),
                name: "Read".into(),
                input_summary: "{}".into(),
            },
        ]),
        entry(
            "003-tool.json",
            EntryKind::ToolResult {
                tool_use_id: "toolu_1".into(),
                content: "bytes".into(),
                is_error: false,
            },
        ),
    ]);
    let got = default_rows(&t);
    // Three entries, four rows: the model message is two things, so two rows.
    let prefixes: Vec<String> = got.iter().map(|r| r.prefix.clone()).collect();
    assert_eq!(
        prefixes,
        vec![
            "user:",
            "shudder-storeroom:",
            "⚙ Read",
            "✔ tool result — ok"
        ],
        "one row per block: {got:?}"
    );
    // Keys are entry-filename + block ordinal, stable across re-reads.
    let keys: Vec<String> = got.iter().map(|r| r.key.clone()).collect();
    assert_eq!(
        keys,
        vec![
            "tx/001-user.md#0",
            "tx/002-opus.json#0",
            "tx/002-opus.json#1",
            "tx/003-tool.json#0"
        ]
    );
}

/// The §11 role stripe (bl-3acb): every speaking row carries the role its
/// committed bytes assert — the reserved `user` token, the model origin, any
/// other sender, the epitaph-bearing result deposit — and machinery carries
/// none. Byte-derived only: nothing here reads the content.
#[test]
fn every_speaking_row_carries_its_byte_derived_role_and_machinery_none() {
    use crate::theme::Role;
    let delivered = |name: &str, sender: &str, epitaph| {
        entry(
            name,
            EntryKind::Delivered {
                sender: sender.into(),
                epitaph,
                body: "words".into(),
            },
        )
    };
    let t = tx(vec![
        delivered("001-user.md", "user", None),
        model(vec![
            Block::Text("on it".into()),
            Block::Thinking("hmm".into()),
        ]),
        delivered("003-peer.md", "peer", None),
        delivered(
            "004-kid.md",
            "kid",
            Some(crate::inboxview::Epitaph::Stopped),
        ),
        entry(
            "005-«live»",
            EntryKind::Streaming {
                thinking: String::new(),
                text: "tail".into(),
            },
        ),
    ]);
    let roles: Vec<Option<Role>> = default_rows(&t).iter().map(|r| r.role).collect();
    assert_eq!(
        roles,
        vec![
            Some(Role::User),
            Some(Role::Model),
            None, // thinking: machinery, nobody speaking
            Some(Role::Peer),
            Some(Role::Ended),
            Some(Role::Model), // the live tail is the agent speaking
        ]
    );
}

#[test]
fn responses_auto_expand_and_everything_else_auto_contracts() {
    let t = tx(vec![
        model(vec![Block::Text("line one\nline two".into())]),
        entry(
            "003-tool.json",
            EntryKind::ToolResult {
                tool_use_id: "t".into(),
                content: "out one\nout two".into(),
                is_error: true,
            },
        ),
    ]);
    let got = default_rows(&t);
    assert_eq!(got[0].class, RowClass::Response);
    assert!(got[0].expanded, "a reply arrives expanded: {got:?}");
    assert_eq!(got[1].class, RowClass::Other);
    assert!(!got[1].expanded, "machinery arrives contracted: {got:?}");
    assert_eq!(got[1].tone, Tone::Bad, "an error result paints ichor");
}

#[test]
fn both_automatics_are_knobs() {
    let t = tx(vec![
        model(vec![Block::Text("a\nb".into())]),
        entry(
            "003-tool.json",
            EntryKind::ToolResult {
                tool_use_id: "t".into(),
                content: "c\nd".into(),
                is_error: false,
            },
        ),
    ]);
    // Inverted knobs invert both automatics — the policy is config, not code.
    let inverted = AutoExpand {
        responses: false,
        others: true,
    };
    let got = rows(&t, SPEAKER, inverted, &HashSet::new());
    assert!(!got[0].expanded, "responses knob off: {got:?}");
    assert!(got[1].expanded, "others knob on: {got:?}");
}

#[test]
fn an_override_flips_that_rows_auto_state_only() {
    let t = tx(vec![model(vec![
        Block::Text("a\nb".into()),
        Block::Thinking("c\nd".into()),
    ])]);
    let mut folds = HashSet::new();
    folds.insert("tx/002-opus.json#0".to_string());
    let got = rows(&t, SPEAKER, AutoExpand::default(), &folds);
    assert!(
        !got[0].expanded,
        "the override contracts the reply: {got:?}"
    );
    assert!(!got[1].expanded, "its neighbour keeps its auto-state");
    // The same override on a contracted row expands it (the flip is symmetric).
    let mut other = HashSet::new();
    other.insert("tx/002-opus.json#1".to_string());
    let got = rows(&t, SPEAKER, AutoExpand::default(), &other);
    assert!(got[0].expanded);
    assert!(
        got[1].expanded,
        "the override expands the thinking: {got:?}"
    );
}

#[test]
fn a_payload_that_fits_one_line_has_no_body_to_fold() {
    let t = tx(vec![model(vec![Block::Text("short".into())])]);
    let got = default_rows(&t);
    assert_eq!(got[0].preview, "short");
    assert!(got[0].body.is_empty(), "nothing to fold: {got:?}");
    // A long single line is clipped in the preview and kept whole in the body.
    let long = "x".repeat(200);
    let t = tx(vec![model(vec![Block::Text(long.clone())])]);
    let got = default_rows(&t);
    assert!(got[0].preview.ends_with('…'), "clipped: {got:?}");
    assert_eq!(got[0].preview.chars().count(), 161);
    assert_eq!(got[0].body, long, "the body keeps every byte");
}

#[test]
fn streaming_is_a_response_and_raw_is_not() {
    let t = tx(vec![
        entry(
            "«live»",
            EntryKind::Streaming {
                thinking: String::new(),
                text: "typing\nmore".into(),
            },
        ),
        entry("junk", EntryKind::Raw),
    ]);
    let got = default_rows(&t);
    assert_eq!(got[0].class, RowClass::Response);
    assert_eq!(got[0].tone, Tone::Live);
    assert!(got[0].expanded, "the live tail is a response: {got:?}");
    assert_eq!(got[1].prefix, "junk", "a raw row is titled by its filename");
    assert_eq!(got[1].preview, "RAWBYTES");
    assert_eq!(got[1].tone, Tone::Weak);
}

#[test]
fn a_delivered_message_is_conversation_and_arrives_expanded() {
    // bl-6ec6: the operator's own turn must be readable without a click —
    // the fold affordance stays, as the opt-in it always was.
    let t = tx(vec![entry(
        "001-user.md",
        EntryKind::Delivered {
            sender: "user".into(),
            epitaph: None,
            body: "does the transcript work\nsecond line".into(),
        },
    )]);
    let got = default_rows(&t);
    assert_eq!(got[0].class, RowClass::Response);
    assert_eq!(got[0].preview, "does the transcript work");
    assert!(got[0].expanded, "a user turn arrives expanded: {got:?}");
    // Membership in the override set flips it shut — collapse is opt-in.
    let folds = HashSet::from([got[0].key.clone()]);
    assert!(!rows(&t, SPEAKER, AutoExpand::default(), &folds)[0].expanded);
}

#[test]
fn rows_are_comparable_clonable_and_printable() {
    let t = tx(vec![model(vec![Block::Text("hi".into())])]);
    let got = default_rows(&t);
    assert_eq!(got.clone(), got, "a row set compares equal to its clone");
    let shown = format!("{got:?} {:?}", AutoExpand::default());
    assert!(shown.contains("Response") && shown.contains("Plain"));
    assert!(shown.contains("responses: true"));
}
