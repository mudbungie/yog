+++
title = "re-baseline the ladder: drive every story script and the clean room on current main; one log, one ball per red"
created = 1786161490
updated = 1786162426
claimant = "Ladderwright"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Premise (verify against the tree first — ball bodies drift): the newest docs/drive-logs/ entry is 2026-07-27; ~177 commits have landed since, including the harness's own move off pixels (bl-e458) and major surface rework (bl-1802 spine-in-chat, bl-9dd4 board, bl-f6fe headless, bl-66fb fleet, S15-S17 capability rungs). Nothing enforces verdict currency (docs/QUALITY.md §4), so 'main works' is currently an eleven-day-old claim about a surface that no longer exists.

Task — triage only, fix nothing here:
1. Release-build current main; drive it in an isolated scratch world (XDG_DATA_HOME override) on a claimed seat (scripts/drive/yogdrive.sh seat — never a hardcoded display, never the live world).
2. Run the full set: stories.sh run (S0/S1, live wire), run-s3s4s6, run-s5s8 (zero wire), run-s7, and cleanroom.sh (the W14 batteries proof). Before blaming yog for a red, verify the wire and the host tool tuple — a red drive is a claim about the machine and the substrate as much as about the code (STORIES).
3. Record ONE log in docs/drive-logs/ in the house style of the 2026-07-26/27 logs: build sha, host tuple, load average, per-beat PASS/FAIL with the on-disk evidence quoted.
4. File one ball per red, citing the story test id (Sn-Tn) and the beat's evidence. Search bl list <needle> --all first; a recurrence of a closed defect is filed as a regression naming it.
5. File one coverage-debt ball naming the graduated rungs (S10+) that have in-crate tests but no drive beats.

Acceptance: the log exists on main naming current main's sha; every red has a filed ball; zero fixes in this worktree beyond the log.