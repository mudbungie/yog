//! **Why the §8.2 per-conversation gestures are shaped the way they are**
//! (bl-d255): prose only — nothing in this file is read by a program.
//!
//! [`Action`](super::Action) keeps the sentence saying what each gesture IS;
//! the rulings behind it live here, split off the roster at §12's cap on the
//! precedent [`exact`](crate::wire::hello::version::exact) set: no ruling is
//! ever deleted, so the pre-split band is answered by a family and not by
//! trimming prose.
//!
//! **[`Stop`](super::Action::Stop)** — `children` is accepted and ignored
//! (bl-6efc, REMOTE §8.2): the cascade is over the whole subtree either way,
//! so the field is a word a client may still say and never a second behaviour.
//!
//! **[`Interrupt`](super::Action::Interrupt)** (bl-a33d) — the deposit's own
//! driver-start is the trigger, litany's standing law (ARCH §2.9: there is no
//! resume verb, a deposit into a quiescent branch starts a driver). One
//! gesture, because the operator's act is one; **two ops rows**, because the
//! interrupt and the deposit are independently observable mutations and a
//! composite row would hide that a stop fired (§4.2).
//!
//! It gates on nothing the seat has to know: a stop landing on a branch with
//! nothing in flight is declined in band and the deposit still lands — the
//! same gesture at zero work, not a case of its own. What it *does* rest on is
//! litany bl-b98d, which the pin carries; [`interrupt`](crate::boundary::interrupt)
//! records what that settling is and why nothing here may be built on a
//! yog-side guess at litany's step state.
//!
//! **[`Nudge`](super::Action::Nudge)** (§8.2's Nudge row, bl-9bef) — it
//! carries no text, and that absence is the whole gesture: litany derives what
//! is due from the transcript tail (ARCH §6 *warrant*), so a first turn whose
//! model call died re-dispatches **in place**. Never
//! [`Message`](super::Action::Message) with an empty body: a deposit would put
//! a second user turn on the wire saying what the first already said.
//!
//! **[`Retarget`](super::Action::Retarget)** (§9.4, bl-2d19, re-scoped by
//! bl-e654) marks this conversation to be re-forked onto the config lineage's
//! head, which its own executor lands at the next step boundary. It is not how
//! a config edit reaches a running conversation, which
//! needs no gesture at all since control resolves the followed lineage's tip at
//! every step; it is how a conversation changes *which* lineage it follows, and
//! the one way out of a divergence holding it on its fork commit. **No config
//! name on the wire**: yog's picker writes one lineage, and that lineage is the
//! one lawful destination, so naming a branch here would be a knob with one
//! value (§9.3).
//!
//! **[`MarkSeen`](super::Action::MarkSeen)** (VISION §5 V5.2, bl-f6fe) — it
//! records this conversation's present evidence as seen: the very watermarks
//! the window writes by focusing it, from one evidence definition, so the two
//! frontends converge over one disk (I0). The windowed seat keeps its
//! focus-tick entry (focus is a view and gains no spelling); this is the entry
//! a seat with no focus needs.
