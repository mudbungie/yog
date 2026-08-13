//! egui widget: the fork composer at a pinned notch (VISION V2.1/V2.2).
//!
//! *"A pinned notch offers **Fork from here**: a goal composer seeded empty."*
//! The seat is the pin's own, beside the banner that says what is pinned — so
//! the gesture is reachable exactly where V2's burden check puts it (*"the
//! composer is reachable only from a pinned notch"*) and nowhere else. Release
//! the pin and the composer is gone with it.
//!
//! Every control here is a **reading of the workspace**, never a yog list: the
//! fork points are the pinned commit plus the workspace's own config branches,
//! the roles are what that ref's `providers.yaml` declares — each shown with
//! the model it names, so the model is visible at the point of choice and
//! cannot lie — and the skills are the world's own pool. A workspace that
//! declares nothing offers nothing: [`Choices::fireable`] and the caller
//! decline together, which is the no-capability-theater rule made mechanical.
//!
//! Every choice paints as a row of selectable labels rather than a dropdown,
//! because these lists are short and an operator comparing candidates is
//! comparing *policies*: a fan is only worth firing if you can see, without
//! opening anything, that #1 and #2 differ.
//!
//! The widget owns no state and fires nothing: it edits the caller's
//! [`Composer`] and returns whether Fire was pressed. The caller turns the
//! attempts into N [`Fork`](crate::boundary::Action::Fork) actions and crosses
//! the boundary with each — one gesture per candidate, because a cohort is
//! counted, never declared.

use super::composer::Composer;
use super::{Attempt, Choices, ForkPoint};
use crate::model_pick::grammar::RoleModel;
use crate::theme;

/// What the seat is, for an operator meeting it cold.
const HEAD: &str = "Fork from here";
const HEAD_HOVER: &str = "Try this conversation again from the mark you pinned. The fork inherits everything the \
     conversation had read by then and nothing it said afterwards. Typed, the whole \
     composer is one `/fork` line.";
const GOAL_HINT: &str = "what the fork should do instead";
const GOAL_HOVER: &str = "What every candidate is asked to do. It reaches the model exactly as typed — the same \
     verbatim rule the ordinary composer keeps; `/fork --goal <the goal…>` says it on \
     one line.";
const MORE: &str = "+";
const FEWER: &str = "−";
const TIMES_HOVER: &str = "How many candidates to fire. Each gets its own controls; they group themselves on the rail \
     because they were born at the same mark — which is also what firing `/fork` more \
     than once from one mark does.";
const FROM_HOVER: &str = "Where this candidate starts. `here` inherits this conversation's history up to the pinned \
     mark; a config branch starts clean from that policy. It is `/fork --from <ref>`.";
const ROLE_HOVER: &str = "The role decides the model: it is read from the config governing the fork point, which is \
     the same file the run itself resolves against. It is `/fork --role <role>`.";
const SKILL_HOVER: &str = "Pin this skill's instructions into the candidate's context — `/fork --skills a,b` \
     names them on the line.";
const FIRE: &str = "Fire";
const FIRE_HOVER: &str = "Dispatch every candidate above. Each is an ordinary fork; nothing records that they belong \
     together — the rail groups them by where they were born. One candidate is one \
     `/fork` line.";

/// Paint the composer. Returns `true` on the frame Fire is pressed; a composer
/// that is not [`ready`](Composer::ready) cannot be pressed at all.
pub fn render(ui: &mut egui::Ui, composer: &mut Composer, choices: &Choices) -> bool {
    ui.horizontal(|ui| {
        ui.colored_label(theme::BRAZEN, HEAD)
            .on_hover_text(HEAD_HOVER);
        times(ui, composer);
    });
    ui.add(
        egui::TextEdit::multiline(&mut composer.goal)
            .hint_text(GOAL_HINT)
            .desired_rows(2)
            .desired_width(f32::INFINITY),
    )
    .on_hover_text(GOAL_HOVER);
    // Skill toggles are collected rather than applied: the attempt is borrowed
    // for the row that offers them, and the toggle belongs to the composer
    // that holds it.
    let mut edits: Vec<(usize, String)> = Vec::new();
    for (index, attempt) in composer.attempts.iter_mut().enumerate() {
        candidate(ui, index, attempt, choices, &mut edits);
    }
    for (index, skill) in edits {
        composer.toggle_skill(index, &skill);
    }
    ui.add_enabled(composer.ready(), egui::Button::new(FIRE))
        .on_hover_text(FIRE_HOVER)
        .clicked()
}

/// The ×N control: the count, and one step either side of it.
fn times(ui: &mut egui::Ui, composer: &mut Composer) {
    let n = composer.attempts.len();
    if ui.small_button(FEWER).on_hover_text(TIMES_HOVER).clicked() {
        composer.resize(n.saturating_sub(1));
    }
    ui.weak(format!("×{n}")).on_hover_text(TIMES_HOVER);
    if ui.small_button(MORE).on_hover_text(TIMES_HOVER).clicked() {
        composer.resize(n + 1);
    }
}

/// One candidate's controls: fork point, role (with the model it names), and
/// the skills it carries.
fn candidate(
    ui: &mut egui::Ui,
    index: usize,
    attempt: &mut Attempt,
    choices: &Choices,
    edits: &mut Vec<(usize, String)>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.weak(format!("#{}", index + 1));
            for point in &choices.points {
                if ui
                    .selectable_label(attempt.from == point.refspec, &point.label)
                    .on_hover_text(FROM_HOVER)
                    .clicked()
                {
                    attempt.from.clone_from(&point.refspec);
                    // The ref decides the policy, so a role the new ref does
                    // not declare must not survive the move: land on that
                    // ref's first role rather than on a name it would refuse.
                    attempt.role = first_role(point);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            for row in &roles_of(attempt, choices) {
                if ui
                    .selectable_label(attempt.role == row.role, role_line(row))
                    .on_hover_text(ROLE_HOVER)
                    .clicked()
                {
                    attempt.role.clone_from(&row.role);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            for skill in &choices.skills {
                let on = attempt.skills.iter().any(|s| s == skill);
                if ui
                    .selectable_label(on, skill)
                    .on_hover_text(SKILL_HOVER)
                    .clicked()
                {
                    edits.push((index, skill.clone()));
                }
            }
        });
    });
}

/// The roles this candidate's fork point declares. A ref the seat does not
/// offer declares nothing here, which paints no role row — the operator sees
/// that the policy has moved out from under the draft.
fn roles_of(attempt: &Attempt, choices: &Choices) -> Vec<RoleModel> {
    choices
        .point(&attempt.from)
        .map(|p| p.roles)
        .unwrap_or_default()
}

/// `worker — anthropic/claude-sonnet-5`: the role, and the model it *is*.
fn role_line(row: &RoleModel) -> String {
    format!("{} — {}/{}", row.role, row.provider, row.model)
}

/// A fork point's first declared role, or nothing — a ref that declares none
/// leaves the attempt role-less, which [`Composer::ready`] refuses to fire.
fn first_role(point: &ForkPoint) -> String {
    point
        .roles
        .first()
        .map(|r| r.role.clone())
        .unwrap_or_default()
}
