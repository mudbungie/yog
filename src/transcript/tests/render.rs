//! Headless shape-walk tests for the transcript widget — assert user-visible
//! text lands in the paint output. The tool rows are [`tools`], split off at
//! the cap: they are the only ones whose subject is a *repaint schedule* rather
//! than a string, so they carry a probe of their own.

mod tools;

use crate::transcript::{AutoExpand, Block, Entry, EntryKind, Transcript, Usage};
use std::collections::HashSet;

/// A screen big enough that the `ScrollArea` lays out all rows.
pub(super) use crate::paint_probe::screen as input;

/// Run the widget headlessly and concatenate every painted galley's text,
/// under the default auto-state and no fold overrides.
pub(super) fn rendered_text(t: &Transcript, raw: bool) -> String {
    painted_with(t, raw, AutoExpand::default(), &mut HashSet::new())
}

/// The same, with the auto-state knobs and the fold-override set spelled out.
pub(super) fn painted_with(
    t: &Transcript,
    raw: bool,
    auto: AutoExpand,
    folds: &mut HashSet<String>,
) -> String {
    crate::paint_probe::paint(|ui| super::plain(ui, t, raw, auto, folds))
}

pub(super) fn entry(kind: EntryKind) -> Entry {
    Entry {
        name: "001-x.json".into(),
        raw: b"RAWBYTES".to_vec(),
        kind,
    }
}

pub(super) fn tx(entries: Vec<Entry>) -> Transcript {
    Transcript { entries }
}

#[test]
fn empty_transcript_shows_placeholder() {
    assert!(rendered_text(&Transcript::default(), false).contains("(no messages yet)"));
}

#[test]
fn delivered_renders_sender_and_body() {
    let t = tx(vec![entry(EntryKind::Delivered {
        sender: "alice".into(),
        epitaph: None,
        body: "hello world".into(),
    })]);
    let painted = rendered_text(&t, false);
    assert!(painted.contains("alice:"), "got:\n{painted}");
    assert!(painted.contains("hello world"));
}

#[test]
fn model_text_and_thinking_render() {
    let t = tx(vec![entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![
            Block::Text("visible answer".into()),
            Block::Thinking("private reasoning".into()),
        ],
    })]);
    let painted = rendered_text(&t, false);
    // bl-2335: the speaker seat carries the agent's name, never the model id.
    assert!(
        painted.contains(&format!("{}:", super::rows::SPEAKER)),
        "got:\n{painted}"
    );
    assert!(!painted.contains("opus:"), "got:\n{painted}");
    assert!(painted.contains("visible answer"));
    assert!(painted.contains("thinking:"), "got:\n{painted}");
    assert!(painted.contains("private reasoning"));
}

#[test]
fn streaming_tail_renders_live_marker() {
    let t = tx(vec![entry(EntryKind::Streaming {
        thinking: String::new(),
        text: "partial output".into(),
    })]);
    let painted = rendered_text(&t, false);
    assert!(painted.contains("live"));
    assert!(painted.contains("partial output"));
}

/// The §11 role stripe (bl-3acb): the render paints each speaking row's role
/// hue at its edge — user, model, peer and result deposit read apart at a
/// glance — and a machinery-only transcript paints no role hue at all.
#[test]
fn each_role_paints_its_stripe_and_machinery_paints_none() {
    use crate::theme::{BRAZEN, BRAZEN_DIM, GATE, SPECTRE};
    let delivered = |sender: &str, epitaph| EntryKind::Delivered {
        sender: sender.into(),
        epitaph,
        body: "words".into(),
    };
    let t = tx(vec![
        entry(delivered("user", None)),
        entry(EntryKind::Model {
            model_id: "opus".into(),
            usage: Usage::default(),
            blocks: vec![Block::Text("reply".into())],
        }),
        entry(delivered("peer", None)),
        entry(delivered("kid", Some(crate::inboxview::Epitaph::Stopped))),
    ]);
    let fills = crate::paint_probe::paint_fills(|ui| {
        super::plain(ui, &t, false, AutoExpand::default(), &mut HashSet::new());
    });
    for hue in [GATE, SPECTRE, BRAZEN, BRAZEN_DIM] {
        assert!(fills.contains(&hue), "a role stripe must paint {hue:?}");
    }
    let machinery = tx(vec![entry(EntryKind::ToolResult {
        tool_use_id: "t".into(),
        content: "bytes".into(),
        is_error: false,
    })]);
    let fills = crate::paint_probe::paint_fills(|ui| {
        super::plain(
            ui,
            &machinery,
            false,
            AutoExpand::default(),
            &mut HashSet::new(),
        );
    });
    for hue in [GATE, SPECTRE, BRAZEN, BRAZEN_DIM] {
        assert!(!fills.contains(&hue), "machinery wears the empty seat");
    }
}

#[test]
fn raw_kind_renders_bytes_in_parsed_mode() {
    let painted = rendered_text(&tx(vec![entry(EntryKind::Raw)]), false);
    assert!(painted.contains("RAWBYTES"), "got:\n{painted}");
    assert!(painted.contains("001-x.json"));
}

#[test]
fn raw_toggle_shows_verbatim_bytes_over_parsed() {
    let t = tx(vec![entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![Block::Text("parsed answer".into())],
    })]);
    // Parsed mode shows the answer, not the backing bytes.
    let parsed = rendered_text(&t, false);
    assert!(parsed.contains("parsed answer"));
    assert!(!parsed.contains("RAWBYTES"));
    // Raw mode shows the verbatim bytes and filename instead.
    let raw = rendered_text(&t, true);
    assert!(raw.contains("RAWBYTES"), "got:\n{raw}");
    assert!(raw.contains("001-x.json"));
    assert!(!raw.contains("parsed answer"));
}
