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
//!
//! This file is the tree itself — the module list, the re-exports, and
//! [`seat`], the rule every docked panel obeys. The assembly that spends them
//! (which panel sits where, in what order) is [`assembly`], split off at §12's
//! budget.

mod act;
mod acting;
mod activity;
mod alerts;
mod assembly;
mod ball_bar;
mod banner;
mod birth;
mod board;
mod bootstrap;
mod center;
mod chrome;
mod clients;
mod clock;
mod config_edit;
mod config_marks;
mod conv_ball;
mod conv_list;
mod conv_row;
mod convs;
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
mod refusal;
pub(super) mod row;
mod search_pane;
mod seat;
mod settings;
mod slash;
mod start_login;
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

pub use assembly::render;

pub use clock::now_ts;
pub(super) use clock::{entropy_seed, now_unix};

/// Seat a resizable panel's content in a rect of the **panel's** own size, and
/// report that rect to egui instead of the content's (§11 rules 2/4/5, bl-9ad4
/// as completed by bl-0424).
///
/// egui stores the rect its content occupied and re-opens the panel at that, so
/// the content — not the operator — owns the boundary in **both** directions.
/// Under-long content shrinks it: a 200 pt trail holding one short row settles
/// at 40 pt on the very next frame. Over-wide content grows it: a row that lays
/// past the edge writes a wider rect, which the next frame opens at, which the
/// row lays past again — ~15 pt a frame until the rule 5 ceiling pins it, and a
/// splitter drag cannot win against it because the walk resumes the frame after
/// the pointer comes up (the operator's "it slides back out", bl-0424).
///
/// A minimum could only ever answer the first half. So the content is laid in a
/// child `Ui` of the panel's own rect and the parent's cursor is advanced by
/// **that** rect rather than by what the child used: the stored rect is the
/// panel's size, whatever the content did inside it. What overflows is clipped
/// — egui already clips a panel to itself — which is rule 1's answer anyway,
/// and the rows are contained in their own right so nothing reaches it.
///
/// The seam closes with the ratchet, and that is the same fact twice. egui
/// clips a panel's fill to the panel rect but starts the next panel at the
/// **content** rect's edge, so content wider than the panel leaves an interval
/// painted by nobody — a translucent bar of the frame's clear colour flickering
/// as the content width changes frame to frame. With the two rects equal there
/// is no interval to paint.
fn seat<R>(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let rect = ui.available_rect_before_wrap();
    let mut inner = ui.new_child(egui::UiBuilder::new().max_rect(rect));
    let out = contents(&mut inner);
    ui.advance_cursor_after_rect(rect);
    out
}
