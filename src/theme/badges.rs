//! The §11 badge mappings — glyph (or short label) + hue + **the fact said in
//! words**, together, in one home per fact.
//!
//! The glyph doctrine (DESIGN §11) forbids a glyph from being a fact's only
//! carrier, and the badge-seat pattern says where the words live: *with* the
//! glyph, in the mapping function, never invented at a render site — exactly as
//! no renderer restates an RGB triple. Each mapping here is total over its enum
//! (or its flag), so a new state, outcome or mark **cannot** ship wordless. The
//! seat then chooses how it says them: outright where it has room (a header, a
//! framing badge, a banner), `on_hover_text` on a dense repeating row.
//!
//! Split from [`super`] at §12's cap: the parent is the palette/visuals/font
//! authority, this is the badge vocabulary painted out of it.

use super::{ASH, BRAZEN, HYDRA, ICHOR, MOONLIT, SPECTRE};

/// The §11 **live-activity** vocabulary — what a conversation is doing right
/// now — split to its own file at §12's cap beside [`op`]: everything left here
/// says what an agent, a mark or an outcome *is*, and those two say what is in
/// flight this instant. Re-exported, so a seat still imports one module.
mod flight;
pub use flight::{doing_badge, flight_badge};

/// What the §10 "?" mark says in words — the glyph-doctrine (§11) carrier for
/// probe-degraded state, hovered wherever the sigil is painted. Since bl-8257
/// the conversation row paints it in the trailing group with every other
/// per-row mark, so the words name the row's badge rather than pointing at
/// whatever happens to sit beside them.
pub const STATE_UNCERTAIN: &str = "uncertain — the liveness probe came back unknown, so this row's state badge \
     is inferred from step framing, never observed";

/// What the display-only name hover says in words (bl-8068): the §3.3 ladder
/// landed on the legacy goal-stamp rung, so the title is prose, not the
/// litany-stored name fact — and litany resolves message targets by id or
/// stored name only, so peers addressing this name fail with
/// `no agent "<x>" in this workspace`. Shown wherever the name seats render a
/// legacy-rung title ([`crate::git_tree::Agent::name_display_only`]).
pub const NAME_DISPLAY_ONLY: &str = "display-only name — no litany-stored name fact backs it (dispatched \
     before names were passed at fire), so agents cannot message it by this \
     name; address its agent id instead";
/// What the conversation row's bare `⚑` says in words (§6, bl-b9e3) — the
/// glyph-doctrine carrier for attention at its densest seat. The row is
/// width-bound (bl-9669) and repeats, so the flag hovers this rather than
/// stating it; and it is a **badge, not a tally** — the number is the strip's
/// one global question ("how many are waiting on you"), while the row answers
/// only "is this one waiting".
pub const ROW_ATTENTION: &str = "needs you — evidence arrived on this conversation while you \
     weren't looking. Opening it acknowledges the signal; the fact behind it — the state badge, \
     the mail accessory, the agent's own marks — stays";

/// The §11 **reply elbow** (bl-fa82): the "little chat-reply line" an unfolded
/// child row wears ahead of its prefix, one per depth of descent. `↳` rather
/// than the box-drawing `└` because a connector only reads as a tree when the
/// continuation strokes above it are drawn too, which a flat scrolling list
/// cannot promise — while `↳` is the reply idiom and stands alone. Here because
/// `theme` is every glyph's one spelling; the crate had no elbow idiom before
/// the unfold.
pub const ELBOW: &str = "↳";

/// Glyph + colour + **the state said in words** for each §3.5 agent state — the
/// state-badge mapping every badge seat shares (conversation rows, descent-tree
/// members). The phrase is not decoration: per the §11 glyph doctrine the glyph
/// may not be the state's only carrier, so every seat states it (hover on the
/// dense repeating rows, outright words where the seat has room) and paints the
/// glyph on top for the glance. Pure over the enum, so tests assert the mapping
/// without a windowing context; the palette module is its one home (§11:
/// renderers never restate an RGB triple, nor a state's name).
pub fn state_badge(
    state: crate::git_tree::AgentState,
) -> (&'static str, egui::Color32, &'static str) {
    use crate::git_tree::AgentState;
    match state {
        // A driver holds the lock, not in a model call: solid + hydra green.
        AgentState::Live => (
            "●",
            HYDRA,
            "live — a driver holds this agent, between model calls",
        ),
        // A model call is streaming right now: half-filled + spectral blue.
        AgentState::InFlight => (
            "◐",
            SPECTRE,
            "in flight — a model call is streaming right now",
        ),
        // Finished-for-now, awaiting a message: hollow + tarnished brazen.
        AgentState::Quiescent => (
            "○",
            super::BRAZEN_DIM,
            "quiescent — finished for now, awaiting a message",
        ),
        // No live executor, no clean terminal: square + ash.
        AgentState::Stopped => ("■", ASH, "stopped — no live driver, and no clean end"),
    }
}

/// Glyph + colour + **the verdict said in words** for the alignment monitor's
/// standing verdict (VISION §4.9, rung V6). The three carriers differ at once,
/// like every other badge here, and the hues are ones the palette already keys
/// to the fact: moonlit calm for work that is on task, brazen bronze for the
/// same "somebody else's doing wants your eye" the attention count wears, ichor
/// for the wound. Aligned is deliberately *quiet* — a monitor that shouts when
/// nothing is wrong teaches the operator to stop reading it.
pub fn verdict_badge(
    verdict: crate::monitor::Verdict,
) -> (&'static str, egui::Color32, &'static str) {
    use crate::monitor::Verdict;
    match verdict {
        Verdict::Aligned => ("◇", MOONLIT, "aligned — the recent work serves the goal"),
        Verdict::Drifting => (
            "◈",
            BRAZEN,
            "drifting — the recent work is wandering off the goal",
        ),
        Verdict::Diverged => (
            "◆",
            ICHOR,
            "diverged — the recent work no longer serves the goal",
        ),
    }
}

/// Short label + colour + **the mark said in a whole sentence** for each
/// `refs/litany/*` agent mark ([`crate::git_tree::AgentMark`]) — the one home of
/// what a mark is called and what it means.
///
/// These are the §6 **facts** behind five of the six attention rules, and the
/// reason the mapping exists at all: acknowledging a signal clears the signal,
/// never the fact, so the fact needs a carrier that outlives its `⚑`. The match
/// is total over the enum, so a further `refs/litany/*` namespace cannot reach a
/// renderer without words — which is precisely how notify shipped invisible
/// (bl-efa2: the ref was read into the model and rendered nowhere at all).
pub fn mark_badge(mark: crate::git_tree::AgentMark) -> (&'static str, egui::Color32, &'static str) {
    use crate::git_tree::AgentMark;
    match mark {
        // §6 rule 1's fact. Brazen: it is a summons, not a wound.
        AgentMark::Notified => (
            "notified",
            BRAZEN,
            "notified — this agent asked for you (refs/litany/notify). The ⚑ cleared \
             when you arrived; the ask stands until the branch raises a new one",
        ),
        // §6 rule 3's fact: the tree hit a spend ceiling — a stop with a cause.
        // **And the way out** (bl-d710), because this was the one mark whose
        // sentence named no remedy. The ceiling is not a stored counter and
        // this mark is not a gate: litany re-derives the tree's spend at every
        // model-call boundary and compares it to the `budgets:` of the config
        // commit the branch is FROZEN on (litany ARCH §6, §2.2 — every axis is
        // an `Option`, so a config with no such block bounds nothing and the
        // check that killed the branch passes). §8.6's workflow fixed point
        // strips that block from every workspace yog starts, so the exit is the
        // §9.4 drift clause's own KEEPING exit, in the settings seat below:
        // `retarget` lands at the next step boundary *before anything resolves
        // config*, and the message that wakes the branch is that step. The mark
        // is left standing as the record of what happened — writing one is
        // litany's (§5.1 #14), and so is clearing it.
        AgentMark::BudgetExhausted => (
            "budget-exhausted",
            BRAZEN,
            "budget-exhausted — this agent tree hit the spend ceiling frozen into the \
             config it forked off (refs/litany/budget-exhausted). The ceiling is \
             re-derived at every model call and never stored, so \"move this \
             conversation onto the current config\" in the settings below lifts it \
             wherever that config carries no budget — then message the branch, and \
             the step that lands the move is the step that answers you. The mark \
             stays as the record",
        ),
        // §6 rule 4's fact: a transfer was declined loudly (§2.6). Ichor.
        AgentMark::Conflicted => (
            "declined-transfer",
            ICHOR,
            "declined-transfer — a child's work product failed to apply and was \
             declined (refs/litany/conflicted)",
        ),
        // §6 rule 6's fact: the capability boundary parked an invocation before
        // it ran (§8.6). Brazen, the hue yog already wears for waiting on
        // someone else to act — this is a summons, not a wound, and the drone
        // costs nothing while it waits.
        AgentMark::Held => (
            "held",
            BRAZEN,
            "held — a tool call is parked at the capability boundary and nothing at or \
             past it has run (refs/litany/held). It waits for your answer and spends \
             nothing meanwhile; approve or decline it to let the branch move on",
        ),
        // Not an attention rule — the assertion that *suppresses* rule 2, so a
        // stopped branch here is quiet on purpose rather than unnoticed.
        AgentMark::Abandoned => (
            "abandoned",
            ASH,
            "abandoned — this stopped branch will not be retried \
             (refs/litany/abandoned), so it no longer stirs attention",
        ),
    }
}

/// The **ops-trail** badge, split to its own file at §12's cap (bl-1296): the
/// activity accessory's vocabulary is about an attempted action, not about a
/// conversation, which is what everything else in this module describes.
mod op;
pub use op::op_badge;

/// Glyph + colour + **the outcome said in words** for a tool result — the
/// ok-vs-error mapping both result seats share (the transcript's one-line
/// result row, the Steps drill-in's per-tool header). Per the §11 glyph
/// doctrine `✔`/`✖` may not be the outcome's only carrier, so the phrase rides
/// with the glyph in this one home and no renderer invents its own wording,
/// exactly as none restates an RGB triple. The flag *is* the enum here and the
/// two arms are total over it, so a result cannot ship glyph-only. Pure over
/// the flag, so tests assert the mapping without a windowing context.
pub fn tool_result_badge(is_error: bool) -> (&'static str, egui::Color32, &'static str) {
    if is_error {
        // The tool call came back failed: cross + the wound hue.
        ("✖", ICHOR, "tool result — error")
    } else {
        // It returned normally: check + hydra green.
        ("✔", HYDRA, "tool result — ok")
    }
}
