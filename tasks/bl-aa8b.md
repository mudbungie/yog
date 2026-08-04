+++
title = "freeze project instructions into agent context with visible provenance"
created = 1785649815
updated = 1785823807
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

After bl-2b8c names the authoritative target, make project instructions a frozen, inspectable spawn input:

- filename set and precedence are yog policy/config, with a documented default that covers this repository's `AGENTS.md`;
- discovery is deterministic from authority root to target;
- exact bytes and source provenance are snapshotted before the first inference through lernie's generic pinned-document mechanism;
- the operator sees what was included; the user's goal remains their payload, not a hidden concatenation;
- child inheritance follows the project-work ruling;
- size is bounded and unreadable files, traversal, external includes, and untrusted parent instructions have explicit fail/skip policy;
- no opaque automatic memory or yog-side duplicate store.

Upstream mechanism: the lernie pinned-documents ball filed by the same audit. Consumer work waits for its published release and an exact yog pin.