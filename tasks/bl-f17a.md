+++
title = "right-click a conversation → delete the agent"
created = 1785645885
updated = 1785648192
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-5134"
on = "claim"
+++
Operator request 2026-08-02, verbatim: 'I need to be able to right click -> delete an agent'. Add a context menu (egui context_menu) on conversation rows (chat list; descent-tree members if it generalizes) with Delete. Constraints: (1) deletion is yog's destructive-verb class — DESIGN §3.6 defines the class and its confirm discipline (name what dies: the agent, its children, pending inbox), scaled sanely. (2) yog may not hand-delete lernie-owned state — I2 forbids a direct write inside a workspace; the removal must be a lernie verb yog spawns. (3) A delete of a live agent must stop it first (Stop exists) or refuse while live.

## STILL BLOCKED — blocker moved from "no verb" to "verb unpublished" (re-checked 2026-08-01, Hazel)

Nothing implemented; no yog code touched. **The lernie verb now EXISTS and is
exactly what this ball asked for — but it is not on crates.io, and yog links
lernie from the registry, so yog cannot dispatch it.**

### What changed since delete-fixer's investigation

lernie store **bl-0d9e closed** (2026-08-02T04:52:15Z, delete-builder), landing
`~/dev/lernie@3423b9d`: `src/cmd/delete.rs` + `Command::Delete` +
`src/archive/delete/{mod,subtree}.rs`. Signature verbatim from the source:

    lernie delete <workspace> <agent> [--children] [--dry-run]

It satisfies all six requirements this ball spec'd, verified against the source:

1. **Subtree semantics explicit** — bare form returns
   `DeleteError::HasDescendants { id, descendants }`, whose message names them
   and says "pass --children to remove the whole subtree".
2. **Every slice** — `DIRS = [AGENTS_DIR, STEPS_DIR, INBOX_DIR]` plus the
   `refs/lernie/*` marks (`mark_refs`) and the agent refs.
3. **Refuses under a live driver** — `DeleteError::Driven { id, lock }`:
   "an executor holds its lock at {lock} (ARCH §2.11); stop it first
   (`lernie stop`) and delete once it is quiescent" (`require_quiescent`).
4. **Pending deposits surfaced** — `DeleteReport.pending_deposits: usize`,
   doc'd as "mail addressed *to* these agents, which dies with them".
5. **Convergent on re-run** — existence is deliberately not guarded: "delete's
   postcondition — this id has no state — an absent agent already satisfies.
   Declining it would make crash recovery a special case."
6. **`--dry-run`** returns the same `DeleteReport` with `removed: false` — doc'd
   as "the census a caller's confirmation enumerates". That is yog's §3.6 dialog
   contents, computed by the substrate rather than re-derived by the UI.

### Why yog still cannot ship it

yog does not exec a host `lernie`. `Binary::Lernie.self_multiplexed()` is
`true` (`src/cli_outbound/resolve.rs`), so every `lernie` argv re-enters yog and
runs against the **linked** crate via `src/multiplex/lernie.rs`
(`cli.command.run(&mut fx)` over `::lernie::cmd`). `Cargo.toml` pins
`lernie = "=0.0.3"`.

- crates.io index (`https://index.crates.io/le/rn/lernie`) lists **0.0.0, 0.0.1,
  0.0.2, 0.0.3** — nothing newer. `cargo search lernie` agrees: `0.0.3`.
- The registry source `lernie-0.0.3/src/cmd/` has **no `delete.rs`**.
- `~/dev/lernie` is still `version = "0.0.3"` with 11 commits past the `v0.0.3`
  tag; `3423b9d` is HEAD, unpublished and un-versioned.

So `yog lernie delete …` would clap-fail at runtime ("unrecognized subcommand").
Wiring the menu now ships a destructive verb that cannot work.

AGENTS.md rule 6 is registry-only, "no exception in force". The one interim
git/rev exception is a **last resort** that re-blocks `make publish` for the
whole crate — not a trade to make unilaterally for a p3 wish. Not taken.

### The exit, in order

1. lernie cuts a release carrying `3423b9d` (lernie store's own ball; NB
   lernie bl-c8ed — the agent-naming work bl-5134 anticipates — is **not** on
   lernie main yet, so the next publish carries delete regardless of naming).
2. **bl-5134** (this store) bumps `lernie = "=0.0.3"` → the published version.
   Now recorded as a real `--needs` edge on this ball rather than prose: both
   balls live in the yog store, so the dependency is expressible.
3. Re-claim this ball and implement the yog side below.

### The yog-side design, ready to implement once the pin moves

- **Confirm ruling (record in DESIGN §3.6's confirmation doctrine, where the
  class is defined): a plain explicit confirm, NOT typed-name — but only when
  the target is a leaf.** The doctrine grounds typed-name in blast radius: the
  workspace verb "takes a sphere wall down, and the wall's contents go with it".
  One leaf agent is one conversation — the row the operator is already pointing
  at, and the dialog *shows* the name it would have them retype. **A subtree
  delete (`--children`) is a different animal** and should take the typed name:
  it destroys conversations that are not the row under the pointer. That gives
  the class a *principle*, not a per-verb answer: **typed-name confirm iff the
  verb destroys objects beyond the one named on screen.** Workspace delete
  satisfies it, leaf-agent delete does not, subtree delete does.
- **The dialog enumerates from `lernie delete --dry-run`**, not from a yog
  re-derivation: the agent, each descendant by name, and the pending-deposit
  count come straight off `DeleteReport`. Single source of truth — the
  substrate that performs the act computes the census. It need not release
  balls: a ball is claimed by a *workspace*, not an agent (§3.2 claimant join is
  workspace-name equality), so a single-agent delete releases nothing.
- **No keyboard binding, ever** (§3.6, §11 rule 3 at its limit).
- **The gate** reuses `crate::delete::Confirmation`'s live-derivation: refuse
  while the agent (or any descendant being deleted) probes Live/InFlight, §10's
  "?" counting as live — fail closed, name them, let the operator Stop first.
  Do **not** fold Stop into Delete (§3.6 rejected (c)). lernie's `Driven` error
  is the substrate's independent fail-closed under the race; yog gates first.
- **The context menu is NOT the sole carrier.** §11: "every verb must survive
  context-menu deletion … It is never the sole carrier of any verb". So the
  operator's literal request needs a **visible carrier too** — a worded, ichor
  `delete this conversation…` row on the inspector's per-conversation surface,
  mirroring the workspace verb's config-mode danger-zone row. §11's seat table
  entry for `conversation row` grows from `Stop (+children), Flush` to
  `Stop (+children), Flush, Delete…`, naming that row as the visible carrier.
- **Files it will touch** (all exist; line counts are today's, cap 300):
  `src/nav/menu.rs` (`Verb::DeleteAgent`, its `Entry` carrier string, the seat
  row), `src/shell/menus.rs` (131) dispatch, `src/shell/conv_list.rs` (181) /
  descent-tree rows for the attach point, a sibling of `src/delete/{mod,exec}.rs`
  (the plan steps differ entirely — no `bl unclaim`, no workspace `ui.json`
  prune), `src/app/deletes.rs` (116) for the fire-time re-gate,
  `src/shell/delete.rs` (158) for the dialog, `src/ui_state/prune.rs` (90) for
  the deleted agent's `seen[ws][agent-id]` watermark, and `docs/DESIGN.md` §3.6
  + §11's seat table + §12's module map.

### A DESIGN bug to fix in the same edit (independent of the above)

DESIGN §3.6 asserts lernie's "retention (§9.2, 30-day default) is *branch*-level
GC inside a living workspace". **That premise is false against lernie 0.0.3** —
the shipped crate has no retention or GC of any kind (delete-fixer's grep found
zero ref-deleting call sites in prod). §3.6's *conclusion* survives (yog owns
whole-workspace disposal because it owns placement), but the supporting clause
must be corrected. Also §3.6's **Rejected (d)** — "*an upstream `lernie delete`
verb* — disposal belongs to whoever owns placement, and that is yog" — is
scoped to *workspace* disposal and must say so explicitly, since lernie places
agents and now ships exactly that verb for them. The two rulings are consistent;
the doc must show why.

### Standing down

Unclaimed, not closed. Blocked on bl-5134 (the pin bump), itself blocked on a
lernie publish.