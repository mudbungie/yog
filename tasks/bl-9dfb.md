+++
title = "in-flight strip gains elapsed: the call's start is in the structure (step/dispatch commits, landed tool inputs), not a file timestamp"
created = 1785729860
updated = 1785730183
claimant = "strip-elapsed"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Operator ruling (2026-08-02, verbatim)

'in-flight strip is also telemetry we have; we know when the commit is made, when lernie/brazen is invoked. it's not a file timestamp per se, but it's definitely in the structure.'

This OVERRULES bl-905f's elapsed refusal (recorded in DESIGN §5.1 #28 and §11): that refusal rejected mtime-of-response (last token, not first), birth time (non-portable), and a stored yog-side flag — all correctly — but did not exhaust the STRUCTURAL starts. The world's own records mark when each call began; derive elapsed from those.

## Verified against the PINNED lernie (`=0.0.3`), `src/prompt/dispatch`

Two candidates SURVIVED, two were REJECTED with evidence. The surviving fact is the same in both drivers — `run_exchange`'s step loop and the `lernie advance` hop (`advance/hop.rs`) — which is what makes it a lernie invariant rather than a path.

**SURVIVED — inference: `steps/<id>/<NNN>/request.json`'s mtime (latest step).**
Both drivers do exactly: `write_request(...)` → `let started_at = clock.now_iso8601()` → `model_call::run(...)`. The stamp on that file and lernie's OWN notion of the step's start are therefore the same instant. Written once per step, never appended.
- `meta.json`'s `started_at` is that same instant *recorded*, but `write_meta` runs only AFTER `call_outcome?` returns — so `meta.json` is absent for exactly the call being timed. It verifies request.json; it cannot replace it.

**SURVIVED — tools: the call's `input.json` mtime.**
`SpawnTool::execute` does `atomic_write_json(dir, INPUT_FILE, ...)` → resolve binary → `let started_at = clock.now_iso8601()` → `spawn_and_capture`. Per call, not per step.

**REJECTED — the tip commit / `tip_timestamp_unix`.**
lernie takes NO pre-call commit from step 2 on (`step_commit` module doc: "Step ≥2 takes no pre-call commit; the branch tip already represents what the model reads"). The tip is whatever the previous step's tool window committed, and `advance`'s warrant is satisfied by a tool-side transcript tail with no fresh commit — so a branch resumed hours after a stop calls the model against an hours-old tip and the strip would read hours into a five-second call.

**REJECTED — the subagents class's dispatch commit.**
Also a ball-body error: `dispatch: <role> [<id>]` (child_dispatch.rs:181) is a CHILD's first commit; a ROOT's is `step 001: dispatch [<conv-id>]` (`step_commit::commit_dispatch`). Either way it fails the tie test the two survivors pass — a start must be tied one-to-one to the thing in flight, and a dispatch commit is written once per branch while the driver over it may be its third run. And the class is by construction the between-steps window (inference and tools outrank it), so there is no call to time at all. That class shows NO elapsed: partial is lawful, invented is not.

**Ball-body correction — "or, if it rides uncommitted".** There is no "or": nothing under `steps/` is git-tracked (lernie §2.3, `step_commit` module doc: "request.json, response.json, and meta.json land outside the worktree and are not git-tracked"). No commit timestamp exists for any step record, so the mtime is the only reading — and the honest one.

## Delivered

- `Agent::call_start_unix: Option<i64>` and `ToolCall::start_unix: Option<i64>`, both stamped at enumerate time beside the presence checks they ride with (`git_tree/enumerate.rs`, `git_tree/tools.rs`) through one `mtime_unix`. The render path stats nothing.
- `nav::convs::strip(agents, root_id, now_unix)` appends ` · <elapsed>` via `age_label` (one home, shared with the list row's age). `AppModel::flight_strip(now_unix)`; the wall clock is minted once at the shell boundary (`shell::now_unix`, moved up from `conv_list` so the ages and the elapsed cannot disagree).
- The strip now reads `◐ inference — a model call is streaming · <who> · N chars streamed · 42s`, `⚙ tools — a tool call is executing · <who> · <tool> · 7m`, `↳ subagents — a dispatched child is running · N children running`.
- DESIGN: new §5.1 row **#28a** (the two structural starts + the lernie verification), §11's third-seat ruling rewritten — the operator ruling verbatim, the three preserved rejections, the tip and dispatch-commit rejections, and the subagents omission as doctrine; §7.2's per-frame note and §12's two module rows amended.
- Tests per class both directions (`nav/convs/tests/strip.rs`), plus end-to-end snapshot-time proof of both starts (`git_tree/tests/starts.rs`).