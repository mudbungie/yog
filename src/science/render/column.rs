//! One candidate's column of the §11 fan group card (bl-77bc) — its identity
//! and derived mark, the V3.3 figures, and its own two affordances. Split from
//! [`super`] on the group/column seam at §12's budget.

use super::super::compose::Intent;
use super::super::{Attempt, Outcome};
use super::Seat;

/// One candidate's column: identity and mark, then the V3.3 figures, then its
/// own affordances.
pub(super) fn candidate(ui: &mut egui::Ui, row: &Attempt, seat: &mut Seat) {
    let handle = row.diff.handle.clone().unwrap_or_default();
    ui.horizontal_wrapped(|ui| {
        ui.monospace(&handle);
        outcome(ui, &row.outcome);
    });
    figures(ui, row);
    ui.horizontal_wrapped(|ui| {
        let picked = seat.compare.iter().any(|h| h == &handle);
        if ui
            .selectable_label(picked, "compare")
            .on_hover_text(
                "Pick two candidates to diff their terminal responses below. No key of \
                 its own: Tab reaches it, Space presses it.",
            )
            .clicked()
        {
            super::toggle(&mut seat.compare, &handle, picked);
        }
        if ui
            .button("Deliver")
            .on_hover_text(
                "Accept this candidate: seed `/deliver` in the composer — add your \
                 summary of what landed and press Enter. Delivery is the ordinary \
                 source-to-target delivery; a stale candidate is refused before \
                 anything merges.",
            )
            .clicked()
        {
            seat.intent = Some(Intent::Deliver {
                handle: handle.clone(),
            });
        }
        if ui
            .button("Retire")
            .on_hover_text(
                "Release this candidate's worktree; its branch stays readable until \
                 the project's retention policy expires it. Seeds `/retire` in the \
                 composer — Enter runs it.",
            )
            .clicked()
        {
            seat.intent = Some(Intent::Retire { handle });
        }
    });
}

/// The figures a candidate is judged by (V3.3), each an honest absence when
/// the world holds nothing: steps, wall time and usage off the step records,
/// churn off the diff row, then the terminal response itself.
fn figures(ui: &mut egui::Ui, row: &Attempt) {
    match &row.conversation {
        Some(_) => ui.weak(format!(
            "{} steps · {} wall · {} tokens",
            row.steps,
            wall(row.wall_secs),
            row.usage.total_tokens()
        )),
        None => ui.weak("no conversation bound yet"),
    };
    ui.weak(churn(row)).on_hover_text(
        "This candidate's project churn — the same target..source read the rows \
         below drill into.",
    );
    match &row.response {
        Some(text) => ui.label(clipped(text)),
        None => ui.weak("nothing said yet"),
    };
}

/// The derived acceptance mark and its absences, in words (V3.2: a rendered
/// consequence of the target's history, stored nowhere).
fn outcome(ui: &mut egui::Ui, outcome: &Outcome) {
    match outcome {
        Outcome::Accepted { commit } => {
            ui.strong(format!("delivered {}", short(commit)))
                .on_hover_text(
                    "The target's own history records this candidate's delivery — the \
                     tagged squash, derived at read time.",
                );
        }
        Outcome::Rejected { by: Some(by) } => {
            ui.weak(format!("stale — {by} delivered")).on_hover_text(
                "A sibling's delivery advanced the target. Rework this candidate \
                 (message it to incorporate the new target) and it can deliver again.",
            );
        }
        Outcome::Rejected { by: None } => {
            ui.weak("discarded").on_hover_text(
                "This candidate's branch is gone — retired past the project's \
                 retention rule.",
            );
        }
        Outcome::Reworked => {
            ui.weak("reworked").on_hover_text(
                "The source has incorporated the advanced target, so delivery would \
                 no longer refuse it as stale.",
            );
        }
        Outcome::Pending => {
            ui.weak("pending")
                .on_hover_text("No delivery yet, this candidate's or any sibling's.");
        }
    }
}

/// The candidate's project churn in one line, off the diff row it carries.
fn churn(row: &Attempt) -> String {
    match &row.diff.change {
        crate::workdiff::Change::Diff { files, .. } => {
            let (added, removed) = files.iter().fold((0u64, 0u64), |(a, r), f| match &f.churn {
                crate::workdiff::Churn::Text { added, removed } => (a + added, r + removed),
                crate::workdiff::Churn::Binary => (a, r),
            });
            format!("+{added} −{removed} across {} files", files.len())
        }
        crate::workdiff::Change::Absent { missing, .. } => {
            format!("no {} yet", missing.join(" and no "))
        }
        crate::workdiff::Change::Unreadable => "project unreadable".to_owned(),
    }
}

/// Wall seconds as an operator reads them.
fn wall(secs: u64) -> String {
    match secs {
        s if s >= 3600 => format!("{}h{:02}m", s / 3600, (s % 3600) / 60),
        s if s >= 60 => format!("{}m{:02}s", s / 60, s % 60),
        s => format!("{s}s"),
    }
}

/// A terminal response clipped for the card — the diff below reads it whole.
/// Clipped in code, because a galley's text is the string that went IN and an
/// egui elision would be invisible to every assertion (the paint-walk rule).
fn clipped(text: &str) -> String {
    const CAP: usize = 280;
    match text
        .char_indices()
        .nth(CAP)
        .and_then(|(at, _)| text.get(..at))
    {
        Some(head) => format!("{head}…"),
        None => text.to_owned(),
    }
}

/// A commit at git's own short-oid width — the width the inspector wears.
pub(super) fn short(oid: &str) -> String {
    oid.get(..crate::rail::SHORT_OID).unwrap_or(oid).to_owned()
}
