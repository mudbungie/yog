//! **The signal vocabulary** (DESIGN §6): which signals exist, and the sentence
//! each one says.
//!
//! Cut off [`super`] at §12's budget on [`roster`](super::roster)'s own seam —
//! that file answers the questions that only exist across a set, this one
//! answers the question that exists before any derivation runs: what are the
//! words. A word is not a derivation, and only one of the two changes when a
//! seat needs a rule stated rather than badged.

/// One firing signal — the per-badge detail (§6, §3.5 badge rendering).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionKind {
    Notify,
    Stopped,
    Budget,
    Conflicted,
    Mail,
    /// A tool invocation is parked at the capability boundary (§6 rule 6,
    /// §8.6) — the one signal an answer, not an acknowledgement, clears.
    Held,
    /// **Rule 2's rest was a provider refusal** (bl-b43b) — not a seventh rule
    /// and not a seventh signal: the same firing, said in the word that is
    /// true of it. [`Stopped`](Self::Stopped) is what an operator's own `/stop`
    /// earns, and a conversation that never got past its first model call is
    /// not something the operator did. The two are mutually exclusive by
    /// construction ([`Attention::kinds`]).
    Refused,
    /// **Rule 2's rest was a truncation** (bl-ebef, litany bl-155f/bl-ecf9) —
    /// the same refinement [`Refused`](Self::Refused) is, on the other shape of
    /// *the turn did not end*. The provider stopped at the request's output
    /// cap with the model mid-utterance, so litany fails the call with
    /// `Error::OutputTruncated`: the staging sink is never sealed, nothing is
    /// committed and no tool call runs. Saying *"came to rest — your turn"*
    /// about that is the exact sentence the upstream ball measured — nine
    /// `apply_patch` calls arriving as `input: {}` at 4096 output tokens, the
    /// loop staging nothing, and the seat reporting a finished turn. It is
    /// mutually exclusive with [`Refused`](Self::Refused) by construction: a
    /// truncation frames COMPLETE around a `finish` whose reason is `length`,
    /// so it carries no failure sentence, and a refusal carries nothing else.
    Truncated,
    /// Somebody raised a flag on this conversation (§6 rule 7, VISION §4.9) —
    /// the signal-out verb's whole point, and the alignment monitor's floor
    /// grant. The *reason* rides the queue row beside this word, because a
    /// signal that says "look at this" and cannot say why costs a second read
    /// to act on.
    Flagged,
}

/// **The sentence a queue ROW says** — every firing signal's clause, joined,
/// with rule 2's clause refined by *which way* the conversation came to rest
/// (bl-511d).
///
/// Rule 2 fires on rest and never on the wound (ruled bl-2194), and that is
/// unchanged: a clean turn-end and a failed one both put your turn on the
/// queue. What bl-511d found is that they also **said the same sentence**, so
/// a conversation that finished, one that died in its birth step and one
/// parked on an answer only the operator can give were three conditions
/// reading as one row — *"came to rest — your turn"* over a conversation that
/// never got anywhere near rest.
///
/// The refinement is a WORD, not a signal and not a token: it stands where
/// `Stopped`'s clause would, the way [`AttentionKind::Refused`] stands where
/// the whole kind would (bl-b43b), and the row's `signals` list is byte for
/// byte what it was. That is what keeps this off the wire version — the shape
/// did not move, one derived string did — and it costs nothing at a seat,
/// which reads the sentence it is handed.
///
/// **`state` is the fact, not a second flag.** `Stopped` is litany's own
/// reading of a latest step that failed, was killed, or never ran
/// ([`AgentState`]), and it is already on the row beside the sentence; asking
/// it here rather than storing a `wounded` bool is why the two can never
/// disagree. Round-trip is unharmed for the same reason: `says` is derived at
/// the encoder from `signals` and `state`, and a decoded row carries both.
pub(crate) fn row_says(kinds: &[AttentionKind], state: crate::git_tree::AgentState) -> String {
    kinds
        .iter()
        .map(|kind| kind.says_at(state))
        .collect::<Vec<&str>>()
        .join("; ")
}

impl AttentionKind {
    /// The rule in words — why this signal is asking (§6). The **one** home for
    /// that sentence, so the seats that state it rather than badge it cannot
    /// word the same rule two ways. Written as a clause that completes *"this
    /// conversation …"*, since every seat that spends it has already named the
    /// conversation.
    ///
    /// **Its carrier is the queue row's `says`** (bl-09ef,
    /// [`boundary::reply`](crate::boundary::reply)): the announcing is a
    /// seat's — a desktop notification belongs on the box the operator is
    /// looking at — so the sentence crosses the §8.5 boundary beside the
    /// signal tokens rather than being re-worded at each seat. The engine-side
    /// `notify-send` fold that used to spend it went with the frame.
    ///
    /// `pub(crate)` per AGENTS.md rule 2: an internal accessor is demoted
    /// rather than cloned to own — the sentence is a `'static` literal and its
    /// only consumer is the row encoder.
    pub(crate) fn says(self) -> &'static str {
        match self {
            Self::Notify => "raised a notify mark",
            Self::Stopped => "came to rest — your turn",
            Self::Budget => "exhausted its budget",
            Self::Conflicted => "has a conflicted branch",
            Self::Mail => "has mail queued and no driver taking it",
            // **The releasing verb is part of the rule** (bl-511d). Every
            // other clause here names a condition the operator can act on by
            // reading the row; this one names a condition that *nothing but an
            // answer clears* (§6 rule 6), and it used to leave the operator to
            // know which gesture that is. The tool itself is not repeated into
            // the sentence — `held.tool` is already on the row, and one fact
            // does not get two homes.
            Self::Held => "parked a tool invocation for your answer — /answer pass|refuse",
            Self::Refused => "was refused at the provider — sign a provider in on this workspace",
            Self::Truncated => {
                "was cut off at its output cap — nothing was committed; raise the role's \
                 max_output_tokens in providers.yaml, or ask for less in one step"
            }
            Self::Flagged => "was flagged for a look, with a reason",
        }
    }

    /// [`says`](Self::says), with rule 2's clause read against the state the
    /// rest arrived in — see [`row_says`], which is the only caller and carries
    /// the argument. Every other signal's clause is a fact about the signal
    /// alone and passes straight through.
    fn says_at(self, state: crate::git_tree::AgentState) -> &'static str {
        match (self, state) {
            (Self::Stopped, crate::git_tree::AgentState::Stopped) => {
                "stopped without finishing — your turn"
            }
            _ => self.says(),
        }
    }
}
