+++
title = "the total router left the shipped worker grant dead: seven of eight granted tools refuse, taking subagents, agent messaging, skills and the conversation's own worktree with them"
created = 1788150349
updated = 1788151382
claimant = "OrderArbiter"
priority = 9
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["remote", "tool-host"]
+++
REMOTE §5.4's inversion made the router total: `ToolInjection::call` answers the
`clients` tool, the two engine acts, every loaded remote name, and renders
`UNLOADED` for everything else (`src/tool_host.rs`). Nothing resolves a binary
behind it any more. The shipped worker grant was not moved with it: seven of
its eight names refused in band — subagents, inter-agent messaging, the skills
corpus and every act on the conversation's own worktree, all dead, with the
artifacts of every build landing in a foot's scratch and `work-diff` empty.
(The original evidence — the probe conversation's 4/4 refusals, the control
that proved the mechanism — is in this ball's journal history.)

## The ruling, as landed

**The grant splits by subject** (REMOTE §5's subject-locality invariant, asked
name by name), and each half gets its lane:

1. **Engine acts** — `dispatch`, `message`, `load_skill`, `cd` join the
   compactor pair in `engine_act::NAMES` (two rows to six): each one's subject
   is the conversation or the world (a fork on the workspace, an inbox
   deposit, a server-disk skill copy, a mark ref), so each is performed at the
   engine's own front door with the caller identity on the env — bl-dfce's
   mechanism unchanged, now at the caller's resolved cwd (litany bl-ddaa).
2. **The worktree lane** — `bash`, `read_file`, `apply_patch` (and any
   granted pool name): subject is the conversation's working tree, which
   lives on the server's box, so the subject chose the executing machine. The
   router resolves the bare name against the workspace roster: the ONE
   registered client that both advertises it and consents to workspace-cwd
   execution (`"subject_cwd": true` on that entry in its own tools.json —
   advertised, because the engine routes on it) runs it, and the invocation
   carries the conversation's resolved cwd (`RoutedCall::cwd`, litany
   bl-ddaa — REMOTE §5's "the half the thrall move owed", landed). Zero
   consenting machines refuses in band naming both remedies; two is a config
   ambiguity refused naming them. The consenting box is §5.4's co-located
   thrall, the normal install.
3. **`multi_tool`** — untouched: litany's step loop fans it out before any
   router; each inner name is judged on its own subject.

Wire: the advertised element gained optional `subject_cwd`, invoke/invocations
gained optional `cwd`; the corpus ledger flagged exactly those four shapes and
PROTOCOL moved 1 -> 2, with thrall in lockstep (thrall bl-36f7).

## Rejected

- **Narrowing the grant to `[multi_tool]`** — stops the decoys, returns none
  of the four capabilities.
- **Answering bash/read_file/apply_patch as engine acts** — the server
  executing machine work in its own process is the second pipeline §12's
  front-door invariant exists to exclude; the compactor carve-out does not
  stretch to filesystem acts.
- **yog re-deriving the conversation's cwd** from litany's mark ref — a
  second home for litany's own fact; the seam hands the resolved value over
  instead.
- **A per-name "worktree tools" roster in yog** — the lane is the router's
  general else-branch, so an operator-granted pool name routes with no list
  to drift.
- **Ambient consent** (routing bare names to any advertiser) — a box must opt
  in before executing at a caller-named path; the containment-honesty clause
  stays true because the stating box is the enforcing box.

Question 2 in the original body ("is a conversation meant to have a worktree
at all any more?") is answered YES by the lane: the worktree stays the work
product's home, written through a consenting thrall that holds it, and
`/files` / `/work-diff` / delivery keep their subject.