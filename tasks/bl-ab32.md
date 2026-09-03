+++
title = "the §8.5 line's in-process query arm still resolves over the frame's cached derivation, so a wall born this instant refuses for one pass"
created = 1786845533
updated = 1788408225
claimant = "Spellbind-I"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["residual", "boundary"]
+++
The residual bl-6c9e left, stated so it is not rediscovered as a defect.

bl-6c9e made the **engine's intake** resolve names over the live §3.1 enumeration, so a workspace's birth is a barrier for every gesture and every query that crosses the boundary. One caller was deliberately left reading the frame's cached derivation: `AppModel::boundary_deps` (src/app/balls.rs), which builds the `Deps` for the §8.5 **line's query arm** — the in-process reads the window still answers for itself, plus the acceptance world's stand-in for the transport.

It was left alone for a reason, not by oversight. The frame does no IO (§7.2), so it cannot ask disk; and its own optimistic fold is forbidden here by the standing rule the field's comment states — *the derivation, never the §7.2 fold: a gesture and a machine-facing query may not be decided by a fact that is only optimistic*. Both available answers are refused, so the honest state is: a line query naming a wall born this instant refuses for at most one derivation.

The cost is small and bounded — a read that refuses briefly, never a two-step flow that cannot compose — which is why it did not ride bl-6c9e. Two candidate dissolutions, neither obviously right:

- Migrate the line's query arm onto the same wire path every other window read now takes, at which point the residual dissolves with the caller rather than being fixed.
- Let the arm build its environment where the environment is already built (the intake), which is the same move one altitude down.

Worth taking only when the §8.5 line arm is touched for another reason.

---

Verified against HEAD, not taken. Both premises hold. `AppModel::boundary_deps`
(src/app/balls.rs) still sets `snapshot: Arc::clone(&self.derived)` with no
`app::addressable` call; `ConsumerCtx::deps` (src/boundary/consumer.rs) still asks
disk per gesture. The residual is real and **one caller wide**: `src/shell/slash.rs`
is the only production caller of `boundary_deps` left, everything else being the
acceptance fixture's stand-in transport and test rigs. A `/prepare NAME` followed by
a query naming NAME refuses for at most one derivation, exactly as filed.

Neither candidate is clean today, and (1) is blocked by a fact the body did not carry.

**(1) migrate the arm onto the wire — blocked by `Query::Search`.** DESIGN §8.5 rules
that "Search is the one query a frame hands over instead of running":
`AppModel::answer` (src/app/view.rs) intercepts `Query::Search` ahead of the
chokepoint, seeds **this instance's** `SearchCell`, and returns the previously landed
rows — which `slash::note` discards, because the §11 Search tab paints
`AppModel::found()` off that cell and not off the reply. Post `/search` over the wire
and the engine answers rows nobody reads while the cell is never seeded, so the tab
focuses on nothing. Migrating therefore does not delete the seat's branch, it adds
one: a `Query::Search` special case at the seat restating a rule whose home is the
chokepoint, or a second writer into the search cell fed by a receipt. No other
carrier is free either — the read path (`wire::link`) is a standing question re-asked
at `ASK_PERIOD` forever, which §5.1 #26 turns into a live provider call twice a
second for a latched `/models`; the act path (`wire::post`) is one-shot and ticketed,
but the acceptance fixture asserts "the act path carries no reads" and REMOTE §9.8
records this arm as having no receipt to wait for.

**(2) build the environment at the intake — overturns bl-6c9e's own ruling.** The
mechanical change is one expression (`boundary_deps`' `snapshot` field wrapped in
`app::addressable` over `binding::workspaces`), but `addressable`'s own doc
(src/app/snapshot/names.rs) rules it stands "at the intake and only there", and
DESIGN §8.5 states why: "the frame keeps both its cached copy and its §3.4 raise
claim because a frame does no IO". `boundary_deps`' one production caller is inside
the render pass. Taking (2) is a §7.2 partition ruling, not a fix, and a bounded
one-pass refusal does not buy one.

**What would pay for (1), when the arm is next touched.** This arm is the last
carrier of the `Cli` pair in the window's click glue — REMOTE §9.8 predicted exactly
that ("what still carries it does so for the §8.5 line's *query* arm and nothing
else"), and it checks out: `bl: &Cli` threads `shell::render` -> `pane::render` ->
`input_bar::composer` -> `verb_row::verb_buttons` -> `slash::seat` and is used for
nothing else on the way, with `lernie` unused in the middle three. Migrating deletes
`bl` from the shell's render signature outright and both Clis from three glue files,
collapses the seat's two arms into one post, and leaves `AppModel::answer` with no
production caller at the window — closing the last in-process gesture answer there,
which is §1.2's own claim. That is what pays for the `Query::Search` seam. The
one-pass refusal alone does not.

Stays parked at p3, ready.

---

Taken. The residual dissolved with the frame, and what was left was not deletable.

**Verified at HEAD.** `src/shell/` no longer exists — bl-7942 deleted the window — so
`AppModel::boundary_deps` has **zero production callers**. `grep -rn boundary_deps src tests`
returns one doc reference (`src/app/view.rs`), one doc mention (`src/app/snapshot/names.rs`),
`src/test_support/chrome.rs`, seven in-src test modules, and `tests/integration/support/asked.rs`.
`AppModel::answer` — the other half of the in-process query arm — is in the same state.
So the filed defect (a line query naming a wall born this instant refuses for one pass)
**cannot occur**: nothing in production builds those `Deps`.

**Why deleting it would be a regression.** The one surviving caller class is the acceptance
world's stand-in for the transport (`tests/integration/support/asked.rs::{ask, act}`), and that
is a separate crate: `AppModel::snap`, `AppModel::state_root()`, `AppModel::roots` and
`app::addressable` are all `pub(crate)`/private, so removing this one narrow public door forces
five wider ones onto the production surface for tests to use. One door in, not five.

**So candidate (2) was taken instead** — the body's own "wrap the `snapshot` field in
`app::addressable` over `binding::workspaces`". Its only objection was that "`boundary_deps`'
one production caller is inside the render pass", which is exactly what bl-7942 deleted. Every
caller of this door now stands where the intake stands, so it asks disk for both addressable
sets exactly as `ConsumerCtx::deps` does. That is worth more than the dead defect it fixes: a
fixture resolving names over a *different* set than the engine is a story asserting against a
world production never builds — bl-6c9e's own argument, one altitude down.

Candidate (1) is moot: the `Query::Search` seam that blocked it was `AppModel::answer`'s
interception at the window, and both went with the frame.

**Doc rot corrected**: DESIGN §8.5's "the frame keeps both its cached copy and its §3.4 raise
claim" passage, `app::addressable`'s "at the intake and only there" note, `AppModel::answer`'s
"the same boundary_deps every dispatch takes", and `Caller`'s "the window's click-glue and the
§4.3 pilot each construct one".

**Left standing, out of scope**: DESIGN §8.5's pending-echo bullets still describe `AppModel`
holding `derived` and `snap` and an `src/app/echo.rs` that bl-7942 deleted. That is a whole
departed mechanism, not a sentence, and half-fixing it would be worse than filing it.
