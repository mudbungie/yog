//! **What the §9.3 pane is aimed at**, split off [`super`] at §12's budget on
//! the seam that pane's own doc states: it used to be a free-text branch name
//! and a free-text path over an empty body, so a config commit was authored
//! blind against a file nobody had read. Choosing the lineage and choosing the
//! file are what fixed that, and they are one subject — `super` holds the pane
//! that edits and sends, this holds the two rows that say *which file*.
//!
//! Both rows are §11 rule 8 strips (bl-7414): every member is a control of its
//! own and none may be dropped, so a row that cannot fit grows a LINE rather
//! than running off the pane. The git reads happen on these gestures and never
//! per frame (§7.2, bl-ee0a).

use super::super::ConfigState;
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::branch::{config_file, config_tree};
use std::path::Path;

/// The dropdown's escape: a lineage the workspace does not have yet, named in
/// the text field the choice reveals. Every list ends in its own escape (§9.4).
const NEW_LINEAGE: &str = "new lineage…";

/// The lineage choice (§9.3) — one phrase for the dropdown and the name field
/// it reveals (§11 rule 4: a phrase worn twice is a named const).
const LINEAGE_HINT: &str = "Which config branch to write. `default` is the one new conversations in \
     this workspace read; any other name is a lineage of its own. Typed, the lineage is \
     `/config branch|fork|orphan <name> <text…>`.";

/// The file choice (§9.3) — worn by the dropdown and the free-path field.
const FILE_HINT: &str = "Which file inside the config branch to edit — `providers.yaml`, for \
     instance. It is created if the branch has no such file. Typed, the file rides the \
     same `/config` line.";

/// The selected lineage's tree, or nothing when the selection is a lineage the
/// workspace does not have yet (there is no ref to read).
pub(super) fn tree(config: &ConfigState, ws: &Path) -> Vec<String> {
    if !config.branches.iter().any(|b| b.name == config.cb_name) {
        return Vec::new();
    }
    config_tree(ws, &format!("config/{}", config.cb_name)).unwrap_or_default()
}

/// The lineage row: the branches the open gesture found, the escape that names
/// a new one, and the advance/orphan origin. Choosing a lineage re-reads its
/// tree — one git call on the gesture that changed the answer, never per frame.
pub(super) fn lineage(ui: &mut egui::Ui, config: &mut ConfigState, ws: &Path) {
    let before = config.cb_name.clone();
    let known = config.branches.iter().any(|b| b.name == config.cb_name);
    // §11 rule 8, not rule 1b (bl-7414): every member here is a control of its
    // own — a dropdown, a revealed field, and the two origin peers — and none
    // may be dropped, so the row grows a LINE when it cannot fit rather than
    // running off the pane. Laid `horizontal` it could not fit at 480x1400 at
    // all, and an over-full row ratchets the seat's `max_rect`, which is how the
    // §9.5 no-reader sentence below it came to elide at 285 pt inside a 224 pt
    // pane and be hard-cut with its ellipsis outside the clip.
    crate::shell::row::peers(ui, |ui| {
        ui.label("lineage:");
        let shown = if known {
            config.cb_name.clone()
        } else {
            NEW_LINEAGE.to_string()
        };
        egui::ComboBox::from_id_salt("config-branch")
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for b in &config.branches {
                    ui.selectable_value(
                        &mut config.cb_name,
                        b.name.clone(),
                        format!("config/{} @ {}", b.name, b.tip_short_oid),
                    )
                    .on_hover_text(LINEAGE_HINT);
                }
                ui.selectable_value(&mut config.cb_name, String::new(), NEW_LINEAGE)
                    .on_hover_text(
                        "Name a config branch this workspace does not have yet, in the \
                         field this choice reveals — the name `/config orphan <name>` \
                         would carry.",
                    );
            })
            .response
            .on_hover_text(LINEAGE_HINT);
        if !known {
            // The remainder of the line it is on, never egui's fixed 280 pt
            // `text_edit_width` — which is what forced this row wider than the
            // pane whatever the pane had. The peers wrap below it (bl-7414).
            ui.add(egui::TextEdit::singleline(&mut config.cb_name).desired_width(f32::INFINITY))
                .on_hover_text(LINEAGE_HINT);
        }
        ui.selectable_value(&mut config.cb_origin, EditOrigin::Advance, "advance")
            .on_hover_text(
                "Commit on top of the branch as it stands, keeping everything already \
                 on it. Typed, it is `/config branch <name> <text…>`.",
            );
        ui.selectable_value(&mut config.cb_origin, EditOrigin::Orphan, "orphan")
            .on_hover_text(
                "Start the branch over from nothing — only the file below survives, \
                 and its previous history is left behind. Typed, it is \
                 `/config orphan <name> <text…>`.",
            );
    });
    if config.cb_name != before {
        config.cb_files = tree(config, ws);
    }
}

/// The file row: the paths the lineage's commit actually holds, and the Load
/// that fills the body from it — an edit is over what is there, never a blank.
pub(super) fn file_row(ui: &mut egui::Ui, config: &mut ConfigState, ws: &Path) {
    // §11 rule 8 (bl-7414): a dropdown, the free-path field it reveals and Load
    // are each a control of their own, and a control that does not fit is not
    // elided — egui simply never lays it out (`Load` measured 26 pt laid, 2 pt
    // shown). So the row wraps to a second line rather than pushing the verb off
    // the pane, and the field takes the remainder of its own line rather than
    // egui's fixed 280 pt `text_edit_width`, which is what forced the row wider
    // than the pane and ratcheted the seat's `max_rect` for every row below it.
    let files = config.cb_files.clone();
    let load_asked = crate::shell::row::peers(ui, |ui| {
        ui.label("file:").on_hover_text(FILE_HINT);
        egui::ComboBox::from_id_salt("config-branch-file")
            .selected_text(config.cb_path.as_str())
            .show_ui(ui, |ui| {
                for path in &files {
                    ui.selectable_value(&mut config.cb_path, path.clone(), path)
                        .on_hover_text(FILE_HINT);
                }
            })
            .response
            .on_hover_text(FILE_HINT);
        if files.is_empty() {
            ui.add(egui::TextEdit::singleline(&mut config.cb_path).desired_width(f32::INFINITY))
                .on_hover_text(FILE_HINT);
        }
        ui.button("Load")
            .on_hover_text(
                "Read this file out of the selected lineage's tip into the editor \
                 below, so the edit starts from what is actually there. \
                 `/config branch <name> <path>` with no text reads the same \
                 bytes, and `/lineages` lists what there is to read.",
            )
            .clicked()
    });
    if load_asked {
        let said = load(config, ws);
        config.cb_act.say(said);
    }
}

/// Read the selected file out of the selected lineage's tip into the body.
fn load(config: &mut ConfigState, ws: &Path) -> String {
    let refspec = format!("config/{}", config.cb_name);
    match config_file(ws, &refspec, &config.cb_path) {
        Ok(bytes) => {
            config.cb_body = String::from_utf8_lossy(&bytes).into_owned();
            format!("loaded {refspec}:{}", config.cb_path)
        }
        Err(e) => format!("load: {e}"),
    }
}
