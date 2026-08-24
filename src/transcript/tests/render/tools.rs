//! **The tool rows** (§11) — the call, its result, and the pulse the one still
//! running pulls. Split off [`super`] at the cap on the seam they already had:
//! every other beat there asks what text reached the glass, and these ask what
//! the frame **scheduled** — an in-flight call must pull a near-term repaint
//! and a resolved one must pull none, which no painted string can say.

use std::collections::HashSet;

use super::{entry, input, rendered_text, tx};
use crate::transcript::{AutoExpand, Block, Entry, EntryKind, Transcript, Usage};

/// Second-frame steady-state repaint delay (egui returns 0 on frame one).
fn repaint_delay(t: &Transcript) -> std::time::Duration {
    let ctx = egui::Context::default();
    let auto = AutoExpand::default();
    let mut folds = HashSet::new();
    for _ in 0..2 {
        let _ = ctx.run(input(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::super::plain(ui, t, false, auto, &mut folds);
            });
        });
    }
    ctx.run(input(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::super::plain(ui, t, false, auto, &mut folds);
        });
    })
    .viewport_output
    .get(&egui::ViewportId::ROOT)
    .expect("root viewport output present")
    .repaint_delay
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
