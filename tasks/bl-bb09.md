+++
title = "AGENTS.md rule 6 records a lernie pin two releases stale: '=0.0.3' where Cargo.toml has '=0.0.6'"
created = 1786513158
updated = 1786513158
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["docs"]
+++
Found while verifying bl-6654's premises (Girder, 2026-08-11).

AGENTS.md rule 6 ("Dependencies are pre-approved + cargo-deny") states, verbatim at line 84:

    since bl-89a4 all three embedded substrate crates are plain crates.io pins
    (`balls = "=0.5.9"`, `brazen = "=0.0.5"`, `lernie = "=0.0.3"`), `deny.toml`
    has no `allow-git` list, and `make publish` works.

Two of the three are current. The lernie figure is not: `Cargo.toml` pins `lernie = "=0.0.6"` (lockfile checksum a2428cb2656cde4b3f03229e4637db5ab6a470c4792af252e786c789e7d493d0), and its own comment block in Cargo.toml narrates the 0.0.4/0.0.5/0.0.6 adoption in detail. docs/DESIGN.md §3.3 likewise says "Pinned lernie 0.0.6" and VISION §4.10 says "pinned 0.0.6" — AGENTS.md is the only surface still saying 0.0.3.

This matters beyond tidiness: rule 6 is the pin authority an implementer consults before touching a substrate version, and a ball whose first act is a pin bump (bl-6654, bl-cd38) reads it to learn the current floor. A stale floor invites a 'bump' that is actually a downgrade.

Fix: correct the one figure to `lernie = "=0.0.6"`. Re-check the other two against Cargo.toml at edit time rather than trusting this body — balls = "=0.5.9" and brazen = "=0.0.5" matched HEAD on 2026-08-11, but the point of this ball is that such figures rot.

Consider also whether restating exact versions in AGENTS.md is worth its drift cost at all, given Cargo.toml is the single source of truth and already carries the justification comments: the rule could name the pin *discipline* (exact registry pins, no path deps, no git deps in force) and cite Cargo.toml for the numbers.