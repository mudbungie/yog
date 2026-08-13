//! §11 rule 1b — **the control wins the row** (bl-bc06).
//!
//! Rule 1 (DESIGN §11) makes every label in a bounded panel truncate rather
//! than extend, so no row can ratchet its panel wider. Laid the obvious way —
//! text first, control after — that rule feeds the whole width to the greedy
//! text and hands the control whatever is left, which at the DEFAULT window
//! size is routinely one character: a button rendered as a bare `…` that says
//! neither what it is nor what it does. The Login pane's worst case was the
//! damaging one: `claude-session-direct` is the longest provider name in
//! brazen's table, so the one row where the operator actually needs the verb
//! was the one row whose verb vanished.
//!
//! The fix is an ordering, not a size. A row that pairs greedy text with a
//! trailing control lays the **control first**, at its own natural width, and
//! the text truncates into what is left. That is lawful in both directions:
//! a truncated *label* still names something (its head carries the verb glyph
//! and the id, and its hover carries the whole value — QUALITY G1), while a
//! truncated *control* is unusable at any length. Nothing extends past the
//! panel, so rule 1's own invariant is untouched.

/// §11 rule 1 — **a bounded panel truncates** — stated at the panel it is true
/// of (bl-5410).
///
/// It was written into the side panel's own body and read as if it were a
/// window rule; it is not, and egui's default everywhere else is `Extend`. So
/// the top bar, the centre and the activity accessory laid every horizontal
/// label at its natural width and the panel's clip rect sliced it mid-glyph —
/// with **no ellipsis**, because a galley that was never asked to truncate has
/// nothing to mark. That is QUALITY G1's defect in its silent form: the audit
/// measured `auth none…` laid 68 pt and shown 36, the `…` egui *had* added to
/// say the row was cut clipped off along with the rest.
///
/// One call per panel root rather than one ambient default: a `Ui` inherits its
/// parent's style, so the rule reaches every row inside without any of them
/// restating it, and a seat that must not truncate (a wrapped prose block) can
/// still say so locally. What it deliberately does **not** do is make a strip of
/// controls fit — a truncated control is unusable at any length ([`peers`] and
/// [`control_last`] are that half of the answer, rules 8 and 1b).
pub(super) fn bounded(ui: &mut egui::Ui) {
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
}

/// Lay `text` and a trailing `control` on one line, control first.
///
/// The control is allocated at its natural width against the row's full
/// extent, then `text` is laid left-to-right into the remainder and truncates
/// there. Both closures hand their value back, in painted order — the text's
/// (a widget response the caller seats a menu on) and the control's (typically
/// the button's `clicked()`).
///
/// **The truncation is set here, not inherited.** Rule 1 puts
/// `TextWrapMode::Truncate` at the *side panel's* root, so a row laid in that
/// panel elides for free — but the same rows paint in seats that set no such
/// mode (the Login rows render inline in the conversation's auth-failed banner,
/// in the centre, where the default is `Extend`). Pinning the control right
/// while the text beside it is free to extend does not make the text run off
/// the edge as it did before; it makes it run **through the control**, which is
/// worse than the defect being fixed — a real overlap, caught at 800×500 by
/// bl-9551's `acceptance::overlap` walk. So the helper states the whole rule
/// itself and depends on no ambient state: the control is pinned, and the text
/// beside it truncates wherever the row is seated.
pub(super) fn control_last<T, R>(
    ui: &mut egui::Ui,
    text: impl FnOnce(&mut egui::Ui) -> T,
    control: impl FnOnce(&mut egui::Ui) -> R,
) -> (T, R) {
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let control = control(ui);
            let text = ui
                .with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    text(ui)
                })
                .inner;
            (text, control)
        })
        .inner
    })
    .inner
}

/// Lay a strip of **peer controls** — a tab bar, a verb row — so that every
/// one of them is laid out, wrapping to a second line rather than running off
/// the pane (§11 rule 8, bl-b531).
///
/// The failure a plain `ui.horizontal` has here is not elision, it is
/// **omission**: egui does not truncate a control that does not fit, it simply
/// never lays it out. Measured on the altitude-2 inspector strip in the 202 pt
/// centre a 420x320 window leaves — the documented `min_inner_size` — the row
/// painted `Transcript Steps Inbox Files` and `Config` and `Work` did not
/// exist. A label can lose its tail and still name itself; a control that was
/// never laid out has no seat to hover, no rect to click and no ellipsis to
/// warn you it is gone, which is the QUALITY G1 violation *"rendered
/// off-screen … the full value is reachable"* in its least recoverable form.
///
/// Rule 1b is the same question with a different answer, and the two are not
/// in tension: there the row pairs *greedy text* with a control, so the control
/// is pinned and the text truncates into the remainder; here every member is a
/// control of its own natural width and none may be dropped, so the row grows
/// a line instead. Wrapping costs a second row at the minimum size and nothing
/// at any other.
///
/// The centre's own tab strip (`shell::center::strip`) reached this answer
/// first, for exactly this reason, and stated it inline; this is that rule with
/// one home, so the next strip does not have to rediscover it.
pub(super) fn peers<R>(ui: &mut egui::Ui, controls: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.horizontal_wrapped(controls).inner
}

#[cfg(test)]
mod tests {
    use crate::keymap::InspectorTab;

    /// The width the centre has at yog's documented `min_inner_size` of
    /// 420x320, once the roster column and the frame margins are taken — the
    /// pane the audit's `Q-S1-small.png` caught the strip in.
    const NARROW: f32 = 202.0;

    /// Every peer label the frame actually laid out, fully inside the pane.
    fn laid_out(wrapped: bool) -> Vec<String> {
        let ctx = egui::Context::default();
        let out = ctx.run(crate::paint_probe::screen_sized(NARROW, 400.0), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let strip = |ui: &mut egui::Ui| {
                    for tab in InspectorTab::all() {
                        // The hover is not decoration here: the §11 scanner
                        // (`acceptance::hover`) holds every control in
                        // `src/shell/*` to stating what pressing it does, and a
                        // fixture that ducked it would be an exemption carved
                        // into the rule for the convenience of a test.
                        let _ = ui
                            .selectable_label(false, tab.label())
                            .on_hover_text(tab.label());
                    }
                };
                if wrapped {
                    super::peers(ui, strip);
                } else {
                    ui.horizontal(strip);
                }
            });
        });
        let pane = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(NARROW, 400.0));
        let mut seen = Vec::new();
        for clipped in &out.shapes {
            let mut here = Vec::new();
            crate::paint_probe::collect(&clipped.shape, &mut here);
            for (text, rect) in here {
                if pane.contains_rect(rect.intersect(clipped.clip_rect)) {
                    seen.push(text);
                }
            }
        }
        seen
    }

    /// **A strip of peers loses none of them at the minimum window** (bl-b531,
    /// §11 rule 8). Both directions, because the failure mode is omission
    /// rather than elision and an assertion that only checked the wrapped row
    /// could not tell a fix from a fixture: laid in one line the same six tabs
    /// lose `Config` and `Work` outright — never laid out, so there is no seat
    /// to hover, no rect to click and no ellipsis to warn you they are gone.
    #[test]
    fn every_peer_in_a_strip_is_laid_out_at_the_minimum_window() {
        let want: Vec<&str> = InspectorTab::all().iter().map(|t| t.label()).collect();
        let wrapped = laid_out(true);
        for label in &want {
            assert!(
                wrapped.iter().any(|seen| seen == label),
                "{label} was dropped from the wrapped strip: {wrapped:?}"
            );
        }
        let flat = laid_out(false);
        assert!(
            flat.len() < want.len(),
            "the one-line strip must still drop peers, or this proves nothing: {flat:?}"
        );
    }
}
