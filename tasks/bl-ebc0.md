+++
title = "bump embedded lernie pin to =0.0.6"
created = 1785737508
updated = 1785737509
claimant = "lernie-bumper"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
lernie 0.0.6 is published on crates.io (0.0.5 + dispatch skill / worker soul rewrite, child terminal deposit fix). Bump Cargo.toml pin `lernie = "=0.0.5"` -> `=0.0.6`, refresh Cargo.lock, and fix any test assertions that track lernie's skill/soul text. brazen stays =0.0.5 — lernie 0.0.6 links `brazen =0.0.5`, the same exact version yog pins, so exactly ONE brazen still resolves (§16.7 parity).