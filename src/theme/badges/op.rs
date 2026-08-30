//! The **ops-trail** row's badge (§4.2, §11) — the one badge vocabulary painted
//! over `ops.jsonl` rather than over a conversation.
//!
//! Its own file since bl-1296 added a fifth outcome and [`super`] reached
//! §12's cap. bl-b95e retired that outcome again and the file stays: the seam
//! was never the count. The seam is the surface — everything left in the parent describes
//! an agent, a step or a tool result — a conversation's vocabulary — while this
//! one describes an *attempted action yog ran*, which is the activity
//! accessory's subject and nothing else's.

use super::{ASH, BRAZEN, ICHOR, MOONLIT};

/// Glyph + colour + **the outcome said in words** for an ops-trail row
/// ([`crate::opslog::OpOutcome`]) — the §11 activity accessory's one badge
/// mapping, worn by both its seats: the collapsed chip's live-failure count
/// (`· M failed ⚠`, said outright — the chip has the room) and the per-row
/// marker (a dense repeating row, so it hovers the phrase). The two failure
/// outcomes deliberately share `⚠` (§6: a retired failure keeps its row and its
/// mark, losing only ichor), which is exactly why the words carry the load —
/// the phrase is the outcome's *name*, short enough to read inline in the chip.
///
/// `Detached` (bl-8433) reuses the descent-arrow glyph and brazen hue
/// [`flight_badge`](super::flight_badge) wears for `Flight::Subagents` — "a
/// dispatched child is running", the same fact a handed-off `litany prompt`
/// states about itself — rather than minting a new glyph/hue pair the palette
/// doesn't already key to this meaning. Its phrase matches
/// `opslog::exit::ExitKind::Detached`'s own wording verbatim, so the collapsed
/// badge and the expanded detail never say two different things about the same
/// row.
///
/// **There is no `Notice` badge** (bl-b95e). bl-1296 minted one — a dimmed
/// handoff, for a driver whose sink held only benign lines — because the sink
/// was folded into every `-2` row and a byte in it painted ichor. The fold is
/// now gated on the state the launch produced, so such a driver *is* a handoff
/// and wears the handoff's badge; a vocabulary entry for it would be a fifth
/// word for a fact the fourth already says.
pub fn op_badge(outcome: crate::opslog::OpOutcome) -> (&'static str, egui::Color32, &'static str) {
    use crate::opslog::OpOutcome;
    match outcome {
        // A live wound: nothing has re-run this verb clean since.
        OpOutcome::Failed => ("⚠", ICHOR, "failed"),
        // §6: superseded, so the fact stays and the prominence retires to ash.
        OpOutcome::Retired => (
            "⚠",
            ASH,
            "failed, retired by a later clean run of the same verb",
        ),
        // Ran clean: a bullet, not an alarm — the moonlit text default.
        OpOutcome::Clean => ("·", MOONLIT, "ran clean"),
        // Handed off: launched, no exit to observe — neither clean nor failed.
        OpOutcome::Detached => ("↳", BRAZEN, "detached — handed off, no exit to observe"),
    }
}
