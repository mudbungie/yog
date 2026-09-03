+++
title = "seat-shaped derivations still in the server with no boundary spelling: drafts, enabled, the name preview, the fork composer"
created = 1788414434
updated = 1788415504
claimant = "Spellbind-M"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["boundary"]
+++
Found by the §11-rule sweep (bl-7dca). Each of these is a `pub` derivation in
this crate that nothing but its own tests reaches, and no §8.5 query or reply
carries what it computes. They are the residue bl-7942 did not sweep: the face
they served is the seat crate's.

- `src/actions/drafts.rs` (`Drafts`, `DraftKey`) — DESIGN §5.3/§11: unsent
  text keyed by the target it was typed for. A draft crosses no boundary by
  ruling (§8.5: views never cross), so this is a seat's RAM held in a server.
- `src/actions/enabled.rs` — the §8.2 "is there anything to fire" predicates
  over a *selection*. A selection is a seat's (§13.1), and every executor
  already refuses exactly what these predict, so a seat cannot ask and does not
  need to.
- `src/start/goal.rs::preview` + `src/start/identity.rs::identity_preview` —
  DESIGN §3.3's composer prefill and the greyed `will be named <name>`
  prediction. `Prepared` (`src/start/exec.rs`) carries workspace, binding,
  lineage, goal and origin and **no predicted name**, so `/prepare` answers a
  seat nothing it could preview; the seed-continuity rulings (bl-28ba, bl-dd3d)
  are about a preview→fire pair this crate no longer has both halves of.
- `src/fork/composer.rs` — VISION V2's ×N control. `Action::Fork` carries one
  attempt by ruling (§8.5) and fires N times, so the composer is a seat's
  arithmetic.

The question each poses is the same and is not answered here: does the fact
belong in a reply (and which), or does the module belong in `lernie`? Either
answer deletes code from this crate or adds a spelling to §8.5; keeping the
derivation with no carrier is the one thing that is certainly wrong, because it
is 100%-covered code proving nothing about a running system.

DESIGN §3.3, §3.4, §5.3 and §12's rows for these modules describe them in
face terms and must be amended with whichever answer lands.

---

Two more found later in the same sweep, same shape: `src/science/compose.rs` (V3's four fan-group affordances, composed as text for a seat's composer) and `src/science/respdiff.rs` (V3.3's line LCS over two candidates' terminal responses). Neither has a caller outside its own tests. `Reply::Science` carries each row's `response` string, so a seat CAN diff them — which is the second-implementation trap this crate exists to avoid — and carries no draft text at all. DESIGN §3.9's seat paragraph cited `science::render`, a module that does not exist; the sweep replaced that citation with this ball's id.

---

Two more, both bl-1fd0's: `crate::start::StartGate` and `crate::start::WallCredit` (`src/start/gate.rs`) — the *sign-in first* rung's three-state fold over a wall's provider rows. Reached by nothing but `src/start/tests/gate.rs`. Its inputs DO cross (`Query::Providers` carries the §5.1 #22 credential presence per row), so unlike `respdiff` a seat can honour the ruling from what it is already told — but the fold itself, and the wording of what the three states mean, would then be the seat's second implementation. Same question as the rest: name a carrier for the answer, or move the module. DESIGN §8.1's sign-in-rung paragraphs now say so and cite this ball.

---

Also `AppModel::last_failure` / `SurfaceFailure` (`src/app/balls.rs`) — no production caller — but that one has its own ball because the missing carrier is larger than the module: bl-4d81.

---

Landed as DELETE, with one dead premise corrected.

Deleted: `src/actions/drafts.rs` (+ `ActionsState` and `draft_clears`, which held and cleared them), `start::goal::{preview, Composer}` + `start::identity::identity_preview`, `src/fork/composer.rs` (and with it `fork::choices`' `Choices`/`ForkPoint`/`choices()`/`pool()` and the now-unread `CONFIG_REF`, all callerless once the composer went), `src/science/{compose,respdiff}.rs`, `src/start/gate.rs` (`StartGate`/`WallCredit`). About 1780 lines out.

**`src/actions/enabled.rs` was NOT callerless and the ball's premise on it is dead.** `src/boundary/answer/agent.rs` carries three of its predicates into the §8.5 agent answer — `nudgeable: nudge_enabled(..)`, `stoppable: stop_enabled(..)`, `stop_children: stop_children_offered(..)` — so a seat is TOLD, not asked to re-derive; `assign_enabled`/`close_enabled`/`unclaim_enabled` are read by `src/nav/menu.rs`. The file stays. The one member with no carrier went: `message_enabled(present, content)`, whose `content` half is a composer's text — the answer already states `present`, the half a boundary can know.

Same shape, still unfiled, deliberately left: `src/nav/menu.rs` (the §11 context-menu roster — a pure seat table reached by nothing but itself), and the actions root's remaining form predicates `work_dir_refusal`/`goal_present`/`new_prompt_enabled`/`create_ball_enabled`/`new_ball_hints`, none of which has a production caller. They are questions about *inputs* rather than views, and they were outside this ball's list, so they want their own ruling.
