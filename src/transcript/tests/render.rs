//! Headless shape-walk tests for the transcript widget — assert user-visible
//! text lands in the paint output, and the in-progress tool chip pulses.

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

/// Second-frame steady-state repaint delay (egui returns 0 on frame one).
fn repaint_delay(t: &Transcript) -> std::time::Duration {
    let ctx = egui::Context::default();
    let auto = AutoExpand::default();
    let mut folds = HashSet::new();
    for _ in 0..2 {
        let _ = ctx.run(input(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::plain(ui, t, false, auto, &mut folds);
            });
        });
    }
    ctx.run(input(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::plain(ui, t, false, auto, &mut folds);
        });
    })
    .viewport_output
    .get(&egui::ViewportId::ROOT)
    .expect("root viewport output present")
    .repaint_delay
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

fn tool_use_entry() -> Entry {
    entry(EntryKind::Model {
        model_id: "opus".into(),
        usage: Usage::default(),
        blocks: vec![Block::ToolUse {
            id: "toolu_9".into(),
            name: "Read".into(),
            input_summary: r#"{"path":"/x"}"#.into(),
        }],
    })
}

#[test]
fn in_progress_tool_chip_pulses_and_paints() {
    // No matching tool_result → in progress.
    let t = tx(vec![tool_use_entry()]);
    assert!(
        repaint_delay(&t) < std::time::Duration::from_secs(1),
        "in-progress tool must schedule a near-term repaint"
    );
    let painted = rendered_text(&t, false);
    assert!(painted.contains("⚙ Read — running"), "got:\n{painted}");
    assert!(painted.contains(r#"{"path":"/x"}"#));
}

#[test]
fn resolved_tool_chip_is_static() {
    // A matching tool_result elsewhere resolves the call → no pulse.
    let t = tx(vec![
        tool_use_entry(),
        entry(EntryKind::ToolResult {
            tool_use_id: "toolu_9".into(),
            content: "file body".into(),
            is_error: false,
        }),
    ]);
    assert_eq!(
        repaint_delay(&t),
        std::time::Duration::MAX,
        "resolved tools must not pull repaints"
    );
    let painted = rendered_text(&t, false);
    assert!(painted.contains("⚙ Read"), "got:\n{painted}");
    assert!(!painted.contains("running"));
}

#[test]
fn tool_result_renders_content_and_error_glyph() {
    let ok = tx(vec![entry(EntryKind::ToolResult {
        tool_use_id: "t".into(),
        content: "all good".into(),
        is_error: false,
    })]);
    let ok_painted = rendered_text(&ok, false);
    assert!(ok_painted.contains("all good"));
    assert!(
        ok_painted.contains("✔ tool result — ok"),
        "got:\n{ok_painted}"
    );
    let err = tx(vec![entry(EntryKind::ToolResult {
        tool_use_id: "t".into(),
        content: "boom".into(),
        is_error: true,
    })]);
    let err_painted = rendered_text(&err, false);
    assert!(err_painted.contains("boom"));
    assert!(
        err_painted.contains("✖ tool result — error"),
        "got:\n{err_painted}"
    );
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
