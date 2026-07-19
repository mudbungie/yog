+++
title = "B1: toolchain pin + deny.toml + CI additions (cargo-deny, doctests)"
created = 1784433623
updated = 1784433634
claimant = "filtered"
parent = "bl-97fb"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
No code changes. (1) rust-toolchain.toml at repo root: channel = the current stable (rustc --version, expect 1.95.0), components clippy+rustfmt. NO musl target (GL desktop app — surfaced skip). (2) deny.toml from the empirical license audit: [advisories] yanked=deny; [licenses] allow = [MIT, Apache-2.0, "Apache-2.0 WITH LLVM-exception", Unicode-3.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, 0BSD, BSL-1.0, CC0-1.0, Unlicense, OFL-1.1] plus a per-crate exception for epaint_default_fonts permitting LicenseRef-UFL-1.0 (non-SPDX Ubuntu Font License); [bans] wildcards=deny, multiple-versions=warn, deny openssl-sys + native-tls; [sources] unknown-registry=deny, unknown-git=deny. Run `cargo deny check` and iterate the config until it passes (the slash-form license expressions like MIT/Apache-2.0 are the version-sensitive risk — fix by config, never by patching deps). Record the installed cargo-deny version in a comment. (3) .github/workflows/ci.yml linux job: add cached pinned cargo-deny install + `cargo deny check` + `cargo test --doc --workspace` steps. Do NOT add ast-grep here yet (B7 wires it once all rules exist). Verify CI passes post-close.