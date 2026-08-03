+++
title = "agent list at the top shows a raw timestamp id instead of the name — every agent-naming seat rides the display ladder"
created = 1785731750
updated = 1785731758
claimant = "seat-namer"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-03, verbatim: 'the agent list at the top is named just some incoherent timestamp.'

The 'incoherent timestamp' is almost certainly a raw agent id (lernie ids are timestamp-prefixed, e.g. 20260803T...Z-<hash>). Some surface at the top of the shell — candidates: the members strip (src/shell/members.rs), the descent-tree member row, the center header, a workspace/agent list — is showing the id instead of the display name.

bl-08f2 (commit 316fef8) just wired Agent::name_fact() — rung one the lernie-stored name, then the legacy 'You are <x>.' goal parse, then first payload line — into 'all seats' (row title, subtitle, center header via display_name_of, in-flight strip name_of, deletion gate, focus). So this is one of: (a) a seat bl-08f2 missed that still formats the id, (b) a fallthrough — the agent in question has no name fact, no stamp, and no payload first line worth showing (the energize runaway children are prime suspects: lernie-dispatched, possibly nameless, goals like 'create a new subagent'), and the ladder's floor is the raw id, or (c) the operator's running binary predates 316fef8 (check ~/.local/bin/yog mtime and the close.post install log).

The work: reproduce against the operator's actual worlds (~/.local/share/yog/workspaces), identify the exact surface and which case holds. Fix accordingly: (a) point the seat at the ladder — no seat formats an id directly; (b) decide the honest floor for a nameless agent (an id is a fact — maybe the floor stays but shortened per whatever id-abbreviation idiom the codebase already has; do not mint fake names for display); (c) operational — record it and fix nothing beyond what prevents recurrence. Verify all paths against the tree; bl-6920 (stamp retirement) may be landing concurrently in the same territory.