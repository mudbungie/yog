+++
title = "the total router left the shipped worker grant dead: seven of eight granted tools refuse, taking subagents, agent messaging, skills and the conversation's own worktree with them"
created = 1788150349
updated = 1788150349
priority = 9
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["remote", "tool-host"]
+++
REMOTE §5.4's inversion made the router total: `ToolInjection::call` answers the
`clients` tool, the two engine acts, every loaded remote name, and renders
`UNLOADED` for everything else (`src/tool_host.rs`). Nothing resolves a binary
behind it any more.

The shipped worker grant was not moved with it. `providers.yaml` in the first
config commit still grants:

    tools: [apply_patch, bash, cd, dispatch, load_skill, message, multi_tool, read_file]

Seven of those eight are now dead names. They are declared to the model on every
single request — they are in the toolset the grant gate enumerates, their
schemas ship in `descriptions/tools/*.json`, their skill docs ship in
`descriptions/skills/*.md` — and every one of them earns the same refusal:

    <name>: no tool of that name is loaded in this conversation; use the clients
    tool to see this workspace's machines and load what one advertises

`multi_tool` is the exception and only because litany fans it out itself; each
of its sub-invocations then meets the same wall.

## Proven, not inferred

A probe conversation was told to call four of them once each in one parallel
batch and report what came back. Result: `4 invocations: 0 ok, 4 failed, 0
skipped` — `load_skill`, `dispatch`, `message` and `cd`, each the sentence
above. A separate conversation's opening step had already taken the same
refusal for `bash` and `read_file`. That is six of the seven demonstrated; the
seventh, `apply_patch`, takes the same path by construction — the router has one
else-branch.

## What is actually gone

This is not eight wasted declarations. It is four capabilities:

- **Subagents.** `dispatch` is the only way a conversation spawns a child. VISION
  V1's two-edge taxonomy and the child card have nothing to hang on.
- **Inter-agent messaging.** `message` is how one conversation reaches another.
- **The skills corpus.** `load_skill` is its only door. The world ships a
  populated `litany/skills/` that no agent can open.
- **The conversation's own worktree.** `bash`, `cd`, `read_file` and
  `apply_patch` were the only acts on it. An agent cannot read the `goal.md`
  sitting beside it, cannot write a file into its own branch, and therefore
  produces nothing a `/files`, a `/work-diff` or a delivery would ever see. The
  first observed conversation tried to read its own `goal.md` in its opening
  step and was refused.

  Everything an agent now builds is built on a foot, in the foot's scratch
  directory, on the foot's box. The workspace repo records the transcript of it
  and none of the artifact.

## Two questions this ball is asking, not answering

1. **Is the grant the bug, or is it the seam?** Narrowing the grant to
   `[multi_tool]` stops the decoys and costs one wasted step per conversation —
   but it does not give `dispatch`, `message` or `load_skill` back, and those
   are not machine acts. By REMOTE §5.4's own subject-locality test ("a tool
   executes where its subject lives") all three have the conversation as their
   subject, the conversation lives on the server, and they belong beside
   `write_summary` / `mark_for_deletion` as engine acts. `src/tool_host/
   engine_act.rs` says the name set is closed at two "because the procedure is
   the only second source of injected definitions litany has" — that is a fact
   about where a DEFINITION comes from, not about where an ACT belongs, and
   these three arrive as role grants rather than as procedure injections.

2. **Is a conversation meant to have a worktree at all any more?** If every
   artifact lives on a foot, then `/files`, `/work-diff`, `/science`, the
   candidate delivery flow and the fan/deliver family are all reading a tree
   nothing writes. That is a §12 migration-order question and it is bigger than
   this ball, but the answer changes what the fix here should be.

Whatever the ruling, the state on the tree today is the one shape that is
certainly wrong: eight tools declared, seven of them incapable, and no config
edit anywhere that says so.