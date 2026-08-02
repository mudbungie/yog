+++
title = "right-click a conversation → delete the agent"
created = 1785645885
updated = 1785648179
claimant = "Hazel"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator request 2026-08-02, verbatim: 'I need to be able to right click -> delete an agent'. Add a context menu (egui context_menu) on conversation rows (chat list; descent-tree members if it generalizes) with Delete. Constraints: (1) deletion is yog's destructive-verb class — DESIGN says workspace deletion is v1's ONLY member, with typed-name confirm enumerating what dies and no keyboard path; deleting an agent joins that class: same confirm discipline (name what dies: the agent, its children, pending inbox), scaled sanely (a click-confirm dialog; decide whether typed-name is warranted for a single agent vs the whole workspace — record the ruling in DESIGN). (2) Find lernie's actual removal surface first: does lernie 0.0.3 expose deleting an agent (agents/* ref + files + steps), or is there only yog-step delete-workspace? If lernie has no lawful removal verb, DO NOT hand-delete lernie-owned state from yog — write findings + the lernie-side ask into this ball and stand down (cross-repo, cf. bl-50f3). (3) Running driver: a delete of a live agent must stop it first (Stop verb exists) or refuse while live.

## BLOCKED on lernie — investigation complete 2026-08-01 (delete-fixer)

Constraint (2) fires: **lernie 0.0.3 exposes no agent-removal surface.** Nothing
was implemented; no yog code was touched. This ball cannot proceed until lernie
ships the verb.

### Evidence (registry source, `~/.cargo/registry/src/index.crates.io-*/lernie-0.0.3`)

`src/cmd/mod.rs` is the single authoritative definition of what lernie can do
("This module is the one authoritative definition of what `lernie` can do").
`enum Command` is exhaustively: `New, Config, Prompt, Dispatch, Stop, Message,
Scan, Bundle, Replay, Advance, Tool, Prime`. There is no delete/remove/rm/prune
verb, and no flag on an existing verb that removes:

- `bundle` (`src/cmd/bundle.rs`) — `Args { workspace, agent, out_dir }`. Pure
  copy-out; no `--remove`. Archives the `agents/<id>` subtree + governing
  `config/*` refs into one git bundle, copies the `steps/<id>*` / `inbox/<id>*`
  slices beside it. Non-destructive by construction.
- `scan` (`src/cmd/scan.rs`) — `Args { workspace }`. Silent-death sweep + inbox
  flush; it *marks* (`refs/lernie/abandoned/<id>`), never reaps.
- `replay` — creates a scratch workspace under `replays/<id>/`; no cleanup verb
  for those either.

A tree-wide grep for ref deletion (`update-ref -d`, `branch -D`, `--delete`,
`gc`) over `src/**.rs` finds **zero** hits in prod code. Every `update-ref` call
site (`prompt/budget`, `prompt/workflow_actions`, `compactor/merge/decline`,
`dispatch/transfer`, `dispatch/terminal`) *writes* a mark. The only `fs::remove*`
in prod code is transient scratch: `prompt/dispatch/transfer.rs` (a patch file),
`prompt/dispatch/child_result*.rs` (a consumed result file), and
`template/checkout.rs` (an authoring checkout teardown). None touches agent
state.

There is also **no retention/GC of any kind** — README's §9 is archival + the
eval suite only. NB: yog DESIGN §3.6 currently asserts lernie's "retention (§9.2,
30-day default) is *branch*-level GC inside a living workspace". **That premise
is stale against lernie 0.0.3** — no such GC exists in the shipped crate. The
§3.6 conclusion (yog owns whole-workspace disposal because it owns placement)
survives, but the supporting clause needs correcting; folding that into the
DESIGN edit this ball will eventually make is the cheapest place to fix it.

### Why yog must not hand-delete it

An agent is entirely lernie-owned state inside the workspace wall: the
`agents/<id>` ref in the bare `repo.git`, its `agents/<id>-*` hyphen-descendants
(lernie ARCH §2.3 descent grammar), the sibling worktree at `<ws>/agents/<id>/`,
the `steps/<id>/` and `inbox/<id>/` slices at the workspace root, and the
`refs/lernie/{notify,budget-exhausted,abandoned,conflicted}/<id>` marks.

DESIGN I2 (§1) is verbatim: "workspace state via `lernie` verbs only (never a
direct write inside a workspace — ARCH §3.5; removing a *whole* workspace dir is
a write to yog's own names root, not inside one — the §3.6 delete verb)". §3.6
draws the same line explicitly: whole-workspace removal is lawful *because* it is
"a write to yog's **own** names root ... never a write *inside* a workspace, so
I2's lernie-verbs-only rule stands untouched." Deleting one agent is exactly the
write inside a workspace that I2 forbids. §4.1 restates it for refs
specifically: "yog may not delete refs ('the UI is a pure reader'; no ack verb
exists)".

Note also §3.6's **Rejected (d)**: "*an upstream `lernie delete` verb* —
disposal belongs to whoever owns placement, and that is yog". That rejection is
scoped to *workspace* disposal, where yog does own placement (§3.1, the names
root). It does not transfer: **lernie places agents**, so by §3.6's own argument
agent disposal belongs to lernie. The two rulings are consistent, not in tension.

### The lernie-side ask (feeds bl-50f3)

A verb of roughly this shape, so yog can spawn it like any other:

    lernie delete <workspace> <agent> [--children]

Requirements yog needs from it:

1. **Subtree semantics, explicit.** Bare = refuse if `agents/<id>-*`
   hyphen-descendants exist (naming them); `--children` = remove the whole
   descent subtree. Mirrors `stop --stop-children`, so the two verbs read alike.
2. **Removes every slice of the agent**, in one lawful act: the `agents/<id>*`
   refs, the worktrees under `<ws>/agents/<id>*/`, `steps/<id>*`, `inbox/<id>*`,
   and the `refs/lernie/*/<id>*` marks. Partial removal would leave yog's §5.1
   derivations rendering a half-dead row.
3. **Refuses while a driver holds the executor lock** (lernie ARCH §2.11) —
   naming the live ids. yog gates independently (below), but the substrate must
   fail closed under the race; an `rm` beneath a flock-holding driver is a race
   with a running process.
4. **Pending inbox deposits are surfaced, not silently dropped** — a count in
   the error or in the verb's product, so yog's confirmation can enumerate them
   ("N undelivered messages die with it"). Deposits addressed *to* the deleted
   agent die with the inbox; a deposit the agent *sent* is already in the
   recipient's inbox and must survive.
5. **Convergent on re-run** — a delete of an already-absent agent is a quiet
   success, not an error, so yog's plan re-runs cleanly after a crash (the
   §8.1 planner idiom).
6. **`bundle` composes in front of it** — archive-then-delete stays two verbs
   the caller sequences (yog's §3.6 already promises exactly that composition
   for workspaces once bundle is surfaced, §8.3 v1.1).

### The yog-side design, ready to implement once the verb exists

Ordered so the next agent starts from a plan, not from scratch.

- **Confirm ruling (recommended, to be recorded in DESIGN §3.6's confirmation
  doctrine where the class is defined): a plain explicit confirm, NOT typed-name
  — but only when the target is a leaf.** The doctrine's own words are "confirms
  by naming the object", and it grounds typed-name in blast radius: the
  workspace verb "takes a sphere wall down, and the wall's contents go with it".
  One leaf agent is one conversation — its blast radius is a single row the
  operator is already pointing at, and the dialog *shows* the name it would have
  them retype, so retyping adds ceremony without adding a decision. **A subtree
  delete (`--children`, N descendants) is a different animal** and should take
  the typed name: the operator is destroying conversations that are not the row
  under the pointer, exactly the workspace verb's "contents go with it" case at
  smaller scale. That gives the class a *principle* rather than a per-verb
  answer: **typed-name confirm iff the verb destroys objects beyond the one
  named on screen.** Workspace delete satisfies it (N conversations), leaf-agent
  delete does not, subtree delete does. Record it that way in §3.6 and the class
  extends without a new ruling per verb.
- **The dialog enumerates concretely, either way**: the agent by display name,
  each descendant by display name, the count of pending inbox deposits, and
  (unlike the workspace verb) it need not release balls — a ball is claimed by a
  *workspace*, not an agent (§3.2 claimant join is workspace-name equality), so
  a single-agent delete releases nothing.
- **No keyboard binding, ever** (§3.6, §11 rule 3 at its limit).
- **The gate** reuses `crate::delete::Confirmation`'s existing live-derivation:
  refuse while the agent (or any descendant being deleted) probes Live/InFlight,
  §10's "?" uncertainty counting as live — fail closed, name them, and let the
  operator use Stop first. Do **not** fold Stop into Delete: §3.6 already ruled
  "*stop-and-delete as one gesture*" rejected, and "folding a kill into a delete
  would let one gesture destroy running work across two substrates".
- **The context menu is NOT the sole carrier.** §11's context-menu doctrine is
  explicit: "every verb must survive context-menu deletion ... It is never the
  sole carrier of any verb, critical or not — right-click is invisible chrome
  ... with no keyboard path, so a menu-only verb fails exactly the test a
  glyph-only state badge fails." So the operator's literal request (right-click
  → delete) requires a **visible carrier too** — the natural seat is a worded,
  ichor `delete this conversation…` row on the inspector's per-conversation
  surface, mirroring the workspace verb's config-mode danger-zone row. The §11
  seat table's `conversation row` entry then grows from `Stop (+children),
  Flush` to `Stop (+children), Flush, Delete…`, with that row named as its
  visible carrier.
- **Files it will touch** (all already exist; sizes are today's, against the
  300-line cap): `src/nav/menu.rs` (add `Verb::DeleteAgent`, its `Entry`
  `carrier` string, and the seat row), `src/shell/menus.rs` (131) dispatch,
  `src/shell/conv_list.rs` (181) / the descent-tree rows for the attach point,
  `src/delete/{mod,exec}.rs` (the `Confirmation`/`plan`/`execute` triple —
  likely a sibling module rather than a widening, since the plan steps differ
  entirely: no `bl unclaim`, no `ui.json` prune), `src/app/deletes.rs` (116) for
  the fire-time re-gate, `src/shell/delete.rs` (158) for the dialog, and
  `docs/DESIGN.md` §3.6 + §11's seat table + §12's module map.
- **`ui.json` needs a prune too**, narrower than the workspace one: the deleted
  agent's `seen[ws][agent-id]` watermark entry (§4.1). `src/ui_state/prune.rs`
  (90) is where it belongs.

### Standing down

Unclaimed, not closed. Re-claim once lernie ships the removal verb and yog's pin
moves past `lernie = "=0.0.3"`.