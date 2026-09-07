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
            Self::Held => "parked a tool invocation for your answer",
            Self::Refused => "was refused at the provider — sign a provider in on this workspace",
            Self::Truncated => {
                "was cut off at its output cap — nothing was committed; raise the role's \
                 max_output_tokens in providers.yaml, or ask for less in one step"
            }
            Self::Flagged => "was flagged for a look, with a reason",
        }
    }
}
