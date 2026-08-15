//! Interaction glue for the conversation-first shell (DESIGN §11 three
//! altitudes): the top bar (attention strip + workspace tab bar), the
//! conversation-list side panel (navigator), the center's tab strip (`center`)
//! and whichever tab focus it heads — the selected conversation (`workspace`)
//! with the composer docked at its bottom (bl-c038), or Config / Login /
//! Search, which are peers of it and never painted over it (bl-1ca2) — the
//! window-level activity accessory, and the short-verb dispatch (the
//! composer's input bar, the keys and the row menus routing through
//! `dispatch`, §8.2).
//!
//! Pure egui — `Response::clicked()` is unreachable in headless tests, so this
//! tree is coverage-excluded alongside `main.rs` (the established precedent,
//! §12). Everything a click *calls* — attention rollups, the tab bar and
//! conversation list builds, the seen-acknowledgement, pin/collapse mutations,
//! the verb dispatchers + their enablement predicates, the ball fetch/join —
//! lives in tested modules (`AppModel`, `nav`, `attention`, `actions`,
//! `opslog`); this tree only wires widgets.

mod act;
mod acting;
mod activity;
mod alerts;
mod ball_bar;
mod banner;
mod birth;
mod board;
mod bootstrap;
mod center;
mod clients;
mod clock;
mod config_edit;
mod config_marks;
mod conv_ball;
mod conv_list;
mod conv_row;
mod delete;
mod delete_agent;
mod dispatch;
mod fire;
mod flight_strip;
mod focus;
mod inbox_queue;
mod input_bar;
mod inspector;
mod keys;
mod login_pane;
mod menus;
mod modal;
mod model_pick;
mod navigator;
mod new_ws;
mod pane;
mod ram;
pub(super) mod row;
mod search_pane;
mod settings;
mod slash;
mod start_pane;
mod start_rows;
mod top_bar;
mod verb_row;
mod wire;
mod workspace;

#[cfg(test)]
mod acceptance;

pub(crate) use config_edit::BrazenPane;
pub use config_edit::ConfigState;
pub use delete::DeleteState;
pub use delete_agent::DeleteAgentState;
pub use model_pick::PickerState;
pub use new_ws::NewWsState;
pub use ram::{InspectorState, LoginHolder, ShellState, StartState, WallRam};

pub use clock::now_ts;
pub(super) use clock::{entropy_seed, now_unix};

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::ui_state::Panel;

/// Pin a resizable panel's content to the panel's own rect (§11, bl-9ad4).
///
/// egui stores the rect its **content** occupied and re-opens the panel at
/// that, so a pane whose content is shorter than the boundary the operator
/// dragged collapses back to its floor on the very next frame — measured: a
/// 200 pt pane holding one short row settles at 40 pt and stays there. Pinning
/// the content's minimum to the panel's own height makes the stored rect the
/// operator's size rather than the content's. It is the mirror of the width
/// ratchet bl-9669 fixed by truncating rows: over-long content grows the rect,
/// under-long content shrinks it, and a panel the operator sizes must do
/// neither.
fn pin_to_panel(ui: &mut egui::Ui) {
    ui.set_min_height(ui.available_height());
}

/// Render the whole window (§11): the top bar (attention strip + workspace tab
/// bar), the activity accessory (window bottom), the conversation-list side
/// panel, and the center's tab strip with whichever focus it heads (§11,
/// bl-1ca2) — the selected conversation, with the composer docked at its own
/// bottom (bl-c038), being the tab the window rests on. `lernie`/`bl` are the
/// two mutating-verb binaries (§8.2); `bz` drives the §8.3 Login surfaces.
///
/// **Which boundaries drag** (§4.1 `panels`): the conversation column, the
/// expanded activity trail, and the start-goal composer — every panel that
/// holds more than it can show. The top bar and the message composer are one
/// row of chrome each: their height *is* their content, so there is nothing for
/// a drag to reveal and no boundary is offered. The center is the remainder by
/// construction — it grows by whatever the others give up.
pub fn render(
    ctx: &egui::Context,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    bz: &Cli,
) {
    // The frame's own receipt duty (REMOTE §9.8, bl-1747), ahead of every
    // surface that reads what it settles: the model drains the poster's channel
    // in `refresh`, and this is where the window's held acts — the composer's
    // box and the §8.5 line — fold the ones that landed. The §3.6 dialogs fold
    // theirs inside their own frames, a dialog's answer being a dialog's.
    acting::settle(model, state);
    // The wall first (§16.2 as amended): every brazen-shaped seam this frame
    // paints — the config editor's file, the login roster, the picker's
    // providers — belongs to a workspace, so a focus change re-lenses them
    // before anything reads them. A frame whose focus has not moved pays one
    // comparison.
    //
    // **The sphere is the start's, not the focus's** (bl-3b62, §3.4's
    // `start_workspace`). They are the same path whenever anything is focused;
    // where they differ is the empty world, and there the focus answers `None`
    // — which lensed the whole frame onto no wall and left the §8.3 roster
    // empty for exactly the stranger who most needs it. A wall is path algebra
    // over a name (§16.2), so the sphere the next Enter founds has one before
    // it exists, and signing in there signs in the workspace that message will
    // create.
    state.focus_wall(Some(&model.start_workspace()));
    // The §6 escalation (bl-e160): folded where the frame reads its snapshot,
    // never inside a widget.
    alerts::escalate(ctx, model, state);
    keys::handle(ctx, model, state);
    // One gesture is one write (§4.1): a boundary is *settled* when the pointer
    // is up, so a drag lands on disk once, where it came to rest, instead of
    // once per frame of the drag.
    let settled = !ctx.input(|i| i.pointer.any_down());
    // The window's own extent, which every panel is a share of (§11, bl-ac3d):
    // `x` for the side panel, `y` for the bottom ones. Read once per frame, so
    // a resized window re-bounds every boundary on the next frame it paints.
    let window = ctx.screen_rect().size();
    egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
        row::bounded(ui);
        top_bar::render(ui, model, state, lernie);
    });
    // The one window-level bottom accessory (§11, bl-c038): the activity
    // accessory is world-level ops chrome, so it alone spans the window. The
    // conversation-scoped stack — goal box and in-flight strip — docks inside
    // the conversation pane (the CentralPanel below), where the conversation it
    // is about is; the navigator keeps the height it used to lose to them.
    activity_panel(ctx, model, state, window.y, settled);
    // The side panel's floor is a sliver rather than egui's 96 pt default so the
    // roster can be dragged out of the way entirely, and `navigator::side_panel`
    // truncates its rows so a long title cannot ratchet it wider (bl-9669). Its
    // opening width is the operator's last (§4.1 `panels`), and its ceiling is
    // half the window — egui re-clamps the stored rect into `width_range` on
    // every frame, so the one row that escapes truncation costs the centre a
    // bounded share instead of ratcheting the column open forever (bl-ac3d).
    let width = egui::SidePanel::left("conversations")
        .resizable(true)
        .default_width(model.panel_size(Panel::Conversations, window.x))
        .width_range(Panel::Conversations.min_size()..=Panel::Conversations.max_size(window.x))
        .show(ctx, |ui| {
            navigator::side_panel(ui, model, state, lernie);
        })
        .response
        .rect
        .width();
    model.settle_panel_size(Panel::Conversations, width, window.x, settled);
    // The remainder is the conversation pane, which divides itself by the same
    // §11 rule 5 budget one level down (`pane`).
    egui::CentralPanel::default().show(ctx, |ui| {
        row::bounded(ui);
        pane::render(ui, model, state, (lernie, bl, bz), window, settled);
    });
    // The §3.6 confirmation and the §11 `new` name form, painted last so they
    // float over every panel. Both delete carriers open the first; nothing else
    // can, and no key reaches it (§3.6). The backdrop goes first of the three —
    // it must sit under both dialogs and over everything else, which is exactly
    // what "shown before them, after the panels" means to egui's layer order
    // (`modal`, bl-d921).
    modal::backdrop(ctx, state);
    delete::dialog(ctx, model, state);
    delete_agent::dialog(ctx, model, state);
    new_ws::dialog(ctx, model, state);
}

/// The activity accessory's panel (§11) — **two** panels, not one, because a
/// chip and a trail are sized by different things and egui keys panel geometry
/// by id: the collapsed chip's height is its content's, while the expanded
/// trail's is the operator's. Under one id the trail would open at the chip's
/// stored height (a panel's own state outranks its `default_height`, and egui
/// offers no way to seed it back), which is the 48 pt sliver the ops pane used
/// to become. Two ids, one each, and the transition needs no detection at all.
fn activity_panel(
    ctx: &egui::Context,
    model: &mut AppModel,
    state: &mut ShellState,
    window: f32,
    settled: bool,
) {
    let trail = state.activity_open;
    let mut panel = egui::TopBottomPanel::bottom(if trail { "activity-trail" } else { "activity" });
    if trail {
        panel = panel
            .resizable(true)
            .default_height(model.panel_size(Panel::ActivityTrail, window))
            .height_range(Panel::ActivityTrail.min_size()..=Panel::ActivityTrail.max_size(window));
    }
    let height = panel
        .show(ctx, |ui| {
            row::bounded(ui);
            if trail {
                pin_to_panel(ui);
            }
            activity::accessory(ui, model, &mut state.activity_open);
        })
        .response
        .rect
        .height();
    if trail {
        model.settle_panel_size(Panel::ActivityTrail, height, window, settled);
    }
}
