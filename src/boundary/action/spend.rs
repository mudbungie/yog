//! **Why the two spend acts are shaped the way they are** (bl-53d1): prose
//! only — nothing in this file is read by a program. The split is
//! [`conversation`](super::conversation)'s.
//!
//! **[`Price`](super::Action::Price)** writes one entry of the §3.5 price
//! table — `(provider row, model)` → the four rates — or, with no rates,
//! deletes it. **The table and the ceiling are read and set through the
//! boundary** (DESIGN §3.5, VISION §4.5): §4.1 ruled them *read-only — no
//! setter, no editor, no verb* because a hand edit on the engine's box was
//! live within a tick, which was true while the window ran in the engine's
//! process; since REMOTE §12 the seat is another machine and `ui.json` is on
//! the server, so the ruling's premise is gone and the empty table was the
//! default nobody could leave. `ui.json` stays the one home and the
//! severability stands — delete the key, lose the column — and what is added
//! is the door. The write is `UiState`'s own write-through, the
//! [`Pin`](super::Action::Pin) shape, and the receipt is the **re-derived**
//! `prices` reply rather than an echo (the `marks` precedent).
//!
//! **A row is priced before it is configured, on purpose.** The engine does
//! not refuse a provider row the workspace's wall roster does not know:
//! a table is world config keyed by the row name litany will hand `bz
//! --provider`, and an operator pricing `anthropic` ahead of signing in to it
//! is pricing the right thing. What is refused, in band, is a negative or an
//! unparseable rate — the line's reader refuses those before a gesture exists.
//! The type holds **micro-USD** integers where the wire and the line say USD
//! decimals, because the roster is `Eq` and money on this side of the boundary
//! is never an `f64` (`spend::prices`).
//!
//! **[`Ceiling`](super::Action::Ceiling)** writes the §3.5 spend ceiling or,
//! with no number, deletes the key — one number, USD, the bound the whole
//! world's spend must stay under (bl-a80a). The same write-through, the same
//! receipt. Its **second effect is the release**: the §8.6 consult parks every
//! running conversation at its next tool call once the world is at or over
//! the number, carrying [`CEILING_HEAD`](crate::spend::CEILING_HEAD)'s sentence
//! as the mark's reason, and a `/ceiling` that lifts the world back under the
//! number — or deletes the key, or has no priced table to compare — drives
//! every conversation so parked through the same detached `litany advance`
//! the answer gesture spends, selecting them by that reason. A floor's park
//! and a policy hold carry other words and are not touched; `/answer pass`
//! still walks one held call through. The count rides back as `released`, so
//! a seat can say how many woke; a read of the same reply carries no such key,
//! because nothing was released by looking.
