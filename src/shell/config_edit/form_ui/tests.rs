//! The measured half of the §9.5 renderer (bl-76f8, bl-2622): what a value box
//! and a raw editor actually let through at a given pane width. Both defects
//! are invisible to every text assertion — the same characters go into the
//! galley either way — so the assertions are over the painted, clipped runs
//! `crate::paint_probe` collects, never over a galley's own string.

use super::{raw_editor, render};
use crate::config_edit::form::{Control, Group, Row};

/// A row whose value is longer than any fixed column.
fn long_row() -> Vec<Group> {
    vec![Group {
        entry: "worker".into(),
        rows: vec![Row {
            entry: "worker".into(),
            field: "tools",
            control: Control::List,
            help: "the tool names this role may call",
            // Long enough that a 700 pt pane genuinely cannot show it and
            // a 2560 pt one can — the two ends of QUALITY §2's shot sheet.
            value: "bash, read_file, write_file, load_skill, edit_file, list_dir, \
                    search_files, run_tests, git_status, git_commit, fetch_url, \
                    dispatch, message"
                .into(),
            fault: None,
        }],
    }]
}

/// The widest run of text actually ON the glass — each galley narrowed to
/// the part its clip rect let through. A `TextEdit` lays its value out
/// unwrapped whatever its box width, so the galley's own size says nothing
/// about the seat; what the operator can read is the visible intersection.
fn widest_visible(out: &egui::FullOutput) -> f32 {
    let mut widest: f32 = 0.0;
    for clipped in &out.shapes {
        let mut here = Vec::new();
        crate::paint_probe::collect(&clipped.shape, &mut here);
        for (_, rect) in here {
            let seen = rect.intersect(clipped.clip_rect);
            if seen.height() > 0.5 {
                widest = widest.max(seen.width());
            }
        }
    }
    widest
}

/// Render the form into a pane `w` points wide and report the widest run
/// of value text visible in it.
fn value_run(w: f32) -> f32 {
    let ctx = egui::Context::default();
    let groups = long_row();
    let out = ctx.run(crate::paint_probe::screen_sized(w, 600.0), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, "t", &groups, &[]);
        });
    });
    widest_visible(&out)
}

/// **A value box is a share of its row, not a constant** (bl-76f8, the
/// width-axis twin of §11 rule 5). egui's `text_edit_width` is a fixed
/// 280 pt column, so at a maximized 2560 pt window the then-`capabilities`
/// row (a `models:` field until bl-3ffa) read `tool_use_native,
/// prompt_caching, streaming, stop_` — cut mid-token, with ~1700 pt of pane
/// unused beside it. Measured, because
/// the same characters go INTO the galley either way: what changes is how
/// much of it the box lets through.
#[test]
fn a_value_box_grows_with_the_pane_it_is_in() {
    let (narrow, wide, whole) = (value_run(700.0), value_run(2560.0), value_run(6000.0));
    // The whole value is legible at the maximized window — the G1 half of
    // the finding. Before: 281 pt of it, cut mid-token at `stop_`, with
    // ~1700 pt of pane unused beside it.
    assert!(
        wide + 1.0 >= whole,
        "the value is still cut at a maximized window: {wide} pt of {whole} pt"
    );
    // And the box is a share, not a constant: a pane too narrow to show
    // the value still cuts it, so this measurement can fail — a box fixed
    // at any width would read the same at both and prove nothing.
    assert!(
        narrow < whole && wide > narrow,
        "the value box ignored the pane: {narrow} pt at 700, {wide} pt at 2560, \
         {whole} pt laid out whole"
    );
}

/// The widest galley one frame painted, in points.
fn widest(shape: &egui::Shape) -> f32 {
    match shape {
        egui::Shape::Text(t) => t.galley.size().x,
        egui::Shape::Vec(v) => v.iter().map(widest).fold(0.0, f32::max),
        _ => 0.0,
    }
}

/// **A raw editor lays its lines out across the pane, not down a column**
/// (bl-2622). Measured, because the defect is invisible to every text
/// assertion — the same characters paint either way, just wrapped into
/// egui's 280 pt `text_edit_width` default while the pane had a thousand.
#[test]
fn a_raw_editor_takes_the_pane_width() {
    let mut text = format!("key = \"{}\"", "x".repeat(400));
    let ctx = egui::Context::default();
    let out = ctx.run(crate::paint_probe::screen(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| raw_editor(ui, &mut text));
    });
    let w = out
        .shapes
        .iter()
        .map(|c| widest(&c.shape))
        .fold(0.0, f32::max);
    assert!(
        w > 500.0,
        "the raw text laid out {w} pt wide — back in the fixed column"
    );
}
