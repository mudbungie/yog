+++
title = "freeze project instructions into agent context with visible provenance"
created = 1785649815
updated = 1785823883
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design"]

[[blockers]]
id = "bl-2b8c"
on = "claim"

[[blockers]]
id = "bl-6654"
on = "claim"
+++
Source: bl-e249's Claude Code comparison at snapshot 06d29efd02547a586a33cab60e8acf3dba2997e8.

## Verified gap

Claude Code deterministically discovers project instruction files and rules from the project hierarchy before work. Yog and pinned lernie discover neither this suite's `AGENTS.md` hierarchy nor a generic configured instruction hierarchy. Lernie's shipped worker soul is only harness mechanics. Yog currently sends target location and task content in the editable goal, so repository standards depend on the model finding them itself.

Claude Code does NOT normally auto-load `AGENTS.md`; this is evidence for a project-context mechanism, not a request to hardcode Claude's filenames or copy its memory system.

## Deliverable

bl-2b8c ruled (VISION §4.10): the authoritative target is the bound attempt worktree, passed typed at fire by bl-6654 (this task's claim blocker). Make project instructions a frozen, inspectable spawn input:

- filename set and precedence are yog policy/config, with a documented default that covers this repository's `AGENTS.md`;
- discovery is deterministic from authority root to the typed target;
- exact bytes and source provenance are snapshotted before the first inference through lernie's generic pinned-document mechanism — **released and pinned** (bl-fb5c, lernie 0.0.4; yog pins 0.0.6), so no upstream wait remains beyond bl-6654;
- the operator sees what was included; the user's goal remains their payload, not a hidden concatenation;
- child inheritance follows VISION §4.10 (a write-capable child's instructions re-freeze against its own attempt, never inherit stale bytes);
- size is bounded and unreadable files, traversal, external includes, and untrusted parent instructions have explicit fail/skip policy;
- no opaque automatic memory or yog-side duplicate store.