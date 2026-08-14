//! The center's **tab strip** (§11 altitude 1) and the one dispatch behind it.
//!
//! The ruling (bl-1ca2): several surfaces — config among them — were interface
//! overlays toggled on rather than tabs, and since they cover everything they
//! should simply be tab focuses. Config, the §8.3 Login
//! surface and the §8.5 search results each used to arrive by covering the
//! conversation — Config swapped the whole `CentralPanel` behind a `bool`,
//! Login rode a collapsing section in the left panel, and the results pane
//! grew out of the composer. They are now four peers in one strip
//! ([`CenterTab`]), of which the center shows exactly one.
//!
//! The general rule the ruling states: **a surface that takes the whole center
//! is a tab — a named peer the operator focuses and leaves by ordinary
//! navigation — never a mode toggled on over everything.** The two modals
//! (§3.1's name form, §3.6's confirmation) are not of this kind and are
//! untouched: a modal owns the frame for one small form.
//!
//! Coverage-excluded glue like the rest of `src/shell/*`: the vocabulary is
//! [`CenterTab`]'s (tested in `keymap`), the search predicate is
//! [`Found::asked`](crate::search::Found::asked)'s, and the surfaces
//! themselves are their own modules'.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::keymap::CenterTab;

use super::ShellState;

/// Focus a center tab — **the one gesture**, spent by the left-panel entries,
/// the strip below, the Command+Shift+digit binding, Escape's way back, and
/// the §9.4 picker's route to the brazen editor.
///
/// Config's re-read hangs here because focusing the tab *is* its freshness
/// (§9, §7.1: the config files carry no watch root, so no frame pays for
/// them). One home, so a second carrier cannot ship a stale pane.
pub(super) fn focus(model: &AppModel, state: &mut ShellState, tab: CenterTab) {
    super::focus::center(state, tab);
    if tab == CenterTab::Config {
        // Lens the brazen-shaped seams on the focused workspace's wall before
        // reading them (§16.2 as amended, bl-c0e2). Idempotent and
        // change-driven, so this costs nothing when the wall has not moved —
        // but the keyboard's Ctrl+Shift+2 is dispatched *before* the frame's
        // own `focus_wall`, and a re-read against the previous sphere's config
        // would be exactly the leak that ruling closed.
        state.focus_wall(model.focused_workspace());
        state.wall.brazen.open();
        state.config.open(model.focused_workspace());
    }
}

/// The strip, then the focused tab's content.
///
/// **The Search tab is offered, not permanent** (§8.5): the results are a view
/// of the published answer, so the tab appears with an answer and goes when a
/// `/search` with no text clears it — there is still no "search mode" to enter
/// or leave. A center left pointing at a tab that has gone falls back home,
/// which is the same rule as "the tab vanished" rather than a case beside it.
pub(super) fn render(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    bz: &Cli,
) {
    let answer = model.found();
    let searching = model.searching();
    // Offered because a search was **asked**, never because it matched
    // (bl-648a): emptiness was standing in for intent, so a needle that hit
    // nothing retired the tab under the operator and dropped them back on
    // Conversation — the frame after a zero-hit search was byte-identical to
    // never having searched. An answer with no needle is still no search, so
    // `/search` with no text clears the tab exactly as before, with no case
    // of its own.
    let offered = searching || answer.asked();
    if state.center == CenterTab::Search && !offered {
        super::focus::center(state, CenterTab::Conversation);
    }
    strip(ui, model, state, offered);
    match state.center {
        CenterTab::Conversation => super::workspace::center(ui, model, state, lernie, bl, bz),
        CenterTab::Config => super::config_edit::center(ui, model, state, lernie, bl),
        CenterTab::Login => {
            let state_root = model.state_root().to_path_buf();
            super::login_pane::login_section(ui, &mut state.wall.login, bz, &state_root);
        }
        CenterTab::Search => super::search_pane::pane(ui, model, state, &answer, searching),
    }
}

/// Whether the conversation's own bottom stack paints this frame — the
/// composer, the settings rows and the in-flight strip all belong to the
/// Conversation tab, since they are accessories *of a conversation* and not of
/// the window. Read by [`super::render`], which owns the panels themselves.
pub(super) fn conversation_open(state: &ShellState) -> bool {
    state.center == CenterTab::Conversation
}

/// One row of tab focuses. `search` says whether the §8.5 tab has an answer to
/// offer; the other three are always peers.
fn strip(ui: &mut egui::Ui, model: &AppModel, state: &mut ShellState, search: bool) {
    let mut pick = None;
    // **Wrapped, not scrolled** (QUALITY G1/G4): at the documented 420 pt
    // minimum the centre is narrower than four tabs laid in a row, and a strip
    // whose last tab is off-window is a peer the operator cannot reach — which
    // would put the reseat right back where it started. A wrapped row costs a
    // second line at that width and nothing at any other.
    super::row::peers(ui, |ui| {
        for tab in CenterTab::all() {
            if tab == CenterTab::Search && !search {
                continue;
            }
            if ui
                .selectable_label(tab == state.center, tab.label())
                .on_hover_text(tab.focus_hover())
                .clicked()
            {
                pick = Some(tab);
            }
        }
    });
    if let Some(tab) = pick {
        focus(model, state, tab);
    }
    ui.separator();
}
