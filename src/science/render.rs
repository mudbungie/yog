//! egui widget: the §11 **fan group card** (VISION V3, bl-77bc) — the cohort
//! rendered as one group above the Work tab's attempt rows, with the
//! comparison the operator actually judges by (V3.3: terminal response,
//! response diff, usage, wall time, churn) and the affordances that resolve it
//! (V3.1–V3.2: Judge, Synthesize, Deliver, Retire — each a
//! [`compose::Intent`] the caller turns into composer text; nothing fires
//! here). The per-candidate column lives in [`column`], split on that seam at
//! §12's budget.
//!
//! A near-pure function of the science rows. A workspace with no fan renders
//! **nothing at all** — the burden check is structural: never fan and this
//! surface does not exist, exactly as V2's cohort of one wears no header.

use super::compose::Intent;
use super::{Attempt, respdiff};

mod column;

/// The caller-owned card state: the compare picks (§5.3 viewport ephemera —
/// *which data you are looking at*) and the affordance clicked this frame,
/// which the shell takes and spends as composer text.
#[derive(Default)]
pub struct Seat {
    /// Handles picked for the response diff. Two compare; a third pick
    /// replaces the elder. A cohort of exactly two compares by itself.
    pub compare: Vec<String>,
    /// The affordance clicked this frame, if any — set here, taken by the
    /// caller, so the card stays a render function rather than a dispatcher.
    pub intent: Option<Intent>,
}

/// Render the group card over `rows` (every attempt the workspace holds).
/// Members are the fan candidates — the rows wearing a handle; with none, the
/// card paints nothing.
pub fn group(ui: &mut egui::Ui, rows: &[Attempt], seat: &mut Seat) {
    let members: Vec<&Attempt> = rows.iter().filter(|r| r.diff.handle.is_some()).collect();
    if members.is_empty() {
        return;
    }
    egui::Frame::group(ui.style()).show(ui, |ui| {
        header(ui, &members, seat);
        ui.columns(members.len(), |cols| {
            for (col, member) in cols.iter_mut().zip(&members) {
                column::candidate(col, member, seat);
            }
        });
        compared(ui, &members, seat);
    });
    ui.separator();
}

/// The group's one line: the obligation, the shared base said once (nothing
/// when members disagree — each column then says its own), and the two
/// cohort-wide dispatch affordances.
fn header(ui: &mut egui::Ui, members: &[&Attempt], seat: &mut Seat) {
    ui.horizontal_wrapped(|ui| {
        ui.strong(format!(
            "fan · {} candidates on {}",
            members.len(),
            members
                .first()
                .map(|m| m.diff.ball_id.as_str())
                .unwrap_or_default()
        ))
        .on_hover_text(
            "Isolated attempts on one delivery target. Delivering one advances the \
             ball's own branch; the rest become stale and must rework before they \
             can deliver.",
        );
        if let Some(base) = shared_base(members) {
            ui.weak(format!("base {}", column::short(&base)))
                .on_hover_text(
                    "The commit every candidate forked from — the cohort's shared root.",
                );
        }
        if ui
            .button("Judge")
            .on_hover_text(
                "Compose a judge dispatch: a goal carrying each candidate's exact refs, \
                 seeded into the composer for you to edit and send. The verdict comes \
                 back as an ordinary message. No key of its own: Tab reaches it, Space \
                 presses it.",
            )
            .clicked()
        {
            seat.intent = Some(Intent::Judge);
        }
        if ui
            .button("Synthesize")
            .on_hover_text(
                "Compose a synthesizer dispatch: a goal carrying each candidate's exact \
                 refs, seeded into the composer. A synthesizer that writes project bytes \
                 is itself an ordinary attempt on the same target. No key of its own: \
                 Tab reaches it, Space presses it.",
            )
            .clicked()
        {
            seat.intent = Some(Intent::Synthesize);
        }
    });
}

/// The response diff (V3.3): the two picked candidates' terminal responses,
/// line against line. Exactly two members compare unpicked — the only pair
/// there is — and any other count waits for picks.
fn compared(ui: &mut egui::Ui, members: &[&Attempt], seat: &mut Seat) {
    let picked: Vec<&&Attempt> = match (seat.compare.len(), members.len()) {
        (2, _) => members
            .iter()
            .filter(|m| {
                m.diff
                    .handle
                    .as_ref()
                    .is_some_and(|h| seat.compare.contains(h))
            })
            .collect(),
        (0, 2) => members.iter().collect(),
        _ => Vec::new(),
    };
    let [left, right] = picked.as_slice() else {
        return;
    };
    let (a, b) = (
        left.response.clone().unwrap_or_default(),
        right.response.clone().unwrap_or_default(),
    );
    let diff = respdiff::lines(&a, &b);
    ui.weak(format!(
        "response diff · − {} · + {}",
        name(left),
        name(right)
    ))
    .on_hover_text(
        "The two picked candidates' terminal responses, compared line by line — \
         what each actually said, not what it changed.",
    );
    for row in &diff.rows {
        match row {
            respdiff::Row::Same(line) => ui.monospace(format!("  {line}")),
            respdiff::Row::Left(line) => ui.monospace(format!("− {line}")),
            respdiff::Row::Right(line) => ui.monospace(format!("+ {line}")),
        };
    }
    if diff.truncated {
        ui.weak("… (responses longer than this comparison reads)");
    }
}

/// A candidate's name for the diff legend — its handle, which every member has
/// (the empty fallback types the unwrap away).
fn name(row: &Attempt) -> String {
    row.diff.handle.clone().unwrap_or_default()
}

/// Toggle one compare pick; a third pick replaces the elder so the pair walks.
pub(super) fn toggle(picks: &mut Vec<String>, handle: &str, picked: bool) {
    if picked {
        picks.retain(|h| h != handle);
        return;
    }
    picks.push(handle.to_owned());
    if picks.len() > 2 {
        picks.remove(0);
    }
}

/// The shared fork point, when every member states the same one — said once at
/// the level that owns it, and not at all when they differ.
fn shared_base(members: &[&Attempt]) -> Option<String> {
    let first = members.first()?.base.clone()?;
    members
        .iter()
        .all(|m| m.base.as_deref() == Some(first.as_str()))
        .then_some(first)
}
