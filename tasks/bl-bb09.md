+++
title = "AGENTS.md rule 6 records a lernie pin two releases stale: '=0.0.3' where Cargo.toml has '=0.0.6'"
created = 1786513158
updated = 1786683124
claimant = "Dredge"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
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

---

## The precedent to follow (Hessian, bl-4a22 — via Alkaloid)

This ball is one instance of a general disease: **a doc restating a fact another file owns will drift.** AGENTS.md rule 6 carrying `lernie = \"=0.0.3\"` while Cargo.toml says `=0.0.6` is the same failure as the four bl-4a22 found in README/STORIES:

- README: 'lernie is still spawned as a binary; embedding it is the next wave (W11)' — false; `src/cli_outbound/resolve.rs` has `self_multiplexed` ON for every namespace.
- README documented the W9 `prime`/`sync`/`install` refusal — deleted by bl-2930.
- README + STORIES S8 called brazen config/credentials/cache ambient-shared — DESIGN §16.2 puts them in the per-workspace wall.
- `scripts/drive/beats_s7.sh` carried a stale `=0.0.3` pin comment.

Each was true when written. That is the point: restating is the defect, not carelessness.

**The fix bl-4a22 landed, and the one to copy here:** it replaced README's enumerated gesture roster with a **pointer to `yog gesture --help`** — the roster cannot drift because it is no longer stored twice. It also rewrote the beats_s7.sh comment to 'the pin authority is Cargo.toml' rather than restating a number.

So for this ball: **do not just correct =0.0.3 to =0.0.6.** That re-arms the same trap for the next bump. Name the pin discipline in rule 6 and cite Cargo.toml as the authority for the numbers, exactly as the body already proposes. A rule that says 'the pins are exact, lockfile-fixed, and live in Cargo.toml' stays true forever; a rule that lists them is stale on the next publish.

Worth a sweep while in there: any other doc line that enumerates something a file owns (module lists, gesture names, provider rows, version numbers) is a candidate for the same treatment.
