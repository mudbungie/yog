+++
title = "capability control: the world tool-control shim classifies and adjudicates every drone tool call"
created = 1785824590
updated = 1785824590
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["capability"]
+++
Implements VISION §4.11 items 1–3 and DESIGN §8.6 (ruled by bl-0cea). Verify premises before building — do not trust this body over the tree.

## Verified premises (2026-08-04)
- The enforcement seam is ALREADY IN THE PIN: lernie =0.0.6 carries the tool-control seam (ARCH §3.3 *Tool control*, bl-de6d shipped 0.0.4; hold-resume fix bl-11af 0.0.5). `workflow.yaml` `tool_control: {command: <exe>}` is consulted before every granted invocation; stdin = the `tool_use` block verbatim + `role` + `agent_id`; stdout = one JSON verdict `pass|refuse|hold` (+ required `reason`); env `LERNIE_CONV_REPO`/`LERNIE_CONV_BRANCH`; cwd = workspace root; fails closed (`Error::ToolControl`); hold parks via `refs/lernie/held/<agent-id>` (value = held id/tool/reason); release is re-adjudication at the next `lernie advance`. NO new lernie primitive is needed; file no upstream ask.
- `template/workflow.yaml` ships no control, and lernie's seeding is seed-if-absent (`src/install.rs::seed_file`), so a yog-authored template survives every later `lernie prime`.
- The worker role's grant is the entire pool (bl-7fc8: yog has no grant path). Grants stay untouched — structure is lernie's; this shim is the policy.

## Build
1. **The control shim**: a `world/tools/` re-exec shim of the yog binary (§16.4 pattern, beside bl/lernie/bz), side-effect-free per consult — it writes NOTHING (re-adjudication demands idempotence); the audit is lernie's own record plus the policy facts' ops rows (bl B).
2. **Classifier** (invocation → VISION §4.11 effect class): intrinsic map for built-ins (read_file=read; load_skill/apply_patch=target-write; message=target-write on the world; dispatch=process; multi_tool=adjudicated per inner by lernie itself, envelope passes structurally; cd=class of its destination); bash via the workspace ruleset (argv patterns over input.command); UNMATCHED BASH = open-world (classification fails toward the safer class). Writable root = the bound attempt worktree ∪ the agent worktree; derive the agent cwd from `refs/lernie/cwd/<agent-id>` (a git read of LERNIE_CONV_REPO) and the ball worktree from the claimant join (computed, bl-delivery formula, never stored).
3. **Judgment**: class→verdict table (shipped defaults: read/target-write/process=pass; open-world=hold; destructive/secret=refuse) folded with per-conversation floors and per-tool_use-id once-answers read from ops.jsonl (bl B owns the writers; unanswered = the table's verdict).
4. **Authoring**: yog writes `tool_control: {command: <world>/tools/<shim>}` (absolute path, not PATH-resolved) into the nested `LERNIE_HOME/template/workflow.yaml` — the I2 lernie-global atomic-replace class, gated like bl-c3a9's provider gate — so every workspace births controlled. Branches forked before the block stay uncontrolled (workflow is frozen at the governing config commit); document, don't paper over.

## Never
- No refusal path may stop an agent: `lernie stop` mid-tool-window wedges the branch permanently (lernie bl-b98d). Refuse = in-band decline; ask = hold/park. The wedge is routed around, not depended on.
- No permission modal, no second adjudicator, no grant-path resurrection.