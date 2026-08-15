//! The top bar (§11 altitude 0): the attention strip on the left, the workspace
//! tab bar on the right. Only what altitude 0 is for — totals and the regime
//! walls; the live mark is a *conversation's* fact and sits on its own headline
//! row (bl-d44e, `super::workspace`). Coverage-excluded glue: the strip total,
//! the jump and the tab-bar build are all tested `AppModel`/`nav` bodies; this
//! file only wires widgets. Its sibling [`super::navigator`] paints the other
//! altitude-0 surface, the conversation-list side panel.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::nav::menu::Seat;
use crate::nav::tabs::{Kind, Tab};
use crate::nav::ws_key;
use crate::theme;

use super::ShellState;
use super::menus::Target;

/// What the tab bar's `Workspaces:` label says on hover (§3.1, §11): the
/// concept in plain words for an operator meeting it for the first time — what
/// a workspace walls off, not how the wall is built.
const WORKSPACES_HINT: &str = "A workspace walls off one sphere of work — personal, an employer, a client. \
     Its conversations, its config, and the balls it claims live inside that wall \
     and never touch another workspace's.";

/// What a workspace tab does when pressed (§11 discoverability invariant).
const TAB_HINT: &str = "Switch to this workspace: the conversation list, the composer and the balls \
     section all re-aim at it. Right-click for its delete / unpin entries. The ↑ / ↓ \
     roster walk crosses workspaces too, landing here on its own.";

/// The ⋯ overflow button (§11): what is folded away behind it, and why.
const OVERFLOW_HINT: &str = "Workspaces that are real but not regimes — checkouts yog did not raise, and \
     read-only replays. Open the menu to focus one, or ★ to pin it as a tab. No key of \
     its own: Tab reaches it, Space opens it.";

/// The §11 top bar: left, the attention strip (totals + jump-to-next); right,
/// the workspace tab bar — named tabs, the slim `new` name form, and the
/// foreign/replay overflow menu.
pub fn render(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, lernie: &Cli) {
    ui.horizontal(|ui| {
        let total = model.strip_total();
        if total > 0 {
            ui.colored_label(theme::BRAZEN, format!("⚑ {total} need attention"));
        } else {
            ui.weak("⚑ nothing stirs");
        }
        // §11 glyph doctrine: `⏭` is not extremely clear on its own, and `next`
        // does not say *next what* — the seat passes only because the strip
        // legend it sits against is co-visible, so the two are painted adjacent
        // (one unit: legend, then the control that walks it) and the job is
        // stated outright on hover. Disabled when the total is zero: an inert
        // control that still accepts a click is the mystery no-op (bl-e266),
        // and the greyed button plus `nothing stirs` say the same thing twice.
        if ui
            .add_enabled(total > 0, egui::Button::new("⏭ next"))
            .on_hover_text(
                "jump to the next conversation needing attention. No key of its own: \
                 Tab reaches it, Space presses it — the ↑ / ↓ walk gets there too.",
            )
            .on_disabled_hover_text("nothing needs attention — nothing to jump to")
            .clicked()
        {
            // The strip's own pointer control lands on a conversation, so it
            // hands the keyboard to the composer like every other pointer
            // selection (§11 focus discipline). The `n`/`i` keyboard plane is
            // untouched: only this button routes here.
            model.jump_next_attention();
            super::focus::request(state);
        }
        // The workspace tab bar, right-aligned under the top right (§11):
        // regime walls, almost invisible. Laid right-to-left, so paint the
        // overflow first, then `new`, then the tabs reversed (leftmost last).
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let bar = model.tab_bar();
            overflow_menu(ui, model, state, &bar);
            if ui
                .button("new")
                .on_hover_text("new workspace — name a fresh sphere wall (w)")
                .clicked()
            {
                super::new_ws::open(state);
            }
            for tab in bar.tabs.iter().rev() {
                workspace_tab(ui, model, state, lernie, tab);
            }
            // The row's own name, painted LAST because this layout lays
            // right-to-left — so it lands immediately left of the leftmost tab
            // (bl-2d87: an unlabelled row of bare names says nothing about what
            // it is). The concept itself is one hover away.
            ui.label("Workspaces: ").on_hover_text(WORKSPACES_HINT);
        });
    });
}

/// One workspace tab: attention-badged name, selected = the focused workspace.
/// Its secondary-click menu is the §11 accelerator seat (`super::menus`) —
/// §3.6's delete on a named workspace, unpin on a pinned hoist — and opening it
/// deliberately does **not** focus the tab.
fn workspace_tab(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    tab: &Tab,
) {
    let badge = if tab.attention > 0 {
        format!("⚑{} ", tab.attention)
    } else {
        String::new()
    };
    // §11 glyph doctrine (bl-9a01): the kind is said in words on the tab itself,
    // from `Kind::mark`'s one home — the bare `▶` suffix it replaces read "play",
    // not "replay", and a hoisted tab's only worded carrier was one click behind
    // the ⋯. Outright words, not hover: the seat is not a dense repeating row
    // (only a *pinned* foreign/replay entry is ever marked, and pinning is an
    // operator's deliberate hoist), and a top-bar tab that has to be hovered to
    // tell a read-only regime from a live one is the same loss in a new place.
    // The mark stays short for exactly the width the tab row cannot spare.
    let label = ui
        .selectable_label(
            tab.selected,
            format!("{badge}{}{}", tab.name, tab.kind_suffix()),
        )
        .on_hover_text(TAB_HINT);
    if label.clicked() {
        // A pointer picked the workspace; the keyboard's next job is the
        // composer it just re-aimed (§11 focus discipline).
        super::focus::workspace(model, state, &tab.ws);
    }
    super::menus::attach(
        &label,
        Seat::WorkspaceTab {
            named: tab.kind == Kind::Named,
            pinned: tab.pinned,
        },
        &Target::Tab(tab.clone()),
        model,
        state,
        lernie,
    );
}

/// The overflow menu (§11): foreign and replay workspaces — real but not
/// regimes — **every one of them, pinned or not** (bl-7e32: pinning changes
/// where an entry also appears, never where it lives). Selecting one focuses it;
/// ★ is the visible pin/unpin toggle, lit while the entry is hoisted, which is
/// what keeps the tab menu's unpin an accelerator rather than a verb's sole
/// carrier (§11 doctrine). The button carries the still-folded entries'
/// aggregate attention and hides when the menu is empty.
fn overflow_menu(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    bar: &crate::nav::tabs::TabBar,
) {
    if bar.overflow.is_empty() {
        return;
    }
    let attention = bar.overflow_attention();
    let title = if attention > 0 {
        format!("⋯ ⚑{attention}")
    } else {
        "⋯".to_owned()
    };
    ui.menu_button(title, |ui| {
        for entry in &bar.overflow {
            // One row per folded-away entry; focusing is the same gesture a tab
            // is, so it says the same thing.
            ui.horizontal(|ui| {
                let badge = if entry.attention > 0 {
                    format!(" ⚑{}", entry.attention)
                } else {
                    String::new()
                };
                let title = format!("{}{}{badge}", entry.name, entry.kind_suffix());
                if ui
                    .selectable_label(entry.selected, title)
                    .on_hover_text(TAB_HINT)
                    .clicked()
                {
                    super::focus::workspace(model, state, &entry.ws);
                    ui.close_menu();
                }
                // The ★ toggle is the pin's visible carrier (bl-7e32); it takes
                // no binding of its own, so it names the frame's own traversal.
                let hint = if entry.pinned {
                    "pinned as a tab — click to unpin. No key of its own: Tab reaches \
                     it, Space presses it."
                } else {
                    "pin as a tab. No key of its own: Tab reaches it, Space presses it."
                };
                if ui
                    .selectable_label(entry.pinned, "★")
                    .on_hover_text(hint)
                    .clicked()
                {
                    model.toggle_pin(&ws_key(&entry.ws));
                }
            });
        }
    })
    .response
    .on_hover_text(OVERFLOW_HINT);
}
