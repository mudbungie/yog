//! The §7.3 **step wound**: what went wrong with one step, said in words where
//! the step is rendered. Two classes, which is the whole of the vocabulary:
//!
//! - **No response** — the driver died before the model produced anything, and
//!   since bl-55d8, why.
//! - **Output limit** — the model call framed cleanly and the *turn* did not
//!   end: the request's `max_tokens` ran out mid-utterance (§4.4
//!   [`Ending::OutputLimit`], bl-fb87). Framing alone paints that step `✔
//!   complete`, which is true of the transport and a lie about the turn — the
//!   same quiet-step misreading the no-response class exists to correct, one
//!   layer up.
//!
//! The no-response class, in detail:
//!
//! Without this state such a step reads as a quiet one — `Framing::Killed`
//! paints the same ash "stopped" badge a mid-stream kill gets, over a
//! `0 attempts · 0 tok` row that looks like nothing happened, while STORIES S0
//! step 4 promises "any step failure is a rendered fact" and §7.3 that a failed
//! action is never stderr-only.
//!
//! The **state** is two observations — the same pair §3.5 already composes for
//! agent state, nothing stored (§5.1 #13):
//!
//! - **Unanswered on disk** — the step's `response.json` carries no bytes
//!   (absent or zero-length) *and* its `meta.json` is absent. lernie writes
//!   `meta.json` only after the model call returns (ARCH §2.3 — its dispatch
//!   loop short-circuits on the call's own error before `write_meta`), so the
//!   pair says exactly: the call emitted nothing and the step never settled.
//! - **Nobody driving** — a live driver's newest step is legitimately
//!   unanswered for the moments between opening `response.json` and the first
//!   streamed event, so the wound is claimed only of an agent whose lock is
//!   free (§3.5). Never a false definite (§10).
//!
//! Only the newest step can be the one a driver is filling, so the liveness
//! observation gates that step alone — an earlier unanswered step is
//! unambiguous, and stays rendered as the place the conversation died.
//!
//! The **reason** is a third read of the same step directory, and it is the
//! whole of bl-55d8. lernie ARCH §2.3 on `stderr.log`, verbatim: *"the adapter
//! subprocess's stderr, appended once per attempt across the model call.
//! **Empty on an ordinary run**: brazen speaks every failure in-band on stdout
//! (§4.4), so bytes here mean the adapter failed outside that contract — a
//! startup failure (a malformed brazen config, an unreadable credstore) that
//! produced no events at all."* That is this wound's class exactly, so the
//! file is not a hint about the cause — it **is** the cause, in the adapter's
//! own words, sitting in the step yog is already reading.
//!
//! **It is not a new state, and it is not a new stored fact** (bl-55d8): the
//! predicate is unchanged, the read is gated on it (a healthy step pays
//! nothing), and the words are re-read from disk every derivation like every
//! other §5.1 fact.
//!
//! **Where it is NOT.** Until bl-55d8 the banner pointed at the ops surface —
//! *"the driver's own stderr is in the activity trail below"*. For the class
//! the operator actually hit that pointer is empty: a turn continued by
//! `lernie message` is driven by a child **lernie** launched, not by a yog
//! detached spawn, so no §8.1 per-spawn sink exists to fold into a `-2` ops row
//! at all. The step's own `stderr.log` is the only copy, which is why the
//! sentence now carries the bytes instead of naming a place to look.
//!
//! Deliberately *not* a reproduction hatch (§8.4 `yog exec`): see §14's
//! rejection — the driver's own words are now rendered where the wound is, so
//! there is even less reason to ask the operator to re-create it.

use std::path::Path;

use crate::git_tree::{AgentState, Ending};

/// The sentence yog renders at the wound — the §7.3 rendered fact for this
/// class. Used verbatim beside the Steps row and composed into the §11
/// Altitude-1 banner, so both surfaces say one thing.
pub const NO_RESPONSE: &str = "driver produced no response";

/// The same, for the §4.4 output-limit class (bl-fb87). It names what happened
/// to the **turn**, not to the transport, because the transport is the half
/// the framing badge already reports and the half that went fine.
pub const OUTPUT_LIMIT: &str = "output limit ended the turn";

/// What the banner adds when the step's own `stderr.log` is empty too — the
/// honest end of the trail, said outright rather than pointing somewhere that
/// has nothing either.
const MUTE: &str = "and its stderr.log is empty too — nothing on disk says why";

/// How the banner introduces the captured bytes. It names the **file**, not a
/// surface, for the §8.3 fallback-grammar reason: an operator who wants more
/// than the tail must be told where the whole of it lives.
const SPOKE: &str = "its stderr.log says:";

/// What the banner adds for the output-limit class: what the operator is
/// looking at, and the one gesture that carries it on.
///
/// It names Nudge in order to **retire** it, because §8.2 offers Nudge on
/// every other resting conversation and a control that silently disappears
/// reads as a bug. Linked lernie derives `NothingDue` from a tool-free
/// assistant tail and exits without creating a step, so the honest sentence is
/// that the gesture cannot help and which one can — never a blind retry, and
/// never a new verb (bl-fb87).
const CUT_OFF: &str = "the reply stops where the model's output budget ran out, so nothing \
     more is coming. Nudge cannot resume it — send a message to carry it on.";

/// The §11 banner's leading mark. Never the only carrier — the sentence beside
/// it states the fact in words (§11 glyph doctrine).
const ALARM: &str = "⚠";

/// The §7.3 wound, its class **and its reason** — four readings of one
/// derivation, never a stored flag (§5.1 #13).
///
/// The two no-response arms are separate rather than an `Option<String>`
/// because a wound with nothing to say is a real, distinct answer (a SIGKILL
/// mid-call leaves an empty `stderr.log`), and `Some("")` would spell it as a
/// wound whose words are blank — one fact with two encodings, which is how
/// they drift.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Wound {
    /// Not a wound: the step answered whole, or it settled, or a driver is
    /// still filling it.
    #[default]
    None,
    /// The driver produced nothing and left no words behind.
    Mute,
    /// The driver produced nothing, and the adapter said why — the tail of the
    /// step's `stderr.log`, verbatim.
    Spoke(String),
    /// The output limit ended the turn (§4.4, bl-fb87). It carries no reason
    /// of its own: the reason IS the class, stated by the canonical finish
    /// reason rather than recovered from anybody's stderr.
    OutputLimit,
}

impl Wound {
    /// Is this step the wound? The §11 Altitude-1 banner's gate and the Steps
    /// row's badge both ask exactly this and nothing finer.
    pub fn wounded(&self) -> bool {
        !matches!(self, Wound::None)
    }

    /// The class, in the badge's one seat — the words the Steps row paints
    /// where the framing badge would otherwise go. `Wound::None` has no word:
    /// the caller gates on [`wounded`](Self::wounded) and paints the framing.
    pub(crate) fn word(&self) -> &'static str {
        match self {
            Wound::None => "",
            Wound::Mute | Wound::Spoke(_) => NO_RESPONSE,
            Wound::OutputLimit => OUTPUT_LIMIT,
        }
    }

    /// The whole §7.3 rendered fact, in words — glyph, the class, and the
    /// reason. One home, so the banner cannot drift from what the derivation
    /// found, and so the sentence is assertable without a frame. `Wound::None`
    /// has no sentence: the caller gates on [`wounded`](Self::wounded).
    pub fn banner(&self) -> String {
        match self {
            Wound::None => String::new(),
            Wound::Mute => format!("{ALARM} {NO_RESPONSE} — {MUTE}"),
            Wound::Spoke(words) => format!("{ALARM} {NO_RESPONSE} — {SPOKE} {words}"),
            Wound::OutputLimit => format!("{ALARM} {OUTPUT_LIMIT} — {CUT_OFF}"),
        }
    }
}

/// Read one step's wound from its own bytes. `response`, `meta_present` and
/// `ending` are the reads [`summarize`](super::summarize) already made —
/// `meta_present` is the *existence* of `meta.json`, not its parse, since a
/// malformed meta still means the step settled, and `ending` is the §4.4
/// classifier's semantic half off the same response bytes (never a second
/// walk, §15 Y13).
///
/// The two classes are disjoint by construction: a step with no response bytes
/// has no settled tail to read an [`Ending`] out of.
///
/// The `stderr.log` read is **gated on the predicate**: an ordinary step pays
/// one comparison and no syscall, so attaching the reason costs a healthy
/// conversation nothing. Both bounds on how much is read are borrowed, not
/// invented — [`crate::opslog::detached::captured`] for how much of a capture
/// file yog ever reads, [`crate::opslog::rows::stderr_tail`] for how much of a
/// stderr a *surface* says.
pub(super) fn read(step: &Path, response: &[u8], meta_present: bool, ending: Ending) -> Wound {
    if !response.is_empty() || meta_present {
        return if ending == Ending::OutputLimit {
            Wound::OutputLimit
        } else {
            Wound::None
        };
    }
    let captured = crate::opslog::detached::captured(&step.join(super::records::STDERR_FILE));
    let words = crate::opslog::rows::stderr_tail(captured.trim());
    if words.is_empty() {
        Wound::Mute
    } else {
        Wound::Spoke(words)
    }
}

/// Is a driver at work on this agent (§3.5)? Then its newest step is allowed
/// to be unanswered — a model call in flight, not a wound.
pub(super) fn driven(state: AgentState) -> bool {
    matches!(state, AgentState::Live | AgentState::InFlight)
}

/// The agent's **latest** step's wound — the §11 Altitude-1 banner's input,
/// read off an already-built [`super::StepsView`] (the one owner of the
/// per-step reading) exactly as the Login banner reads its own. It takes the
/// view, not the disk: the shell declares one standing `Query::Steps` that this
/// banner and the Steps tab share (REMOTE §9.7, bl-13f9), and a predicate that
/// re-read the whole steps tree per frame was the chat pane's frame-time cost.
pub fn latest_wound(steps: &super::StepsView) -> Wound {
    steps.steps.last().map_or(Wound::None, |s| s.wound.clone())
}
