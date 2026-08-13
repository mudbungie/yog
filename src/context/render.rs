//! egui widget: the conversation's context-fullness line (§11's settings rows,
//! §5.1 #35) — seated with the §3.5 spend figures it is deliberately *not*.
//!
//! A pure function of [`Fullness`], with no click, so it is a headless
//! shape-walk-tested render exactly as [`crate::spend::render`] is; the
//! derivation is tested in [`super`].
//!
//! **Absence renders nothing at all.** There is no "context unknown" row: a
//! model whose window nobody declared paints no line, because a seat that
//! always says something teaches the operator to stop reading it, and a
//! percentage of a fabricated denominator is worse than a silence (§3.5's own
//! rule that an unpriced figure is a floor, not an answer).

use super::Fullness;

/// Render one conversation's fullness. The percent leads because it is the
/// question; the tokens behind it are the evidence, weak beside it.
pub fn render(ui: &mut egui::Ui, full: &Fullness) {
    ui.horizontal(|ui| {
        ui.label(format!("context {}%", full.percent()));
        ui.weak(format!(
            "({} / {} tok · {})",
            full.prompt_tokens, full.window, full.model
        ))
        .on_hover_text(hover(full));
    });
}

/// What the figure is and what it is not — the one place its two caveats are
/// spelled, so the row itself can stay a number.
fn hover(full: &Fullness) -> String {
    format!(
        "The prompt {}'s latest step sent, against the context_window \
         models.yaml declares for it — how full this conversation's context is \
         now, not what it has spent (the budget line above sums every step of \
         the whole descent). A provider that reports its cached prefix \
         separately makes this a floor.",
        full.model
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the seat **shows**, through the one paint walk (bl-36c3). This test
    /// carried its own copy of it, and that copy read `Galley::text()` — the
    /// string that went *in*, so a row egui had truncated to `…` still reported
    /// itself whole and every assertion below was blind to the elision. Both
    /// halves are `paint_probe`'s: the glyph read, and a screen large enough
    /// that what is missing is missing because the row dropped it.
    fn painted(full: &Fullness) -> String {
        crate::paint_probe::paint(|ui| render(ui, full))
    }

    #[test]
    fn renders_the_percent_and_the_evidence_behind_it() {
        let text = painted(&Fullness {
            model: "claude-sonnet-5".to_owned(),
            prompt_tokens: 50_000,
            window: 200_000,
        });
        assert!(text.contains("context 25%"), "got:\n{text}");
        assert!(
            text.contains("(50000 / 200000 tok · claude-sonnet-5)"),
            "got:\n{text}"
        );
    }

    /// The hover names the model and separates the two figures the seat carries
    /// — fullness now versus spend to date.
    #[test]
    fn the_hover_says_which_figure_this_is() {
        let full = Fullness {
            model: "gpt-5.4".to_owned(),
            prompt_tokens: 1,
            window: 400_000,
        };
        let hover = hover(&full);
        assert!(hover.contains("gpt-5.4"), "got: {hover}");
        assert!(hover.contains("context_window"), "got: {hover}");
        assert!(hover.contains("floor"), "got: {hover}");
        assert!(painted(&full).contains("context 0%"));
    }
}
