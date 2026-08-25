//! **The new-ball form** (§8.1), split off [`super`] at §12's budget on the
//! seam the balls section is built from: every other affordance there acts on a
//! ball that already exists — ▶ Start and `assign →` on a ready one, ▶ Continue
//! on a bound one — while this one files a ball that does not, out of two RAM
//! drafts keyed by project path. It ends in the same
//! [`run_prepare`](super::run_prepare) they all do, which is where the flow
//! itself lives.

use crate::AppModel;
use std::path::Path;

use super::{ShellState, run_prepare};

/// A per-project new-ball form (§8.1): title + body RAM drafts and a
/// Create-&-Start button that mints the ball and enters the start flow.
///
/// Headed by the project's `label`, not its path (§11, bl-ac3d): an
/// `egui::CollapsingHeader` lays its text `TextWrapMode::Extend` whatever the
/// panel's own wrap mode says, so this one row escaped the bl-9669 truncation
/// and sized the whole left column to an absolute path. The full path stays on
/// hover, and the header is keyed by the **path** rather than by its text, so
/// two projects can never share fold state however their labels elide.
pub(crate) fn new_ball_form(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    project: &Path,
    label: &str,
) {
    let (mut title, mut body) = state
        .start
        .new_ball
        .get(project)
        .cloned()
        .unwrap_or_default();
    let mut create = false;
    egui::CollapsingHeader::new(format!("+ new ball · {label}"))
        .id_salt(project)
        .show(ui, |ui| {
            // Hints from the covered [`crate::actions::new_ball_hints`]
            // (bl-b2ed) — empty, the two boxes were indistinguishable.
            let hints = crate::actions::new_ball_hints();
            ui.add(egui::TextEdit::singleline(&mut title).hint_text(hints.title))
                .on_hover_text(
                    "The new ball's title — the one line it is listed by. Typed, it is \
                     the words before any flag in `/create <title…>`.",
                );
            ui.add(egui::TextEdit::multiline(&mut body).hint_text(hints.body))
                .on_hover_text(
                    "The new ball's body — the task written out as the agent will read \
                     it. Typed, it is `/create <title…> --body <text…>`.",
                );
            create = ui
                .add_enabled(
                    crate::actions::create_ball_enabled(&title),
                    egui::Button::new("Create & Start"),
                )
                .on_hover_text(
                    "File this ball in the project (`bl create`), claim it for the focused \
                     workspace, and open its goal. Nothing is sent until Send. Typed, it \
                     is `/create <title…>` then `/prepare ball`.",
                )
                .on_disabled_hover_text("give the ball a title first")
                .clicked();
        })
        .header_response
        .on_hover_text(format!(
            "Fold open a form for filing a brand-new ball in {}. No key of its own: Tab \
             reaches it, Space presses it — and `/create <title…>` files one without \
             the form.",
            project.display()
        ));
    if create {
        let inputs = model.new_ball_inputs(project, &title, &body);
        state.start.new_ball.remove(project);
        run_prepare(model, state, inputs);
    } else {
        state
            .start
            .new_ball
            .insert(project.to_path_buf(), (title, body));
    }
}
