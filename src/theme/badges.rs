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

use super::{ASH, BRAZEN, HYDRA, ICHOR, MOONLIT, SIGIL, SPECTRE};

/// What the §10 "?" mark says in words — the glyph-doctrine (§11) carrier for
/// probe-degraded state, hovered wherever the sigil is painted. Since bl-8257
/// the conversation row paints it in the trailing group with every other
/// per-row mark, so the words name the row's badge rather than pointing at
/// whatever happens to sit beside them.
pub const STATE_UNCERTAIN: &str = "uncertain — the liveness probe came back unknown, so this row's state badge \
     is inferred from step framing, never observed";

/// What the display-only name hover says in words (bl-8068): the §3.3 ladder
/// landed on the legacy goal-stamp rung, so the title is prose, not the
/// lernie-stored name fact — and lernie resolves message targets by id or
/// stored name only, so peers addressing this name fail with
/// `no agent "<x>" in this workspace`. Shown wherever the name seats render a
/// legacy-rung title ([`crate::git_tree::Agent::name_display_only`]).
pub const NAME_DISPLAY_ONLY: &str = "display-only name — no lernie-stored name fact backs it (dispatched \
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

/// Glyph + colour + **the class said in words** for each §11 live-activity
/// class ([`crate::nav::convs::Flight`]) — what is in flight in a conversation,
/// worn by both indicator seats (the conversation row's pulsing name, the
/// altitude-1 header's chip).
///
/// The three must read **apart at a glance**, so each differs in all three
/// carriers at once, and each hue is the one this palette already keys to that
/// fact — nothing new is minted: `◐` spectral blue is the `InFlight` state's own
/// glyph and hue (a model call streaming); `⚙` hydra green is the tool glyph in
/// the hue of `Live`, which is exactly the state an agent is in while its tool
/// runs; `↳` brazen bronze is the descent arrow in the hue yog already wears for
/// *another agent's* doing (pending mail, attention counts). Each pulses in its
/// own hue through the one shared [`super::pulse`] animation, so they beat
/// together and still say three different things.
pub fn flight_badge(
    flight: crate::nav::convs::Flight,
) -> (&'static str, egui::Color32, &'static str) {
    use crate::nav::convs::Flight;
    match flight {
        Flight::Inference => ("◐", SPECTRE, "inference — a model call is streaming"),
        Flight::Tools => ("⚙", HYDRA, "tools — a tool call is executing"),
        Flight::Subagents => ("↳", BRAZEN, "subagents — a dispatched child is running"),
    }
}

/// Hue + **what the agent is doing said in words** for each §5.1 #28b
/// per-agent state — the one home of the §11 live mark's vocabulary.
///
/// **A pair, not the usual triple, and that is the seat's shape rather than an
/// exemption.** The glyph doctrine forbids a glyph from being a fact's only
/// carrier; on the mark the carrier *is* the circle, and these words are what
/// backs it — the mark hovers a roster naming every seat and saying what it is
/// doing. Minting five glyphs for a surface that paints none would add five
/// tofu risks to say nothing new.
///
/// **The set is chosen for legibility at 3 px, not for its names.** Every hue
/// here is driven through `icon::deep` onto a node circle about three pixels
/// across, where hue angle and brightness are the only channels that survive —
/// so the five are picked to be maximally separable *against each other*, and
/// two of them are borrowed from facts they name elsewhere. Ichor is the wound
/// hue everywhere else; sigil is the §10 uncertainty suffix. Nothing on the
/// mark is ever an error or an uncertainty, so neither reuse can be ambiguous
/// at this seat, and minting two more palette entries to dodge a collision that
/// cannot occur would cost the palette its one-hue-per-fact discipline instead.
///
/// **What the spread costs, measured** (bl-c16f, amending bl-b768's set). Thinking wore gate violet and moved to sigil: violet is the
/// *dimmest* hue in the palette (luminance 67 against a void of 17) and it is
/// the wordmark's own hue, painted two pixels to the mark's right — so the
/// state the operator most wanted to see was both the hardest to see and
/// indistinguishable from brand furniture. Tools then kept ichor rather than
/// taking the freed violet, on the whole-set numbers: min ΔE **65** with red,
/// **49** with violet. Red↔orange sit close in hue but 79 against 175 in
/// luminance, and that 2.2× carries them apart where hue does not;
/// violet↔magenta are close in *both* (67 against 97), so nothing rescues them
/// — and that pair is thinking↔tools, the two an operator most needs apart.
pub fn doing_badge(doing: crate::nav::convs::Doing) -> (egui::Color32, &'static str) {
    use crate::nav::convs::Doing;
    match doing {
        // The mark at rest is hydra green, so idle is the logo, not a state
        // painted over it — and an empty seat reads the same as an idle one.
        Doing::Idle => (HYDRA, "idle — nothing in flight"),
        // The request is out and nothing has come back: brazen, the hue yog
        // already wears for waiting on something else to act.
        Doing::Waiting => (BRAZEN, "waiting — the call is open, nothing back yet"),
        // Reasoning, which displays nothing — the segment that used to look
        // identical to a stalled call. Sigil, whose stated job in the palette
        // is already to never blend into a definite state's hue.
        Doing::Thinking => (SIGIL, "thinking — reasoning, no text yet"),
        // Answering: the same spectral blue the InFlight badge and the live
        // streaming tail already wear for a model call producing text.
        Doing::Inference => (SPECTRE, "inference — the answer is streaming"),
        Doing::Tools => (ICHOR, "tools — a tool call is executing"),
    }
}

/// Short label + colour + **the mark said in a whole sentence** for each
/// `refs/lernie/*` agent mark ([`crate::git_tree::AgentMark`]) — the one home of
/// what a mark is called and what it means.
///
/// These are the §6 **facts** behind five of the six attention rules, and the
/// reason the mapping exists at all: acknowledging a signal clears the signal,
/// never the fact, so the fact needs a carrier that outlives its `⚑`. The match
/// is total over the enum, so a further `refs/lernie/*` namespace cannot reach a
/// renderer without words — which is precisely how notify shipped invisible
/// (bl-efa2: the ref was read into the model and rendered nowhere at all).
pub fn mark_badge(mark: crate::git_tree::AgentMark) -> (&'static str, egui::Color32, &'static str) {
    use crate::git_tree::AgentMark;
    match mark {
        // §6 rule 1's fact. Brazen: it is a summons, not a wound.
        AgentMark::Notified => (
            "notified",
            BRAZEN,
            "notified — this agent asked for you (refs/lernie/notify). The ⚑ cleared \
             when you arrived; the ask stands until the branch raises a new one",
        ),
        // §6 rule 3's fact: the tree hit a spend ceiling — a stop with a cause.
        AgentMark::BudgetExhausted => (
            "budget-exhausted",
            BRAZEN,
            "budget-exhausted — this agent tree hit its spend ceiling \
             (refs/lernie/budget-exhausted)",
        ),
        // §6 rule 4's fact: a transfer was declined loudly (§2.6). Ichor.
        AgentMark::Conflicted => (
            "declined-transfer",
            ICHOR,
            "declined-transfer — a child's work product failed to apply and was \
             declined (refs/lernie/conflicted)",
        ),
        // §6 rule 6's fact: the capability boundary parked an invocation before
        // it ran (§8.6). Brazen, the hue yog already wears for waiting on
        // someone else to act — this is a summons, not a wound, and the drone
        // costs nothing while it waits.
        AgentMark::Held => (
            "held",
            BRAZEN,
            "held — a tool call is parked at the capability boundary and nothing at or \
             past it has run (refs/lernie/held). It waits for your answer and spends \
             nothing meanwhile; approve or decline it to let the branch move on",
        ),
        // Not an attention rule — the assertion that *suppresses* rule 2, so a
        // stopped branch here is quiet on purpose rather than unnoticed.
        AgentMark::Abandoned => (
            "abandoned",
            ASH,
            "abandoned — this stopped branch will not be retried \
             (refs/lernie/abandoned), so it no longer stirs attention",
        ),
    }
}

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
/// [`flight_badge`] wears for `Flight::Subagents` — "a dispatched child is
/// running", the same fact a handed-off `lernie prompt` states about itself —
/// rather than minting a new glyph/hue pair the palette doesn't already key to
/// this meaning. Its phrase matches `opslog::exit::ExitKind::Detached`'s own
/// wording verbatim, so the collapsed badge and the expanded detail never say
/// two different things about the same row.
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
