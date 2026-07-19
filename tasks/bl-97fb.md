+++
title = "epic: Rust Bootstrap v3 adoption — pinned toolchain, deny.toml, ast-grep rules, panic-free prod, owned signatures, pedantic gate"
created = 1784433623
updated = 1784433623
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Implement the user's "Rust Project Bootstrap v3" standard in yog, with surfaced adaptations. Assessment reports in the session scratchpad; adaptations recorded in AGENTS.md + DESIGN when B7 lands. Deliberate skips (surfaced to user): workspace/crates split (single published binary crate; module tree + 300-line cap already contain); musl static target (GL desktop app needs dynamic platform libs; TLS bans kept in deny.toml anyway); pre-commit framework + nextest + bacon/sccache/mold (one hook system — .githooks — carries the same checks; cargo test + tarpaulin remain the runners); anyhow (main.rs has no error plumbing; thiserror already in place). One irreducible unsafe (SIGTERM) is confined by an ast-grep location rule instead of unsafe_code=forbid (forbid is unoverridable and reaches tests; a nix/rustix dep or a crate split for ~10 lines is worse).