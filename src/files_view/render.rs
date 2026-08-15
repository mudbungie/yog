//! egui widget: the §11 Altitude-2 Files tab — the agent worktree read-only
//! listing plus the selected file's bounded preview.
//!
//! A near-pure function of the [`FilesView`] listing and the caller-built
//! [`Preview`]. The one interaction is intrinsic (click a file row to select
//! it for preview) so — like jsonview's collapse toggle (§11 widget-split
//! exception) — it lives here and is exercised by a simulated-pointer render
//! test; the selection itself is caller-owned RAM ephemera (§5.3). An absent
//! worktree paints as a fact, not an error (§3.5: disposable materialization).

use super::{FileEntry, FilesView, Preview};

/// Two spaces of indent per path level (matches jsonview / the tree view).
const INDENT: &str = "  ";

/// Render the Files tab. `sel` is the caller-owned selection (RAM ephemera) and
/// it names the entry by its **path**, not by a row number: since bl-13f9 the
/// listing is a wire answer and the preview beside it is the same question
/// asked one depth down, so the selection is what the next ask carries. A row
/// number would index a listing that arrived a round trip ago — the Work tab's
/// `work_sel` holds a `WorkFile` for exactly this reason. Clicking a file row
/// sets it, and the caller's next ask brings `preview` back.
pub fn render(
    ui: &mut egui::Ui,
    view: &FilesView,
    preview: Option<&Preview>,
    sel: &mut Option<String>,
) {
    match view {
        FilesView::AbsentWorktree => {
            ui.weak("worktree not materialized — this agent is quiescent (disposable, torn down)");
        }
        FilesView::Present { entries, truncated } => {
            render_present(ui, entries, *truncated, preview, sel);
        }
    }
}

fn render_present(
    ui: &mut egui::Ui,
    entries: &[FileEntry],
    truncated: bool,
    preview: Option<&Preview>,
    sel: &mut Option<String>,
) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if entries.is_empty() {
            ui.weak("(empty worktree)");
        }
        for entry in entries {
            render_row(ui, entry, sel);
        }
        if truncated {
            ui.weak("… (listing truncated)");
        }
        ui.separator();
        render_preview(ui, preview);
    });
}

/// One entry: dirs are inert `name/` labels; files are selectable `name (N B)`
/// rows. Indent is the entry's path depth (root entries flush-left).
fn render_row(ui: &mut egui::Ui, entry: &FileEntry, sel: &mut Option<String>) {
    let depth = entry.rel_path.matches('/').count();
    let name = entry.rel_path.rsplit('/').next().unwrap_or(&entry.rel_path);
    ui.horizontal(|ui| {
        if depth > 0 {
            ui.monospace(INDENT.repeat(depth));
        }
        if entry.is_dir {
            ui.monospace(format!("{name}/"));
        } else {
            let label = format!("{name}  ({} B)", entry.size);
            if ui
                .selectable_label(sel.as_deref() == Some(entry.rel_path.as_str()), label)
                .on_hover_text(
                    "Preview this file's contents below, read-only. No key of its own: \
                     Tab reaches it, Space presses it.",
                )
                .clicked()
            {
                *sel = Some(entry.rel_path.clone());
            }
        }
    });
}

/// The selected file's preview, or a hint when nothing is selected. Text and the
/// leading bytes of a truncated file are shown monospace; a binary/opaque file
/// reports only its size.
fn render_preview(ui: &mut egui::Ui, preview: Option<&Preview>) {
    let Some(preview) = preview else {
        ui.weak("select a file to preview");
        return;
    };
    match preview {
        Preview::Text(text) => {
            ui.monospace(text);
        }
        Preview::Binary { size } => {
            ui.weak(format!("binary file — {size} bytes"));
        }
        Preview::Truncated { text, size } => {
            ui.monospace(text);
            ui.weak(format!(
                "… (preview truncated at {} KiB of {size} bytes)",
                super::PREVIEW_CAP / 1024
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::paint_probe::screen;

    fn file(rel: &str, size: u64) -> FileEntry {
        FileEntry {
            rel_path: rel.into(),
            size,
            is_dir: false,
        }
    }

    fn dir(rel: &str) -> FileEntry {
        FileEntry {
            rel_path: rel.into(),
            size: 0,
            is_dir: true,
        }
    }

    fn paint(view: &FilesView, preview: Option<&Preview>, sel: &mut Option<String>) -> String {
        crate::paint_probe::paint(|ui| render(ui, view, preview, sel))
    }

    #[test]
    fn absent_worktree_paints_as_a_fact() {
        let text = paint(&FilesView::AbsentWorktree, None, &mut None);
        assert!(text.contains("not materialized"), "got:\n{text}");
    }

    #[test]
    fn present_tree_lists_entries_with_dir_suffix_and_preview_hint() {
        let view = FilesView::Present {
            entries: vec![file("goal.md", 12), dir("work"), file("work/x.txt", 3)],
            truncated: false,
        };
        let text = paint(&view, None, &mut None);
        assert!(text.contains("goal.md"), "got:\n{text}");
        assert!(text.contains("work/"), "dir suffix:\n{text}");
        assert!(text.contains("x.txt"), "nested file:\n{text}");
        assert!(text.contains("select a file to preview"));
        assert!(!text.contains("truncated"));
    }

    #[test]
    fn empty_and_truncated_markers_paint() {
        let empty = FilesView::Present {
            entries: Vec::new(),
            truncated: false,
        };
        assert!(paint(&empty, None, &mut None).contains("(empty worktree)"));
        let full = FilesView::Present {
            entries: vec![file("a", 1)],
            truncated: true,
        };
        assert!(paint(&full, None, &mut None).contains("listing truncated"));
    }

    #[test]
    fn preview_variants_each_paint() {
        let view = FilesView::Present {
            entries: vec![file("a", 1)],
            truncated: false,
        };
        assert!(
            paint(&view, Some(&Preview::Text("body text".into())), &mut None).contains("body text")
        );
        let bin = paint(&view, Some(&Preview::Binary { size: 9 }), &mut None);
        assert!(bin.contains("binary file"), "got:\n{bin}");
        assert!(bin.contains('9'));
        let trunc = paint(
            &view,
            Some(&Preview::Truncated {
                text: "leading".into(),
                size: 99999,
            }),
            &mut None,
        );
        assert!(trunc.contains("leading"), "got:\n{trunc}");
        assert!(trunc.contains("preview truncated"));
    }

    /// Two-frame pointer click: frame one lays the row out, frame two delivers
    /// the click on the first (file) row, selecting it.
    #[test]
    fn clicking_a_file_row_selects_it() {
        let view = FilesView::Present {
            entries: vec![file("a.txt", 5), file("b.txt", 5)],
            truncated: false,
        };
        let mut sel: Option<String> = None;
        let ctx = egui::Context::default();
        let run = |input: egui::RawInput, sel: &mut Option<String>| {
            let _ = ctx.run(input, |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| render(ui, &view, None, sel));
            });
        };
        run(screen(), &mut sel);
        let pos = egui::Pos2::new(24.0, 14.0);
        let click = egui::RawInput {
            events: vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            ..screen()
        };
        run(click, &mut sel);
        assert_eq!(
            sel.as_deref(),
            Some("a.txt"),
            "the first file row should select on click, by the path the next ask carries"
        );
    }
}
