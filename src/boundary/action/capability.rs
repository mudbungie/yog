//! **Why the §8.6 capability family's two acts are shaped the way they are**
//! (bl-d255): prose only — nothing in this file is read by a program. The
//! split is [`conversation`](super::conversation)'s.
//!
//! **[`AnswerHold`](super::Action::AnswerHold)** answers the invocation parked
//! at one conversation's capability boundary (VISION §4.11 items 5–6, §8.6).
//! The held `tool_use` id is *derived* — read off `refs/litany/held/<agent>` at
//! fire time — never typed, so the answer lands on exactly what is parked now
//! and cannot race. **Nothing there ever calls stop** — a decline is the
//! model's own in-band tool result, which it reads and steps past, and that is
//! a property of what an answer *means* rather than of what a stop costs: the
//! cost changed when litany bl-b98d landed
//! ([`Interrupt`](super::Action::Interrupt)), and this is unaffected by it.
//!
//! Its `answer` is the verdict **and the scope it stands over** (bl-94a5):
//! `pass` releases, `refuse` declines in band, `hold` pins the park; `call`
//! settles the held call alone, `conversation` and `workspace` settle its whole
//! class ([`crate::control::judge::Answer`]).
//!
//! **[`Floor`](super::Action::Floor)** raises or lowers one conversation's
//! capability floor (VISION §4.9's fifth rung, §4.11 item 7, §8.6): under a
//! raised floor every effect class above `read` adjudicates to a hold, so a
//! drone keeps reading, keeps its branch and keeps its history, and everything
//! it reaches for waits on an operator instead of executing. Lowering is the
//! symmetric restore — the fold is latest-row-wins, so the two directions are
//! one gesture, never an order anyone has to get right.
//!
//! **A verdict is an input to this; it is never a substitute for it.** The
//! monitor rules whether work serves the goal, the capability boundary rules
//! what an agent may ever do. That is why it sits beside
//! [`AnswerHold`](super::Action::AnswerHold) rather than inside
//! [`Monitor`](super::Action::Monitor): §4.9's ladder spends existing verbs
//! from the families that own them — notice is `message`, stop is `stop` — and
//! this rung belongs to the capability family.
//!
//! Its `agent` is the conversation the floor is written for. It stands over
//! that conversation **and its whole descent** — the fold matches by hyphenated
//! prefix ([`crate::control::judge::Answers::floored`]) — so flooring a parent
//! floors a subtree without enumerating one, children not yet born included.
