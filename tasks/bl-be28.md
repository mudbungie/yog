+++
title = "Y15: ops.jsonl — durable action-outcome log"
created = 1784349560
updated = 1784349732
claimant = "filtered"
parent = "bl-4e66"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-a5f0"
on = "claim"
+++
DESIGN.md §15 Y15. O_APPEND writer producing one JSON line per completed CLI action {ts, argv, cwd, exit, stdout, stderr}, hard-capped at 4096 bytes (PIPE_BUF) with stdout/stderr truncation and a "truncated":true marker; tail parser (forgiving per line); fs-watched so both instances render the shared history; ops pane VM. Files: src/opslog/mod.rs (~150).