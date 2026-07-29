+++
title = "runaway recursive compaction dispatch blows through max_depth budget and leaves unresolved git conflict markers committed into a live summary"
created = 1785287779
updated = 1785287779
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["investigation"]
+++
## Origin
Follow-up investigation, operator report: "agents still seem confused about usage. see chat <agent-id>". This conversation reproduces the known toolset-gap defect (bl-bd9d/bl-55b1 — evidence appended there), but it ALSO exhibits a second, unrelated, and much more severe defect in lernie's automatic intermediate-compaction machinery. Filing separately because the layer, mechanism, and severity are distinct from the tools/grant confusion.

## Evidence

Workspace: /home/u/.local/share/yog/workspaces/<workspace>/repo.git. Root conversation: agents/<agent-id>, goal "send a message to another agent, don't care which or what".

Within about 90 seconds of wall-clock time (branch timestamps 20260728T080619Z through ~20260728T080750Z), this ONE root conversation spawned:

    git -C repo.git log --all --oneline --grep="^dispatch: compactor" | wc -l    -> 226
    git -C repo.git branch -a | grep -c 3b14deaf                                 -> 227

226 automatic `dispatch: compactor` commits, 227 total branches, all descending from one root. Sample descent chain, six generations deep:

    agents/<agent-id>
      -<agent-id>
        -<agent-id>
          -<agent-id>
            -<agent-id>
              -<agent-id>

Every one of these is a `compactor`-role dispatch (confirmed via `goal.md` on each: "You are the compactor for branch `<parent>`. Read the branch's transcript ... produce a signal-preserving, minimal view ... using the `write_summary` tool"). This is the harness's own automatic procedure (lernie ARCH.md §2.7 Compaction, §6 workflow: `worker_flush: dispatch(compactor)`, `compactor_return: compaction_merge`), not anything the model elected — none of these branches used a `dispatch` tool call (the worker role doesn't even have `dispatch` in its `tools:` list, confirmed in bl-bd9d).

The workspace's own `config/default:workflow.yaml` sets:

    compaction:
      intermediate:
        trigger: every_n_commits
        n: 20
    budgets:
      max_total_tokens: 2000000
      max_wall_seconds: 3600
      max_depth: 4

`max_depth: 4` is a configured, supposedly-enforced budget ("ARCH §6 Budgets... Checked at every model-call boundary before the adapter is invoked... depth ... derived from disk each check"). The observed descent chains reach at least depth 6 (root + 5 descent segments, sample above), and there are 226 compactor dispatches total — this is not a near-miss, it is the depth budget being blown through wholesale, consistent with the `every_n_commits` compaction trigger applying to compactor branches THEMSELVES (each compactor branch's own step-commits presumably re-trip `worker_flush` and get a compactor dispatched on top of it), with no evidence the `max_depth: 4` guard ever stopped the cascade.

## The corrupted result
The compaction merge mechanism (ARCH §2.6, "the one merge in the system") failed under this concurrent/recursive load and left a live, unresolved git conflict checked into the ROOT branch's tip:

    git -C repo.git rev-parse agents/<agent-id>
    -> 1596809d26fa89e2f01cb2c2c22cb4501ff62209   (subject: "compaction merge [...0c4c53c6]")

    git -C repo.git show agents/<agent-id>:summary/001.md | grep -c '^<<<<<<<\|^=======\|^>>>>>>>'
    -> 9

`summary/001.md` at the current HEAD of the root branch contains THREE nested, fully unresolved 3-way git conflicts (`<<<<<<< HEAD` / `=======` / `>>>>>>> agents/...-0c4c53c6`, `...-1f56f40a`, `...-153d5f01`), each side an independent compactor's own summary of the same underlying transcript. This is not a rare/edge merge — it is the terminal state of the root conversation right now. Per manifest.yaml (`worker: order: [summary/**, ...]`), `summary/**` is composed into every future model call's context on this branch, so the NEXT step of this conversation will feed raw, triple-conflicted, self-contradictory diff markup to the model as if it were a clean summary — likely to produce yet more confused-looking agent behavior that would be misdiagnosed as a model or tool-use problem, same as the operator's original complaint pattern.

## Layer at fault
lernie's automatic intermediate-compaction procedure and its budget enforcement (ARCH.md §2.6 compaction merge, §2.7 compaction, §6 workflow-configured `compaction.intermediate` trigger and `budgets.max_depth`), pinned crate `lernie = "=0.0.2"`. Not the model (gpt-5.4 never chose to dispatch anything here — these are harness-internal procedure dispatches), not yog.

## Not yet determined (would need code-level, not transcript-level, investigation)
- Exact reason the `every_n_commits: n=20` trigger fires again almost immediately on a freshly-dispatched compactor branch that has produced only a handful of its own commits (the observed generations are ~5-13 seconds apart) — possibly a commit-count accounting bug that doesn't reset/scope per-branch.
- Exact reason `max_depth: 4` did not halt dispatch before depth 6+ was reached — possibly the depth budget check applies only to explicit tool-initiated dispatch (the `dispatch` tool) and not to workflow-triggered procedure dispatch (`dispatch(compactor)`), which would be an enforcement gap between two dispatch call sites that should share one guard.
- Exact reason the compaction merge (`merge=ours`-style discipline per §2.6) produced literal unresolved conflict markers instead of either resolving cleanly or refusing loudly via the documented `refs/lernie/conflicted/*` escape hatch (§2.6 mentions this hatch exists for exactly this kind of situation, but it was seemingly not used here — no `refs/lernie/conflicted/<agent-id>` was checked yet in this investigation, worth confirming as a next step).

## Suggested fix direction
1. Scope/reset the `every_n_commits` intermediate-compaction counter per-branch-since-its-own-dispatch, not inherited or globally accumulated, so a freshly dispatched compactor doesn't immediately re-trigger compaction on itself.
2. Make `budgets.max_depth` (and the other budget axes) apply uniformly to EVERY dispatch call site — explicit tool-initiated and workflow-triggered procedure dispatch alike — with one shared enforcement function, not two.
3. Make the compaction merge fail loudly (refuse the merge, mark `refs/lernie/conflicted/*`, surface to the UI) rather than silently committing raw conflict-marker text into a file that gets fed back into model context as if it were clean.
4. As a cheap mitigation independent of the above: never compose a `summary/**` file into context if it still contains literal `<<<<<<<`/`=======`/`>>>>>>>` lines — treat that as a hard assembly-time error rather than trusting the file's contents blindly.

## Workspace-wide cost
This one root conversation left 227 branches in `<workspace>/repo.git` (up from 2 before this incident), which will slow every future `git for-each-ref`/`branch -a`/yog agent-tree enumeration in this workspace and clutter any UI rendering the agent tree (yog DESIGN.md §5.1 #8 "Agent set, descent, tips" derives directly from `git for-each-ref agents/*`).