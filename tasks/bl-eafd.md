+++
title = "B7: AGENTS.md + hook/CI ast-grep wiring + DESIGN amendment + fixtures smoke target"
created = 1784433625
updated = 1784436935
claimant = "filtered"
parent = "bl-97fb"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-a47a"
on = "claim"
+++
The closing ball. (1) AGENTS.md at repo root: the bootstrap doc's agent-rules template adapted to yog — flat numbered rules; record the adaptations verbatim so future agents know the deviations and why: unsafe confined to cli_outbound/sys.rs by ast-grep (not forbid) because one irreducible SIGTERM syscall; locks in state.rs + test_support carve; no workspace split (single published binary crate); no musl (GL app); no async/tokio today — rules 8 installed and vacuous; no anyhow (thin main); pub boundary = the real lib surface, internal APIs are pub(crate); plus the repo's own discipline: bl claim->close worktree flow, 300-line cap, 100% tarpaulin, DESIGN.md is the architecture authority, never credit AI. (2) .githooks/pre-commit: add `ast-grep scan` (after fmt/clippy, before tarpaulin — fail fast) + a fixtures smoke-check (ast-grep scan rules/fixtures must FAIL — i.e. expect nonzero — proving the rules still bite; wire as a small guard in the hook or a make target `make rules-check` the hook calls). (3) CI linux job: cached pinned ast-grep install + `ast-grep scan` + the rules-check. Record tool pins (ast-grep, cargo-deny versions from `--version`) in the workflow and Makefile comments. (4) Makefile: `lint` or `check` grows ast-grep scan + cargo deny check so `make check` = the full local gate. (5) docs/DESIGN.md: a short new section (or §12 note) recording that Bootstrap v3 governs code style, pointing at AGENTS.md and rules/, and listing the surfaced skips. README one-liner under Building. Gate green + CI fully green on close.