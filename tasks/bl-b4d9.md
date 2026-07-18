+++
title = "Y1: Split cli_outbound/tests.rs into submodule files"
created = 1784349553
updated = 1784349553
parent = "bl-4e66"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
DESIGN.md §15 Y1. src/cli_outbound/tests.rs is at 274/300 and every subsequent cli_outbound extension adds tests there. Split it into src/cli_outbound/tests/{run,stream,spawn}.rs (+ tiny mod.rs), preserving every test verbatim and the SPAWN_LOCK/ENV_LOCK usage. Pure refactor, zero behavior change, green gate. Files: src/cli_outbound/tests/* (<=250 each).