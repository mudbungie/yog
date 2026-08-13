//! The config editors' status sentences (§9): each Apply / Reload outcome
//! rendered as the one line the pane paints. Coverage-excluded glue like the
//! rest of `src/shell/*` — every arm here is a projection of an enum whose
//! transitions are tested in `config_edit`.

use crate::config_edit::brazen::Applied;
use crate::config_edit::lernie_global::Saved;

pub(super) fn status_line(ui: &mut egui::Ui, status: &str) {
    if !status.is_empty() {
        ui.weak(status);
    }
}

pub(super) fn reload_status(result: std::io::Result<()>) -> String {
    match result {
        Ok(()) => "reloaded".to_string(),
        Err(e) => e.to_string(),
    }
}

pub(super) fn describe_applied(applied: Applied) -> String {
    match applied {
        Applied::Ok => "applied".to_string(),
        Applied::Rejected { stderr } => format!("rejected: {stderr}"),
        Applied::Conflict => "conflict — reload to re-diff".to_string(),
        Applied::Io { error } => format!("io error: {error}"),
    }
}

pub(super) fn describe_saved(saved: Saved) -> String {
    match saved {
        Saved::Ok => "saved".to_string(),
        Saved::Rejected { unknown } => format!(
            "rejected: brazen has no such provider row — {}",
            unknown
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Saved::Conflict => "conflict — reload to re-diff".to_string(),
        Saved::Io { error } => format!("io error: {error}"),
    }
}
