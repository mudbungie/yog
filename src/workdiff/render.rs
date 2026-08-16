//! egui widget: the §11 Altitude-2 Work tab — what this workspace's attempts
//! have actually changed in their project.
//!
//! A near-pure function of the [`Attempt`] list and the caller-built patch
//! [`Preview`]. The one interaction is intrinsic (click a file row to read its
//! patch), so — like the Files tab's own selection — it lives here and the
//! selection itself is caller-owned RAM ephemera (§5.3).
//!
//! **Every arm says what it knows and no more.** An unreadable project, a ref
//! that is not there, and a resolved pair with nothing between them are three
//! different sentences; none of them is a blank list.

use super::{Attempt, Change, Churn, FileChurn, WorkFile};
use crate::files_view::{PREVIEW_CAP, Preview};

/// Render the Work tab. `sel` is the caller-owned selected file; clicking a
/// row sets it, and the caller rebuilds `patch` from it on the next frame.
pub fn render(
    ui: &mut egui::Ui,
    attempts: &[Attempt],
    patch: Option<&Preview>,
    sel: &mut Option<WorkFile>,
) {
    if attempts.is_empty() {
        ui.weak(
            "this workspace holds no ball, so it owes no work anywhere — nothing to compare. \
             Claim one and its changes show up here.",
        );
        return;
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        for attempt in attempts {
            render_attempt(ui, attempt, sel);
            ui.separator();
        }
        render_patch(ui, patch);
    });
}

/// One attempt: what it is, then what git says about it. A fan candidate
/// (bl-c2bd) additionally wears its handle, and — when the target's history
/// records its delivery — the derived acceptance mark, which is the only kind
/// of "winner" there is (VISION V3.2).
fn render_attempt(ui: &mut egui::Ui, attempt: &Attempt, sel: &mut Option<WorkFile>) {
    ui.horizontal(|ui| {
        ui.strong(&attempt.ball_id);
        if let Some(handle) = &attempt.handle {
            ui.monospace(handle)
                .on_hover_text("A fan candidate: one isolated attempt on this ball's target.");
        }
        ui.weak(&attempt.project)
            .on_hover_text("The project this ball's work is delivered into.");
        if let Some(delivered) = &attempt.delivered {
            ui.strong(format!("delivered {}", short(delivered)))
                .on_hover_text(
                    "The target's own history records this attempt's delivery — derived from \
                     the tagged squash, stored nowhere.",
                );
        }
    });
    match &attempt.change {
        Change::Unreadable => {
            ui.weak(
                "this project's repository cannot be read here — it is gone, it is not a git \
                 repository, or its checkout is on no branch. Nothing is being compared.",
            );
        }
        Change::Absent {
            target,
            source,
            missing,
        } => {
            ui.weak(format!("comparing {target}..{source}"));
            ui.weak(format!(
                "but this repository has no {} — no work has been committed there yet, or the \
                 branch was removed.",
                missing.join(" and no ")
            ));
        }
        Change::Diff {
            target,
            source,
            target_oid,
            source_oid,
            files,
            truncated,
        } => {
            ui.horizontal(|ui| {
                ui.monospace(format!("{target}..{source}"))
                    .on_hover_text(format!(
                        "Everything on this ball's branch that is not yet on its delivery \
                         target. Run it yourself, in {}: git diff {target}..{source}",
                        attempt.project
                    ));
                ui.weak(format!("{} … {}", short(target_oid), short(source_oid)));
            });
            render_files(ui, attempt, files, *truncated, sel);
        }
    }
}

/// The changed-file rows, or the plain fact that there are none. The pick
/// carries the whole row identity — ball *and* handle — because a fan's
/// candidates all wear the obligation's ball and only the handle says whose
/// diff a path belongs to (bl-c2bd).
fn render_files(
    ui: &mut egui::Ui,
    attempt: &Attempt,
    files: &[FileChurn],
    truncated: bool,
    sel: &mut Option<WorkFile>,
) {
    if files.is_empty() {
        ui.weak("nothing changed yet — the branch is there, and it carries no edits.");
        return;
    }
    for file in files {
        let picked = sel.as_ref().is_some_and(|s| {
            s.ball == attempt.ball_id && s.handle == attempt.handle && s.path == file.path
        });
        let label = format!("{}  {}", churn_label(&file.churn), file.path);
        if ui
            .selectable_label(picked, label)
            .on_hover_text(
                "Read this file's changes below. No key of its own: Tab reaches it, \
                 Space presses it.",
            )
            .clicked()
        {
            *sel = Some(WorkFile {
                ball: attempt.ball_id.clone(),
                handle: attempt.handle.clone(),
                path: file.path.clone(),
            });
        }
    }
    if truncated {
        ui.weak("… (more files changed than this listing shows)");
    }
}

/// One file's churn in words: added and removed line counts, or the honest
/// "binary" for a file whose change lines cannot express.
fn churn_label(churn: &Churn) -> String {
    match churn {
        Churn::Text { added, removed } => format!("+{added} -{removed}"),
        Churn::Binary => "binary".to_owned(),
    }
}

/// The picked file's patch, bounded exactly as a file preview is. No patch is
/// the invitation — which is also the right thing to say in the one other case
/// that reaches it, a pick the diff no longer carries: pick a file.
fn render_patch(ui: &mut egui::Ui, patch: Option<&Preview>) {
    let Some(patch) = patch else {
        ui.weak("pick a file to read its changes");
        return;
    };
    match patch {
        Preview::Text(text) if text.is_empty() => {
            ui.weak("this file's patch came back empty");
        }
        Preview::Text(text) => {
            ui.monospace(text);
        }
        Preview::Binary { size } => {
            ui.weak(format!("binary change — {size} bytes of patch"));
        }
        Preview::Truncated { text, size } => {
            ui.monospace(text);
            ui.weak(format!(
                "… (patch truncated at {} KiB of {size} bytes)",
                PREVIEW_CAP / 1024
            ));
        }
    }
}

/// A commit at git's own short-oid width — the width every other seat in this
/// inspector already wears (the rail's notches, the Steps commit column).
fn short(oid: &str) -> String {
    oid.get(..crate::rail::SHORT_OID).unwrap_or(oid).to_owned()
}
