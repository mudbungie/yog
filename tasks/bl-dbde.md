+++
title = "the remote teleoperator cannot set priority, blockers, hierarchy or tags—the facts its fleet schedules"
created = 1787206351
updated = 1787275064
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["missing", "boundary", "balls", "agentic"]
+++
Yog's board reads balls' `priority`, `tags`, `parent` and blocker graph, orders ready work by priority, and the fleet selects that order.

The control boundary exposes only title/body/name on create and update. Balls itself supports priority, tags, parent, subtasks and arbitrary claim/close blockers. A local operator can escape through `yog exec bl`; a remote seat cannot.

Consequently a remote coordinator can arm the fleet but cannot reprioritize its queue, gate dependent work, express decomposition, or add policy tags through yog. Those are not optional metadata: they decide what the autonomous loop runs.

## Required design result

Expose the scheduling facts needed for whole-day teleoperation through the existing create/update boundary without cloning balls' grammar or storing a second representation. Define read/write parity, validation and help for each accepted fact, plus a real boundary drive that creates priorities and blockers and observes the resulting board/fleet order.

---

Premise verified at HEAD in full. It parks on one thing: the fix cannot be made without first ruling how the `bl` family is carried on the action roster, and that is a DESIGN §12 decision.

## Verified

- `src/boundary/action.rs:99` `Action::Create { project, title, name, body }` and `:106` `Action::Update { project, id, name, title, body, note }`. No priority, tag, parent or blocker on either.
- `src/actions/verbs/balls.rs:130` builds `[create, title, --as, name]` plus an optional `--body`, and `:172` builds `[update, id, --as, name]` plus optional `--title/--body/-m`. Nothing else is ever appended.
- `src/boundary/line/verbs.rs:125` `args::only(&flags, &["body"], verb)` and `:140` `args::only(&flags, &["title","body","note"], verb)` — the line grammar refuses every other flag by name.
- balls does support all of it. `bl create --skill`: `-p N`, `-t TAG` (repeatable), `--parent ID`, `--subtask-of ID`, `--needs ID[:OP]`, `--blocks OP|ID:OP`. `bl update --skill`: `--parent|--no-parent`, `-p|--no-priority`, `-t|--no-tag TAG`, `--needs|--no-needs ID`.
- yog already READS every one of these: `src/projects/balls.rs:38` `Ball` carries `priority`, `tags`, `parent`, `blockers`, and the board orders on priority while `fleet::pilot::plan::spawn` takes the board's first ready row. So the loop schedules on facts no remote seat can write. Read/write asymmetry confirmed, exactly as filed.
- The local escape is real and the remote one is absent: `src/multiplex/bl.rs` is a CLI arm ("balls' own thin bin, verbatim") reached by running the binary. Nothing in `src/wire/` or `src/registry/` routes to it, so a wire seat has no path to it.

## Why it parks

`src/boundary/action.rs` is at 297 lines against the 300 hard cap. Every shape of this fix adds to it, so the first act is structural, and DESIGN §12's own row for that file already legislates the area — it spends a paragraph on when a family may fold onto its own `Verb` ("a family whose members every layer beneath already reads as a pair folds to ONE variant over that family's own `Verb`... which is a real seam and not a line budget precisely because the seam is already drawn three files down"). Picking among the shapes below is applying that rule, not implementing a decided thing.

The brief also contains a tension that decides the shape and is not resolved in it: "expose the scheduling facts... without cloning balls' grammar". balls' grammar for these facts is inherently a set of flag applications with set/clear/repeat semantics. A faithful boundary spelling is a clone of it; a non-clone loses power. Which side to land on picks the shape outright.

## The three shapes, with their costs

1. **Fold the `bl` family onto its own `Verb`** — `Action::Balls { project, verb: BallVerb }` over close/assign/release/move/create/update. §12's sanctioned move, and the seam is already drawn three files down (`codec/balls`, `answer/balls`, `verbs/balls`). Widest churn: every dispatch arm, codec arm and line verb. Leaves the most room afterwards.
2. **One folded field list** — `fields: Vec<BallField>` on both variants, in balls' own vocabulary (`Priority(Option<i64>)`, `Tag{name,on}`, `Parent(Option<String>)`, `Needs{edge,on}`), folded to argv in `verbs/balls.rs` and validated *by balls* (§8.2: its stderr is the product, so yog never re-implements the cycle refusal). Handles repeatability, which N typed options cannot. Still needs room in `action.rs`.
3. **Move the payloads into structs** — `Action::Create { project, name, fields: BallCreate }`, `Action::Update { project, id, name, fields: BallUpdate }`, owned by `verbs/balls.rs` beside the `Update` struct that already exists there for exactly this reason ("a five-line literal costs the chokepoint its line budget (§12)"). Precedent exists — `Action::Prepare` already carries `start::Payload`. Shrinks `action.rs`. Touches all 27 construction sites across 12 files.

Every shape touches all 27 `Action::Create`/`Action::Update` sites regardless, since enum-variant construction has no `..Default`.

## Scope note for whoever rules

Four facts cover the brief and the rest is derivable sugar, which is worth stating because it keeps the surface small: priority, tag, parent, needs. `--blocks OP|ID:OP` and `--subtask-of ID` are both expressible as a second op — `--subtask-of E` is `--parent E` plus `bl update E --needs <new>:close` — so neither needs a boundary spelling. Clearing forms are no-ops at create (a new ball's fields start empty), which is the general path with an empty input rather than a create/update split.

Not touched: no code or doc edited under this ball.
