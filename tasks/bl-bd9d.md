+++
title = "worker role's default toolset omits message/dispatch — root agents can never message siblings or spawn subagents out of the box"
created = 1785287600
updated = 1785373701
claimant = "entrance-bd9d"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["investigation"]
+++
## Origin
Investigating operator report on conversation <agent-id> ("it's clearly not capable of using the tools correctly"). Model behavior was actually CORRECT; the real defect is upstream in config.

## Evidence
Workspace: /home/u/.local/share/yog/workspaces/<workspace>/repo.git, branch agents/<agent-id>.

The user asked the agent to `send a message to <agent-id>` (messages/003-user.md). The agent (gpt-5.4/codex) repeatedly said it could not, e.g. messages/004-gpt-5.4.json:

> "I can help draft a message, but I can't actually send one from here."

and after inspecting its own environment (messages/006, 008 — bash exploring `descriptions/`), messages/018-gpt-5.4.json:

> "I can see the `message` skill instructions now, but I still can't invoke a `message` tool directly from the callable tool interface available in this conversation. Only `bash`, `read_file`, and `load_skill` are actually exposed to me as callable tools here."

This is TRUE, not a hallucination or tool-grammar bug. Confirmed by reading the actual wire request sent to the model:

    python3 -c "import json; d=json.load(open('.../steps/<agent-id>/006/request.json')); print([t['name'] for t in d['tools']])"
    -> ["bash", "read_file", "load_skill"]

This matches the workspace's config/default:providers.yaml verbatim:

    roles:
      worker:
        provider: codex
        model: gpt-5.4
        tools: [bash, read_file, load_skill]
      compactor:
        provider: codex
        model: gpt-5.4

No `message`, no `dispatch`. Per lernie ARCH.md §4.3: "A root records no role and resolves the worker default — roots are workers." So every root/interactive conversation in every yog-started workspace is capped at exactly [bash, read_file, load_skill] and can NEVER message a sibling agent or dispatch a subagent, unless an operator hand-edits providers.yaml after creation.

Checked 3 separate workspaces (<workspace>, <workspace>, <workspace>): all three have byte-identical providers.yaml worker.tools lists. This traces to lernie's own shipped template, byte-identical:

    ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lernie-0.0.2/template/providers.yaml
    /home/u/.local/share/yog/world/lernie/template/providers.yaml

Both read:

    roles:
      worker:
        tools: [bash, read_file, load_skill]
      compactor: {}

## Layer at fault
lernie's own default template (external pinned crate `lernie = "=0.0.2"` in yog's Cargo.toml), NOT the model, NOT yog's dispatch code, NOT a tool-call-syntax bug. The model's refusal was accurate; the operator's read ("not capable of using the tools correctly") is a reasonable but incorrect diagnosis of a config-authoring gap one layer up.

## Suggested fix direction
Either (a) upstream in lernie: change the shipped template's default `worker.tools` to include `message` (and arguably `dispatch`, since subagent delegation is also a core primitive никогда usable out of the box otherwise), or (b) in yog's own workspace-seeding step (DESIGN.md §16.2/§16.5, the world materialization that calls `lernie new`): author/patch providers.yaml post-creation to grant message+dispatch to the worker role by default, since yog's whole premise is multi-agent orchestration (dispatch children, message between conversations) and a worker that can't do either out of the box contradicts that premise. Whichever fix lands, it should not be silent — the current failure mode (agent looks broken, spends 3 user turns diagnosing a config gap) is expensive and confusing.

## Not at fault
- The model (gpt-5.4/codex): behaved correctly, did not fabricate a tool call, gave an accurate (if verbose) explanation once it investigated.
- lernie's wire/tool-call parsing: request.json confirms tools array was assembled exactly per providers.yaml; no malformed calls, no truncation, no lost tool_results.

## Reproduced again — conversation <agent-id>

Operator report: "agents still seem confused about usage. see chat <agent-id>". Same workspace (<workspace>), same root cause, independently confirmed.

Goal (goal.md): "send a message to another agent, don't care which or what".

The agent's first response was plain text instead of a tool call (messages/002-gpt-5.4.json: "Hello from this agent to another agent: hi."), then after a user correction (per the compactor's summary, since the raw user turn was later deleted by compaction — see bl-<compaction-storm task> for that separate bug) it investigated its own tools:

    messages/011: read_file descriptions/tools/message.json
    messages/013: load_skill("message")
    messages/015-020: tries to derive its own/a sibling's agent id via bash/python string-splitting on $PWD's basename (guesses "20260728T080619Z", which is not a valid agent id)
    messages/021-026: bash-lists sibling agent dirs and their goal.md files looking for a recipient
    messages/027 (final): "I need the actual `message` tool available in this tool interface to do that. In this branch, I can inspect the skill description for `message`, but the callable tools exposed to me here are only `bash`, `read_file`, and `load_skill`, so I can't deposit an actual message."

Confirmed via `steps/<agent-id>/*/request.json`: wire `tools` array is again exactly `["bash","read_file","load_skill"]` — no `message`. This is the identical defect already filed here: worker role's default toolset omits `message`, so a root agent given an explicit "send a message" goal structurally cannot comply, no matter how much it investigates. Two independent conversations, two independent operator complaints, same root cause.

New minor symptom this time (worth folding into the fix, not a separate task): because the agent has no way to learn a valid agent id short of `bash`-exploring sibling directories under `../`, it burned several steps trying to reverse-engineer an address from its own branch dirname before falling back to directory-listing siblings — another consequence of bl-55b1 (no role-scoped ground truth reachable from the agent's own worktree), compounding the confusion once `message` is eventually granted.