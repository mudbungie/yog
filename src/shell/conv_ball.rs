//! Ball-badge painters for the conversation surfaces (§3.5, §11, §15 Z9), split
//! from [`super::navigator`] per §12's 300-line budget. Coverage-excluded glue:
//! the ids, states, and badges come from the tested `nav::convs`/`AppModel`
//! derivations; these only choose the hue (`theme::ball_hue`) and lay the widgets.

use crate::nav::convs::ConvBall;
use crate::nav::convs::group::ConvGroup;
use crate::theme;

/// A conversation row's ball-id badge (§3.5): the id from the goal stamp (source
/// 1), coloured by the derived join status when known, neutral otherwise.
pub(super) fn row_badge(ui: &mut egui::Ui, ball: &ConvBall) {
    let label = format!("◈{}", ball.id);
    match ball.state {
        Some(s) => ui.colored_label(theme::ball_hue(s), label),
        None => ui.weak(label),
    };
}

/// The grouped view's ball header: the ball id in its §3.5 join hue plus its
/// badge, or "unassociated" for the trailing group (§11 grouped view).
pub(super) fn group_header(ui: &mut egui::Ui, group: &ConvGroup) {
    let Some(ball) = &group.ball else {
        ui.weak("◈ unassociated");
        return;
    };
    let label = format!("◈ {}", ball.id);
    // Badge pinned right, id truncating into what is left — the roster's
    // no-overflow rule (bl-9669: an overflowing row ratchets the panel wider).
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(badge) = &ball.badge {
                ui.weak(badge);
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                match ball.state {
                    Some(s) => ui.colored_label(theme::ball_hue(s), label),
                    // An unresolved stamp still names its ball, neutral hue.
                    None => ui.weak(label),
                };
            });
        });
    });
}

/// The conversation-header ball (§3.3): the id, its title, status hue, and badge
/// — the conversation's own start-flow ball, one per conversation (§3.2). `None`
/// leaves the header ball-less.
pub(super) fn header_ball(ui: &mut egui::Ui, ball: &ConvBall) {
    let label = match &ball.title {
        Some(t) => format!("◈ {} · {t}", ball.id),
        None => format!("◈ {}", ball.id),
    };
    ui.horizontal(|ui| {
        match ball.state {
            Some(s) => ui.colored_label(theme::ball_hue(s), label),
            None => ui.weak(label),
        };
        if let Some(badge) = &ball.badge {
            ui.weak(badge);
        }
    });
}
