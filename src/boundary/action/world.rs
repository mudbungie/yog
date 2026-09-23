//! **Why the acts that make, unmake and rank the world's own nouns are shaped
//! the way they are** (bl-d255): prose only — nothing in this file is read by
//! a program. The split is [`conversation`](super::conversation)'s.
//!
//! **[`Prepare`](super::Action::Prepare)** is the §8.1 start flow's mutating
//! half — seed → ensure-workspace → the ball rung's `bl` steps — and the
//! prompt is the separate, deferred
//! [`Prompt`](super::Action::Prompt), exactly as the GUI defers it to the
//! composer: two real gestures under one §8.1 composite.
//!
//! **[`Prompt`](super::Action::Prompt)** (bl-6920) mints the conversation
//! name, passes it via `--name` and spawns detached. `prepared` is the
//! [`Prepare`](super::Action::Prepare) reply (or a re-composed equal); `goal`
//! the edited text. **`seed` is the firing seat's own §3.3 prediction**
//! (bl-1747): a seat that painted a greyed name fires the seed it painted, and
//! `None` predicted nothing (a deposited line, the §4.3 loop) so the door draws
//! off the stamp. A `Deps` field until acts crossed.
//!
//! **[`Fork`](super::Action::Fork)** is one **attempt** (VISION §5 V2,
//! bl-dc0c): `litany dispatch <role> <ws> <parent> --goal <goal> --from <ref>
//! [--pin …]` — the ordinary fork, with the pinned notch's commit (or a
//! `config/<name>` head) as its ref.
//!
//! **A cohort is N of these, not a variant of its own.** V2's ×N fires that
//! gesture N times with per-attempt overrides; membership is derived from the
//! notch the children hang on and the ref each forked off
//! ([`crate::rail::cohort`]), so nothing there — and nothing on disk — records
//! a fan. That is why the boundary grows one attempt-shaped verb instead of a
//! fan verb: N=1 and N>1 are the same gesture, counted.
//!
//! **[`DeleteWorkspace`](super::Action::DeleteWorkspace)** is the §3.6
//! unmaking, gated exactly as the dialog gates it: refused unless the workspace
//! is yog's own, nothing is live, and `typed` re-states its name — fail-closed
//! at fire time, whichever frontend fires.
//!
//! **[`DeleteAgent`](super::Action::DeleteAgent)** is that class one
//! conversation deep (bl-f17a). Gated on liveness there, fail-closed; `typed`
//! re-stating the conversation's name is the one thing that arms `--children`,
//! and an unarmed fire is the bare verb — litany's own `HasDescendants` decline
//! rides back for a subtree nobody confirmed.
//!
//! **[`Pin`](super::Action::Pin)** (§4.1 `pinned`, bl-b986) is the durable
//! operator assertion the §11 tab strip hoists in — a fact every seat has read
//! since bl-296f and nothing has written since bl-7942 took the window's tab
//! strip. An explicit **set**, never a toggle: a toggle races two seats, each
//! reading one rank and flipping it, so the second undoes the first and neither
//! operator asked for what they got. Two ops for one variant, the
//! [`Floor`](super::Action::Floor) shape — a boolean field would make unpinning
//! the absence of pinning rather than an instruction.
