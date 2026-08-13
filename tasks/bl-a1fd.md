+++
title = "the live mark's eye is bronze then sigil for most of a model call: blue means inference, so the eye must wear it for the whole call"
created = 1786599900
updated = 1786599917
claimant = "Latchkey"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator complaint, 2026-08-12, verbatim:

> the icon color doesn't match: we use blue for inference, so the icon center
> should be blue when the model is streaming.

## What is on screen now

The live mark's eye (`src/theme/mark.rs`, `tints`: the first seat is the eye,
the conversation's root) takes its hue from `doing_badge(seat.doing)`, and
`Doing` splits **one open model call into three hues**
(`src/nav/convs/doing.rs:73-82`, verbatim):

    pub fn doing(agent: &Agent) -> Doing {
        if agent.state == AgentState::InFlight {
            return match agent.last_delta {
                None => Doing::Waiting,
                Some(Delta::Thinking) => Doing::Thinking,
                Some(Delta::Text) => Doing::Inference,
            };
        }

and `src/theme/badges.rs::doing_badge` paints those three apart:

    Doing::Waiting   => (BRAZEN, "waiting — the call is open, nothing back yet"),
    Doing::Thinking  => (SIGIL,  "thinking — reasoning, no text yet"),
    Doing::Inference => (SPECTRE, "inference — the answer is streaming"),

So the eye is **bronze** for the whole pre-first-token window and **sigil** for
the whole reasoning window, and only turns spectral blue once a text delta has
landed. That is why the mark does not read as blue while the model is working.

## The ruling this changes, and why it is not a plain bug

The three-way split is deliberate and was itself an operator ruling. From
`src/theme/badges.rs::doing_badge`, verbatim:

    /// **What the spread costs, measured** (operator ruling, 2026-08-02,
    /// amending bl-b768's set). Thinking wore gate violet and moved to sigil:
    /// violet is the *dimmest* hue in the palette (luminance 67 against a void
    /// of 17) and it is the wordmark's own hue, painted two pixels to the
    /// mark's right — so the state the operator most wanted to see was both the
    /// hardest to see and indistinguishable from brand furniture.

and, on why the set is five at all:

    /// **The set is chosen for legibility at 3 px, not for its names.** Every
    /// hue here is driven through `icon::deep` onto a node circle about three
    /// pixels across, where hue angle and brightness are the only channels that
    /// survive — so the five are picked to be maximally separable *against each
    /// other* […]

The operator's ruling of 2026-08-12 supersedes it **for the meaning of blue**:
spectral blue means *a model call is streaming*, and the eye must wear it for
the whole call. `Doing` already has the exact predicate as one word
(`src/nav/convs/doing.rs:54`):

    pub fn is_model_call(self) -> bool {
        matches!(self, Self::Waiting | Self::Thinking | Self::Inference)
    }

whose own doc says *"That union is exactly §5.1 #28's `Inference` class"* — so
the union is already the named fact and the mark is painting a finer
distinction than the class it claims to show.

## What to decide, and where

This is a design edit before it is a code edit. Do not just recolour the match
arm and leave DESIGN §11 and the `doing_badge` doc comment stating the
superseded ruling — that is exactly the drift AGENTS.md forbids ("Nothing is
set in stone. […] don't implement a deviation: fix the doc").

The question the implementer must settle and record:

- **Is the three-way split retired, or is it retired only at the mark?**
  `Doing`'s three arms also feed the flight chip and other seats. The cheap
  answer — every model-call arm returns `SPECTRE` in `doing_badge` — collapses
  the distinction everywhere at once and leaves `Waiting`/`Thinking` as three
  names for one colour, which is a smell: if the hue no longer separates them,
  ask whether the enum should still. If they *are* still worth telling apart,
  the separation must move to a carrier that is not the hue (the mark's hover
  roster already names each seat's doing in words, so the fact is not lost).
- The elegant shape is probably the union, not three arms agreeing:
  `is_model_call()` → SPECTRE, and the remaining arms unchanged. State it as
  one rule, not three coincidences.
- Check `icon::deep` (`src/theme/icon.rs:109-114`) does not distort SPECTRE at
  the eye's size: it pure-saturates the hue, and the eye is larger than the
  3 px node circles the palette was tuned against. If the deepened blue does
  not read as blue, say so with the actual RGB rather than adjusting by feel.

## Acceptance

A paint-layer assertion on the composed frame — the eye's fill is SPECTRE while
the root agent is `InFlight` with `last_delta == None` and with
`Delta::Thinking`, the two cases that are wrong today. Assert the colour, not
the `Doing` value; a test that stops at the enum passes without the mark ever
being painted (bl-70b8's shapes, and the standing lesson that a probe blind to
the render proves nothing).