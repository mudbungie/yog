//! §11 rule 5's arithmetic, in one home: **how much of a container its
//! accessories may take between them.**
//!
//! Rule 5 was written as a per-panel cap — *"no panel may take more than half
//! the window"* — and read that way it is not an invariant at all: the
//! conversation pane docks three accessories (the composer, the settings rows,
//! the in-flight strip) plus a start-goal box, and three halves are 150% of the
//! pane. Measured at the documented 420x320 minimum, the composer and the
//! settings rows took 107 pt of a 138 pt pane, the conversation body was left
//! 26 pt, and every run in it painted on top of every other one — the QUALITY
//! G4 defect bl-9551 was filed for.
//!
//! **The reframe: the cap is a budget over the stack, not a cap on each
//! member.** A container keeps [`KEEP`] of itself for its own content; every
//! accessory it docks draws from the other share, in the order they are
//! created, and each one's ceiling is whatever the budget still holds. Because
//! the ceiling is measured against what is *still unallocated*, the reserve is
//! preserved no matter how many accessories a pane grows — the rule does not
//! have to be re-derived when a fifth one is added, which is exactly what the
//! per-panel spelling could not promise.
//!
//! **Creation order is not priority order** (bl-58e4). egui creates docked
//! panels outermost-first, so the band at the container's bottom edge claims
//! first and the band nearest the container's own content claims last — and a
//! budget spent purely in creation order therefore starves the *innermost* band
//! first. That was harmless while the composer held the bottom edge; the
//! 2026-08-12 ruling moved the settings rows there instead, which would have
//! made the rows the operator demoted outbid the box they were demoted below.
//! So [`share`] takes the count of bands still to be seated inside this one and
//! holds a [`ROW`] back for each. The starved container then sheds from its
//! bottom edge up, and the band nearest its content is the last to go rather
//! than the first.
//!
//! **An accessory the container cannot pay does not paint** ([`share`] answers
//! `None`). egui sizes a panel by its *content*, so a panel handed a ceiling
//! below its content's height does not shrink to it — it lays out at its
//! natural size wherever it was seated, which is the overlap again, one level
//! down. There is therefore no honest "very small accessory": either the budget
//! seats at least [`ROW`] — one line of text, the least a row can be and still
//! say anything — or the accessory is not on screen this frame. This is the
//! in-flight strip's own rule (§11: *"the panel itself is conditional, not its
//! content"*) generalized to every accessory.
//!
//! The same arithmetic bounds the panels that store a size ([`Panel`]'s
//! ceiling, §4.1 `panels`) — one home, so the roster column, the activity trail
//! and the conversation's own accessories cannot disagree about what half means.
//!
//! [`Panel`]: crate::ui_state::Panel

/// The share of itself a container keeps for its own content (§11 rule 5).
///
/// A share rather than a point count because the defect it closes is a ratio:
/// 690 pt of a 1150 pt window is a wide roster, and of an 800 pt one it is an
/// unusable centre. The centre is what the window is *for* — every accessory is
/// a margin around it — so the accessories together may have the other half and
/// no more.
const KEEP: f32 = 0.5;

/// The least an accessory can be and still say anything: one line of text with
/// its frame. Below this there is no small version of a row, only a sliver that
/// paints its content past its own edge, so the answer is to not paint it.
pub(crate) const ROW: f32 = 24.0;

/// The ceiling for the **next** accessory in `container`'s stack, given how
/// much of the container is still unallocated (`available`) — or `None` when
/// the budget can no longer seat a [`ROW`], which is the signal not to paint
/// the accessory at all.
///
/// `container` is the extent the stack is a share of, along the accessory's own
/// axis: the window's height for the window-level accessories, the pane's for
/// the conversation's. It is read once, before the first accessory is created,
/// so every member of one stack divides the same number.
///
/// **`held` is what the accessories still to be seated between this one and the
/// container's own content need between them** — each at its own floor, summed
/// by the stack that knows the order. The reserve exists because claim order
/// and seat order are opposed: egui stacks docked panels outermost-first, so
/// the band at the container's bottom edge claims first. Since the 2026-08-12
/// ruling put the conversation's settings rows there (*"the work directory,
/// budget, context, and model selection … should be below the input box, not
/// above it"*), an unreserved budget would let those rows take the pane out
/// from under the box they were just ruled below — an input squeezed under its
/// own content, or gone, in favour of rows the operator had just demoted.
///
/// **A floor is per band, not [`ROW`] for everyone.** A row of figures really is
/// one line, and the strip's own rule says so; a goal box is a target line, a
/// text box and a verb row, and holding one row back for it buys nothing. So
/// the caller sums real floors and this function only subtracts them.
///
/// It stays one rule over every band, not a carve-out for the goal box: each
/// holds back for whatever is inside it, and a container too small to pay even
/// the reserve pays **nobody** — the goal box included, since a pane that
/// cannot seat a row of it has nothing to type into either.
pub(crate) fn share(container: f32, available: f32, held: f32) -> Option<f32> {
    let left = available - held - container * KEEP;
    (left >= ROW).then_some(left)
}

/// The least a value control can be and still show a token of what it holds.
/// In points, not a share, for the same reason [`Panel::min_size`]'s floor is:
/// a legible field is a physical size, and a share of a narrow pane is not one.
///
/// [`Panel::min_size`]: crate::ui_state::Panel::min_size
const FIELD_MIN: f32 = 120.0;

/// The width a row's value control takes: everything the row has left after
/// what is pinned beside it, never below [`FIELD_MIN`] — **the width-axis twin
/// of [`share`]** (§11 rule 1's "trailing metadata pinned right, the greedy
/// element filling what is left", read for a form row rather than a list row).
///
/// egui's own default is a *constant*: `Style::spacing::text_edit_width`, a
/// fixed 280 pt column whatever the pane. Measured at a maximized 2560 pt
/// window, the §9.5 `capabilities` row (a `models:` field until bl-3ffa retired
/// it) read `tool_use_native, prompt_caching,
/// streaming, stop_` — cut mid-token, no ellipsis, with ~1700 pt of pane unused
/// immediately to its right: QUALITY G1 and G4 in one row, the space that would
/// un-cut it right there and unspent. A constant cannot be right at two window
/// sizes, so the width is derived at both ends — the same reframe rule 5 makes
/// on the other axis.
pub(crate) fn value_width(available: f32, trailing: f32) -> f32 {
    (available - trailing).max(FIELD_MIN)
}

/// The largest a *sized* panel may open at, given the extent of the window
/// along its own axis — the same budget with nothing else yet allocated, which
/// is what a panel the operator drags is measured against (§4.1 `panels`,
/// [`crate::ui_state::Panel::max_size`]).
pub(crate) fn panel_ceiling(window: f32) -> f32 {
    window * KEEP
}

#[cfg(test)]
mod tests {
    use super::{FIELD_MIN, ROW, panel_ceiling, share, value_width};

    /// A value control spends the row it is in, at both ends of the window
    /// range — never egui's fixed 280 pt column, which is wrong at both.
    #[test]
    fn a_value_control_takes_the_room_the_row_has_left() {
        assert!((value_width(2300.0, 100.0) - 2200.0).abs() < f32::EPSILON);
        assert!((value_width(700.0, 100.0) - 600.0).abs() < f32::EPSILON);
        // A row with nothing left still seats a legible field rather than
        // collapsing to a sliver that shows no token at all.
        assert!((value_width(40.0, 100.0) - FIELD_MIN).abs() < f32::EPSILON);
    }

    /// The budget is over the **stack**, not over each member: three
    /// accessories in a row cannot take more than half the pane between them,
    /// which is the property the per-panel spelling of rule 5 could not state.
    #[test]
    fn a_stack_of_accessories_never_eats_the_half_the_pane_keeps() {
        let pane = 600.0;
        let mut available = pane;
        for _ in 0..5 {
            let Some(ceiling) = share(pane, available, 0.0) else {
                break;
            };
            // Each accessory takes everything it is offered — the worst case.
            available -= ceiling;
            assert!(
                available >= pane / 2.0 - 0.001,
                "the pane's own half was spent: {available}"
            );
        }
        assert!(available >= pane / 2.0 - 0.001);
    }

    /// A pane that has already spent its accessory budget offers nothing, and
    /// so does one whose remainder is under a row — the two cases are one
    /// answer, because a sliver of an accessory is not a smaller accessory.
    #[test]
    fn a_budget_that_cannot_seat_a_row_offers_nothing() {
        assert_eq!(share(600.0, 300.0, 0.0), None);
        assert_eq!(share(600.0, 300.0 + ROW - 0.1, 0.0), None);
        assert_eq!(share(600.0, 300.0 + ROW, 0.0), Some(ROW));
        // A pane so small the reserve alone overruns it never pays out.
        assert_eq!(share(20.0, 5.0, 0.0), None);
        // And nobody is paid when the held-back rows are what it cannot afford:
        // the goal box is not exempted from its own container running out.
        assert_eq!(share(20.0, 20.0, 0.0), None);
    }

    /// **A band cannot take the pane out from under the one seated inside it**
    /// (bl-58e4). Drive the conversation pane's stack in the order egui creates
    /// it — the settings rows at the bottom edge, holding back the goal box's
    /// floor, then the goal box itself, taking everything it is offered.
    ///
    /// The claim is conditional on purpose: **whenever the settings rows are
    /// seated at all, the goal box still gets its whole floor.** A pane too
    /// small to pay both does not pay the rows — it does not pay them and
    /// squeeze the box, which is the defect the band reorder would
    /// otherwise have introduced (measured: the composer's target line painted
    /// over its own draft at 420x320 with the activity trail open). The floor is
    /// the conversation pane's own (`shell::pane`): a goal box is a target line,
    /// a text box and a verb row, never [`ROW`].
    #[test]
    fn an_outer_band_cannot_take_the_pane_out_from_under_an_inner_one() {
        const GOAL: f32 = 96.0;
        for pane in [138.0_f32, 250.0, 600.0, 1700.0] {
            let mut left = pane;
            let settings = share(pane, left, GOAL);
            if let Some(ceiling) = settings {
                left -= ceiling;
            }
            let goal = share(pane, left, 0.0);
            assert!(
                settings.is_none() || goal.is_some_and(|cap| cap >= GOAL),
                "the rows were seated at pane {pane} and left the goal box {goal:?}"
            );
        }
    }

    /// The reserve is what the ceiling gives up, exactly — no rounding and no
    /// floor of its own. Without this the assertion above would pass on a
    /// `share` that quietly ignored what it was told to hold back.
    #[test]
    fn what_is_held_back_costs_the_ceiling_exactly_that_much() {
        assert_eq!(share(600.0, 600.0, 0.0), Some(300.0));
        assert_eq!(share(600.0, 600.0, ROW), Some(300.0 - ROW));
        assert_eq!(share(600.0, 600.0, 3.0 * ROW), Some(300.0 - 3.0 * ROW));
        // Held back past what is left, the outer band does not paint at all —
        // the starved pane sheds from its bottom edge up, never from the text
        // down, and the rows the band-order ruling demoted are what it sheds.
        assert_eq!(share(400.0, 400.0, 96.0), Some(200.0 - 96.0));
        assert_eq!(share(240.0, 240.0, 96.0), Some(ROW));
        assert_eq!(share(230.0, 230.0, 96.0), None);
    }

    /// The sized panels divide the same half — one home, so a dragged boundary
    /// and a docked accessory cannot disagree about what the reserve is.
    #[test]
    fn a_sized_panel_opens_at_the_same_half_the_stack_divides() {
        assert!((panel_ceiling(1150.0) - 575.0).abs() < f32::EPSILON);
        assert_eq!(share(1150.0, 1150.0, 0.0), Some(panel_ceiling(1150.0)));
    }
}
