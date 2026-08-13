+++
title = "adopt the bl-24e7 speculative merge queue with builds on GitHub Actions: gate conformance to the verdict-cache fingerprint + speculation/** remote builder"
created = 1786601138
updated = 1786601138
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
balls landed the speculative merge queue (design: balls docs/design/bl-24e7-speculative-merge-queue.md; shipped in balls 0.5.10 as the bl-speculate binary: verdict cache bl-1263, merging queue bl-5c5f, speculator bl-d0c2, GH Actions remote builder bl-6312). yog should ride it, with the gate builds happening on GitHub Actions rather than local machines.

Two facts shape the work:

1. The verdict-cache gate fingerprint hashes a FIXED file list (balls src/speculate.rs GATE_FILES): scripts/pre-commit, scripts/check-line-lengths.sh, scripts/check-coverage.sh, Makefile — all must exist or bl-speculate check/record error out and the cache is permanently inert (fail-open, so invisible). yog's gate lives in .githooks/pre-commit today; it must move to scripts/pre-commit, with the hook keeping only the mainline-commit guard and delegating.

2. The remote builder is pure gate policy: bl-speculate run --gate CMD runs CMD inside a detached checkout of each candidate commit. A CMD that pushes HEAD to a speculation/<sha> branch, waits for the speculate.yml workflow run, downloads the verdicts artifact and imports it (bl-speculate import), then answers with bl-speculate check, IS the GH Actions builder — no balls change needed.

Deliverables:
- scripts/pre-commit: the gate proper (verdict-cache check first, exit 0 on hit; then make fmt-check, make lint, held-output coverage; bl-speculate record pass at the end; both cache touches fail-open).
- scripts/check-coverage.sh: the held-output tarpaulin step, moved out of the hook (invocation stays make coverage — one home).
- scripts/check-line-lengths.sh: delegates to make line-cap (exists chiefly to be fingerprinted; say so in it).
- .githooks/pre-commit: branch guard + exec scripts/pre-commit.
- .github/workflows/speculate.yml: on push to speculation/**; toolchain resolves via rust-toolchain.toml (1.95.0 — the fingerprint's toolchain half must match local); install pinned tarpaulin 0.35.2 / cargo-deny 0.20.2 / ast-grep 0.44.1 and bl-speculate (cargo install balls --version 0.5.10 --bin bl-speculate, cached); run BALLS_IDENTITY=github-actions scripts/pre-commit; bl-speculate record fail on failure; upload ~/.local/state/balls/plugins/bl-speculate/verdicts/ as artifact 'verdicts' (always).
- scripts/speculate-gate: the remote-build driver described above (push, gh run watch, gh run download, import, sweep the branch, exit by local check).
- AGENTS.md: short merge-queue section — the agent flow is: seal with bl-speculate enqueue <id>; bl-speculate run --gate scripts/speculate-gate (builds land remotely); bl close hits the cache; any miss degrades to the stock local gate.

yog is a PRIVATE repo: Actions minutes are metered, a stated and accepted cost. Note it in the workflow header.