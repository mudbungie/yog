+++
title = "pins → brazen =0.0.19 + litany =0.0.14: the catalog read goes live in-process (bare bz --login lists the sign-in catalog; providers is the routing table)"
created = 1792729607
updated = 1792730268
claimant = "Gelded"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["release"]
+++
## Why
bl-c6e3 shipped the `catalog` op (edition 24) against brazen =0.0.18, where `bz --login --json` with no --provider exits 78 and the catalog is empty. brazen 0.0.19 (bl-6feb) is on crates.io; litany bumps to =0.0.19 in litany bl-cdcb → litany 0.0.14. yog's lockfile must resolve exactly ONE brazen, so both pins step together: `brazen = { version = "=0.0.19", features = ["native-host"] }` and `litany = "=0.0.14"` (Cargo.toml ~143 and ~352, with the running comment history each pin carries).

## Done when
Both pins moved, lockfile resolves one brazen, `make check` green (the effects test for `catalog` now sees the real oauth preset), landed on main.