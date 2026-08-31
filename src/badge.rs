//! **The badge vocabulary** — a glyph and *the fact said in words*, together,
//! in one home per fact.
//!
//! What is left here after the severance (bl-7942) is the half a **server**
//! owns. The rest of this module was the congeries palette: hues, egui
//! visuals, fonts, the application mark — every one of them a statement about
//! how a face paints, and every one of them now the seat's
//! (REMOTE §12). A badge is not: the *words* are a derived row's own content,
//! they cross the wire inside the row, and a seat that invented its own
//! wording would be a second spelling of a fact yog states.
//!
//! Two mappings survive, and each is total over its subject so a new outcome
//! cannot ship wordless: [`op_badge`] over the ops-trail's outcomes, and
//! [`tool_result_badge`] over a tool result's one flag. Both return
//! `(glyph, words)`; the hue slot they used to carry was the palette's, and it
//! went with the palette — every caller already discarded it.

/// Glyph + **the outcome said in words** for an ops-trail row
/// ([`crate::opslog::OpOutcome`]), worn by the activity chip's live-failure
/// count (`· M failed ⚠`).
///
/// The two failure outcomes deliberately share `⚠` (§6: a retired failure
/// keeps its row and its mark, losing only prominence), which is exactly why
/// the words carry the load — the phrase is the outcome's *name*.
/// `Detached`'s phrase matches `opslog::exit::ExitKind::Detached`'s own
/// wording verbatim, so the collapsed badge and the expanded detail never say
/// two different things about one row.
///
/// **There is no `Notice` badge** (bl-b95e). bl-1296 minted one — for a driver
/// whose sink held only benign lines — because the sink was folded into every
/// `-2` row. The fold is now gated on the state the launch produced, so such a
/// driver *is* a handoff and wears the handoff's badge.
pub fn op_badge(outcome: crate::opslog::OpOutcome) -> (&'static str, &'static str) {
    use crate::opslog::OpOutcome;
    match outcome {
        // A live wound: nothing has re-run this verb clean since.
        OpOutcome::Failed => ("⚠", "failed"),
        // §6: superseded, so the fact stays and the prominence retires.
        OpOutcome::Retired => ("⚠", "failed, retired by a later clean run of the same verb"),
        // Ran clean: a bullet, not an alarm.
        OpOutcome::Clean => ("·", "ran clean"),
        // Handed off: launched, no exit to observe — neither clean nor failed.
        OpOutcome::Detached => ("↳", "detached — handed off, no exit to observe"),
    }
}

/// Glyph + **the outcome said in words** for a tool result — the one ok-vs-error
/// mapping, read by the transcript row projection so no seat invents its own
/// wording. The flag *is* the enum here and the two arms are total over it, so
/// a result cannot ship glyph-only.
pub fn tool_result_badge(is_error: bool) -> (&'static str, &'static str) {
    if is_error {
        // The tool call came back failed.
        ("✖", "tool result — error")
    } else {
        // It returned normally.
        ("✔", "tool result — ok")
    }
}

#[cfg(test)]
mod tests;
