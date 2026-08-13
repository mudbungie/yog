//! The **conversation pane's own column** (§11 altitude 1): its docked
//! accessory stack and the bounded viewport that is everything else.
//!
//! Split from [`super::render`], which owns the *window's* panels, at §12's
//! line budget — and the seam is real rather than a line count. The window
//! divides itself between a top bar, a roster column, one world-level
//! accessory and a remainder; this file divides that remainder between the
//! conversation's own accessories and the conversation. Two containers, the
//! same §11 rule 5 budget applied to each, read once at the top of the
//! container it is a share of ([`crate::layout`]).
//!
//! Coverage-excluded shell glue like the rest of `src/shell/*`; the arithmetic
//! it obeys is [`crate::layout`]'s and is tested there, and the property the
//! whole file exists for — no two painted runs sharing pixels, at any window
//! size — is pinned in `shell/acceptance/overlap.rs`.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::ui_state::Panel;

use super::ShellState;

/// **The goal box's own floor**: the target line, a row of typing and the verb
/// row — the height at which the box is whole. It is the composer panel's
/// `default_height` for the same reason (a panel's first frame is its default
/// height, so a smaller one culls the verb row for a frame), and it is what the
/// settings band below it holds back — `crate::layout::ROW` is the floor of *a
/// row*, and a goal box is not one.
const GOAL_FLOOR: f32 = 96.0;

/// The pane, outermost accessory first. `window` is the whole window's extent
/// (the start box's stored size is a §4.1 window fact); `settled` says the
/// pointer is up, so a dragged boundary lands on disk once.
pub(super) fn render(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    clis: (&Cli, &Cli, &Cli),
    window: egui::Vec2,
    settled: bool,
) {
    let (lernie, bl, bz) = clis;
    // **A pending start draft takes the composer's seat** (§11 S0, bl-6ad8): it
    // is a goal box, and so is the composer, so painting both stacked two live
    // boxes with an identity line each and no answer to "which one does Enter
    // fire?". The draft is the more specific target — the operator opened it on
    // purpose, one gesture ago — so it wins the seat, and Cancel/Escape hands it
    // straight back. Nothing is lost in the trade: the composer's text lives in
    // the per-target draft map (bl-a69a), which the start pane never touches.
    //
    // The conversation's own accessories — the composer, the settings rows and
    // the in-flight strip — belong to the **Conversation tab** (§11, bl-1ca2).
    // They are accessories of a conversation, not of the window: with Config or
    // Login focused there is no conversation on screen for them to hang off.
    let conversation = super::center::conversation_open(state);
    let composer_open = conversation
        && state.start.pending.is_none()
        && model.focused_workspace().is_some()
        && !model.focused_is_replay();
    // The extent every accessory below is a share of (§11 rule 5 as amended,
    // bl-9551): read ONCE, before the first of them is created, so the whole
    // stack divides one number and the half it leaves the conversation cannot
    // be spent twice.
    let pane = ui.available_height();
    // **The pane seats exactly one goal box**: the composer, or the §3.4 start
    // draft holding its seat (bl-6ad8, above). One `if`, so the seat is a fact
    // of the stack rather than a coincidence of two adjacent panels.
    let goal = composer_open || state.start.pending.is_some();
    // The conversation-scoped bottom stack, inside the pane it feeds (§11,
    // bl-c038 — operator: the chat line belongs in the conversation box, not
    // across the entire bottom; untouched, the whole stack stays in the pane).
    // Inner panels stack outermost-first, exactly as the window-level ones do,
    // so the code order below is the reverse of the reading order. Top to
    // bottom on screen:
    //
    //     transcript
    //     in-flight strip   — hard against the chat tail (bl-905f, untouched)
    //     goal box          — the outbox rides INSIDE it, above the draft
    //     settings rows     — the pane's bottom edge (bl-2e18 as amended)
    //
    // **The band-order ruling:** the work directory, the budget, the context and
    // the model selection must not sit between the input bar and the chat —
    // those elements belong below the input box, not above it. They are exactly
    // the settings band, so exactly one band moves: bl-2e18's ordering clause —
    // *"between the goal box and the in-flight strip, so what the conversation
    // runs on reads beside where the operator talks to it"* — is superseded,
    // while its settings-seat ruling (every setting to the bottom of the
    // conversation) stands. The strip is not among the four and keeps its seat; the outbox is
    // the ruling's named exception and needs no move at all, being the §11
    // inbox-composer queue riding above the draft *inside* the goal box's own
    // panel rather than a band beside it.
    //
    // Each band asks the budget for its share and paints nothing when the answer
    // is `None` — the goal box included, with no special case: a pane that
    // cannot seat a row of it has nothing to type into either. **One band moved,
    // so exactly one thing about the budget changes.** The settings band is now
    // created FIRST, and claim order is creation order, so left alone it would
    // take the accessory budget before the box the operator types into ever
    // asks: measured at 420x320 with the activity trail open, the composer was
    // left under 30 pt and painted its target line across its own draft — the
    // rows the ruling just demoted squeezing the input they were demoted below.
    // So the settings band's ceiling holds back the goal box's floor
    // ([`GOAL_FLOOR`], `share`'s third argument), and a pane that cannot pay
    // both stops paying the ROWS rather than shrinking the box.
    //
    // Nothing else holds anything back, because nothing else changed rank: the
    // goal box claimed ahead of the strip before this ball and still does, and
    // the strip's own rule — *"the panel itself is conditional, not its
    // content"* — is what a pane too small for all three has always answered
    // with.
    if let Some(cap) =
        crate::layout::share(pane, ui.available_height(), GOAL_FLOOR).filter(|_| conversation)
    {
        super::settings::render(ui, cap, model, state, lernie, bl, bz);
    }
    if let Some(cap) = crate::layout::share(pane, ui.available_height(), 0.0).filter(|_| goal) {
        if let Some(height) = start_box(ui, model, state, (lernie, bl), (cap, window.y)) {
            model.settle_panel_size(Panel::StartGoal, height, window.y, settled);
        } else {
            // The share is also the fold line's ceiling (§11 inbox-composer,
            // bl-929d): past it the queue scrolls tail-anchored instead of the
            // line climbing. Not `resizable`: the panel's height IS its content
            // (queue region at the derived fold-line height + verb chrome), so a
            // dragged boundary would be a second, stored answer (§11 rule 3).
            //
            // A multi-row default: a panel's first frame is its default height
            // (content height only lands the next frame), so without this the
            // composer's verb row is culled for one frame at every appearance.
            // The same number the band below held back for it — one floor.
            egui::TopBottomPanel::bottom("composer")
                .default_height(GOAL_FLOOR)
                .show_inside(ui, |ui| {
                    ui.set_max_height(cap);
                    super::input_bar::composer(ui, model, state, lernie, bl, cap);
                });
        }
    }
    // The in-flight strip stays innermost, hard against the chat tail (bl-905f,
    // untouched by the band-order ruling): it is not one of the four elements
    // named there, and its own ruling — an operator looking down at the chat
    // must see that it is working — is what that seat exists for.
    if conversation && crate::layout::share(pane, ui.available_height(), 0.0).is_some() {
        super::flight_strip::render(ui, model);
    }
    body(ui, model, state, clis);
}

/// The §3.4 start draft in the goal box's seat, when one is pending — `None`
/// when the seat is the composer's. Hands back the height the panel settled at,
/// which is the one §4.1 fact this stack stores.
///
/// The start box is the one accessory of this stack the operator sizes, so the
/// budget bounds its drag rather than replacing it: the ceiling is the share,
/// the floor the panel's own — lowered to the ceiling when the budget cannot
/// pay even that, since a range whose floor is above its ceiling is not a range.
fn start_box(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    clis: (&Cli, &Cli),
    bounds: (f32, f32),
) -> Option<f32> {
    let (lernie, bl) = clis;
    let (cap, window_y) = bounds;
    state.start.pending.as_ref()?;
    Some(
        egui::TopBottomPanel::bottom("start-composer")
            .resizable(true)
            .default_height(model.panel_size(Panel::StartGoal, window_y))
            .height_range(Panel::StartGoal.min_size().min(cap)..=cap)
            .show_inside(ui, |ui| {
                super::pin_to_panel(ui);
                super::start_pane::composer(ui, model, state, lernie, bl);
            })
            .response
            .rect
            .height(),
    )
}

/// The remainder: the center's tab strip and whichever tab focus it heads (§11,
/// bl-1ca2) — never a mode painted over the conversation.
///
/// **It scrolls** (§11 rule 6, bl-9551). The budget above guarantees the
/// remainder is at least half the pane; it cannot guarantee the surface FITS in
/// half a pane, and at the documented 420x320 minimum nothing does. A pane
/// whose column is a free flow paints its overflow straight over the
/// accessories below it — six runs on the same pixels, the QUALITY G4 defect —
/// so the column is a bounded viewport instead: what does not fit is reached by
/// scrolling, the one answer that stays true at every window size.
///
/// And it is clipped to exactly the room it was left — §11 rule 1 ("nothing in
/// this panel extends past it") on the vertical axis. A scroll body's own clip
/// rect is its viewport grown by egui's `clip_rect_margin`, a few points of
/// deliberate slack for anti-aliasing; against an accessory docked hard beneath
/// it, those points are the bottom of a banner glyph painted over the
/// composer's first row. The viewport is the pane's, so the clip is too.
fn body(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, clis: (&Cli, &Cli, &Cli)) {
    let (lernie, bl, bz) = clis;
    ui.set_clip_rect(ui.clip_rect().intersect(ui.available_rect_before_wrap()));
    egui::ScrollArea::vertical()
        .id_salt("center-body")
        .show(ui, |ui| {
            super::center::render(ui, model, state, lernie, bl, bz);
        });
}
