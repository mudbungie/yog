//! The §11 **settings seat** (bl-2e18: every setting for a conversation moves
//! to the bottom of the surface instead of the top).
//!
//! Driven on the real window, because the claim is a *seat*: the config-shaped
//! rows must paint inside the conversation pane's own bottom stack, below the
//! goal box and below everything the transcript leads with — and nowhere else
//! on the surface. Text alone cannot say that, so these read the panel rects
//! and the galley positions of the settled frame.
//!
//! **Below the goal box since bl-58e4** — bl-2e18 seated them above it, and the
//! band-order ruling put them the other side of the input: the work directory,
//! the budget, the context and the model selection must not sit between the
//! input bar and the chat, so they belong below the input box rather than
//! above it. The seat's ordering claim is asserted at every window size in
//! [`super::bands`]; what these beats own is the seat's **contents** — which
//! facts belong in it and which must not be left in the header above. How
//! large the seat may become is [`bound`]'s.

/// **The bound half** — a seat that expands may not eat its pane (QUALITY G4),
/// held at both ends of the supported window range.
mod bound;
/// **The settled frame** these beats measure — the driver, and the galley
/// locators over what it painted.
mod frame;

use super::fixture::world;
use frame::{Window, all, one};

/// The whole ruling, on a selected conversation: the spend figures and the
/// §9.4 model line paint **inside** the conversation pane's settings panel,
/// that panel sits below the goal box, and no copy of either fact is left up in
/// the header the transcript leads with.
#[test]
fn a_conversations_settings_rows_sit_below_the_composer_at_the_pane_foot() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let win = Window::new();
    let painted = win.settled(&mut world);

    let seat = win.panel("conversation-settings");
    let composer = win.panel("composer");
    assert!(
        seat.top() >= composer.bottom() - 1.0,
        "the settings rows sit below the goal box (bl-58e4): seat {seat:?} vs composer {composer:?}"
    );

    // The §3.5 figures — the conversation's own, and one per bound workspace
    // ball — are all in the seat, and none of them is anywhere else.
    let budgets = all(&painted, "budget ");
    assert!(
        !budgets.is_empty(),
        "the spend figures reach the paint layer"
    );
    for figure in &budgets {
        assert!(
            figure.top() >= seat.top() - 1.0,
            "a spend figure paints above the settings seat: {figure:?} vs {seat:?}"
        );
    }

    // The §9.4 model row rides the same seat — and since bl-cd2a the row IS the
    // two dropdowns: the model one shows what the config branch tip assigns,
    // which since bl-a842 is the worker model litany's own template declares.
    let model = one(&painted, "claude-sonnet-5");
    assert!(
        model.top() >= seat.top() - 1.0,
        "the model dropdown sits in the bottom seat: {model:?} vs {seat:?}"
    );
    assert!(
        all(&painted, "change…").is_empty(),
        "the row is the selection: nothing to press before the dropdowns"
    );

    // And the header above is the identity line: the name, the when-seat, and
    // nothing config-shaped.
    let name = one(&painted, "hello");
    assert!(
        name.bottom() <= seat.top(),
        "the identity header leads the surface: {name:?} vs {seat:?}"
    );
}

/// An empty selection is the same seat, not an empty one (§11, bl-824e
/// re-seated): the birth-config block's rows paint in the settings panel too,
/// so "what would a conversation started now run on" is answered exactly where
/// "what is this one running on" is.
#[test]
fn an_empty_selection_answers_the_same_question_in_the_same_seat() {
    let mut world = world();
    let ws = world.ws.clone();
    // `focus_workspace` selects no agent — the very state the block is for.
    world.model.focus_workspace(&crate::naming::leaf(&ws));
    let win = Window::new();
    let painted = win.settled(&mut world);

    let seat = win.panel("conversation-settings");
    // Not the block's "new conversation" heading: the navigator paints those
    // same two words as its own start affordance, and a needle that matches
    // two seats proves neither.
    // The birth block's own two rows: the work directory, and the §9.4 model
    // row — since bl-cd2a the pair itself, whose model half reads what the
    // config head assigns the worker role (bl-a842 gave the fixture one).
    for row in ["work directory:", "claude-sonnet-5"] {
        let rect = one(&painted, row);
        assert!(
            rect.top() >= seat.top() - 1.0,
            "{row:?} belongs to the settings seat: {rect:?} vs {seat:?}"
        );
    }
    // And the half of the seat that has no empty twin (§5.1 #35): a context
    // figure is a *conversation's* fullness, so with none selected there is
    // nothing measured and the row is absent rather than zeroed.
    assert!(
        all(&painted, "context ").is_empty(),
        "no conversation, no context figure"
    );
    let composer = win.panel("composer");
    assert!(
        seat.top() >= composer.bottom() - 1.0,
        "and the empty seat takes the same side of the box as the full one \
         (bl-58e4): {seat:?} vs {composer:?}"
    );
}

/// The context-window percentage per chat, on the paint layer (§5.1 #35).
/// The figure paints **in the settings seat, under the spend line it is not**:
/// this world's one step sent a 50 000-token prompt against a declared 200 000
/// window, so the row reads 25% — where the budget line above it sums a whole
/// descent's burn and answers a different question entirely.
#[test]
fn a_conversation_states_how_full_its_context_is_beneath_what_it_has_spent() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let win = Window::new();
    let painted = win.settled(&mut world);

    let seat = win.panel("conversation-settings");
    let context = one(&painted, "context 25%");
    assert!(
        context.top() >= seat.top() - 1.0,
        "the context figure belongs to the settings seat: {context:?} vs {seat:?}"
    );
    // The evidence rides beside it: the prompt read, the declared window, and
    // the model both were keyed on.
    one(&painted, "(50000 / 200000 tok · m)");
    // And it is a *second* figure, not a rewording of the first: the spend line
    // states the whole descent's tokens, which are not 50 000.
    let spend = one(&painted, "budget ");
    assert!(
        spend.bottom() <= context.bottom(),
        "spend leads, fullness follows: {spend:?} vs {context:?}"
    );
}
