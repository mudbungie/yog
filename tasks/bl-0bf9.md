+++
title = "activity list: a prompt op's full goal text renders inline, breaking the one-row-per-op scan"
created = 1785646893
updated = 1785646893
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
The expanded activity accessory renders each op as one line — except a prompt op, whose full multi-line goal (identity preamble + payload, arbitrarily long with ball worktree preambles) flows into the list unelided and wraps across lines. The list is a scan surface; verbatim belongs in the row's expansion (argv/cwd/exit/stderr, §4.2). Fix: clamp every collapsed row to one line with middle/tail elision; expansion keeps bytes verbatim. Acceptance: a goal of 500 chars renders as one elided list row; expanding yields the verbatim argv.