+++
title = "nothing in the gate stops a credential, a routable IP, or a live session transcript from being committed"
created = 1786602278
updated = 1786602443
claimant = "Abbrevs"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
yog is a published crate whose whole job is driving real agent sessions on a
real box: transcripts, brazen credentials, world paths, drive-log evidence. The
gate (`scripts/pre-commit` -> `make fmt-check`, `make lint`, coverage) checks
style, structure, supply chain and coverage. It checks NOTHING about content
disclosure. A pasted `.env`, an `sk-ant-…` in a fixture, a home-rooted path, or
a dumped Claude Code session JSONL commits clean today.

## Baseline (audited 2026-08-12, 811 tracked files)

Clean: zero vendor tokens, zero PEM private keys, zero credential assignments,
zero routable IPv4/IPv6/MAC, zero real vendor resource ids. The only IPv4 in
the tree is `127.0.0.1` in a test fixture.

Not clean: absolute paths under the operator's OWN home (`/home/u/...`),
70 occurrences — 62 in `docs/drive-logs/` (evidence records of real runs) and
8 illustrative lines in `docs/DESIGN.md`, `src/binding/tests.rs`,
`src/opslog/tests.rs`, `src/opslog/line/tests.rs`. Synthetic home roots
(`/home/u`, `/home/op`, `/home/x`) are the house convention and are fine.

## Deliverable

`scripts/leak-scan.sh` — one scanner, whole tracked tree (`git ls-files`, the
INDEX, so a staged addition is covered), all offenders reported at once, same
shape as `make line-cap`. Wired into `make lint`, so the hook, `make check`
and CI get it from one edge and cannot drift.

Rules: PEM private keys; vendor tokens (anthropic/openai/github/aws/google/
slack/JWT); credential assignments with a non-placeholder value; routable IPv4
(loopback + RFC5737 doc ranges exempt), full-form IPv6, MAC; the SCANNING
operator's own \$HOME path (no hardcoded username — \$HOME is the one
authority, so the rule is self-updating and never itself names the operator);
session artifacts (long vendor resource ids, Claude Code transcript keys);
forbidden file shapes by path (*.pem, id_rsa, .env, credentials.json, .netrc,
.ssh/, .claude/).

Regression half — the point of the ball: `scripts/leak-fixtures/`, one file per
rule, and `--self-test` asserts EVERY rule fires on its own fixture. Stronger
than `rules-audit`'s fixtures check (which only asserts the directory is
flagged, so one dead rule among many hides). A silently-broken pattern is the
failure mode a leak gate actually dies of.

Also: `.githooks/commit-msg`, running the same scanner over the message file —
`pre-commit` never sees the commit message, and a secret pasted into one is
committed just the same.

## Open decisions for the operator

1. `docs/drive-logs/` is exempted from the home-path rule only (every other
   rule still applies there). Those files are append-only evidence of runs on a
   real box and the path IS the evidence. The alternative is redacting 62 lines
   across 7 log files. Say the word and the exemption goes — it is one line of
   config, not a code edit.
2. The 8 illustrative `/home/u` lines outside drive-logs are rewritten to
   `/home/u`, the dominant synthetic convention.

## Known gap

`bl-speculate`'s verdict-cache fingerprint is a FIXED file list compiled into
the binary — `Makefile`, `scripts/pre-commit`, `scripts/check-line-lengths.sh`,
`scripts/check-coverage.sh` (verified by `strings`). `scripts/leak-scan.sh` is
not in it, so editing the scanner does not invalidate stored verdicts the way
editing any other gate file does. The real fix is upstream in balls: read the
gate file list from the repo instead of compiling it in. Filed separately.

---

DELIVERED as designed, with two corrections the tree forced.

Landed: `scripts/leak-scan.sh` (8 content rules + `forbidden-path`),
`scripts/leak-fixtures/` (a fixture per rule, `clean.txt`, `clean-paths.txt`),
`make leak-scan` wired second in `make lint`, `.githooks/commit-msg`, and the
`## The local gate` section of AGENTS.md.

The tree scan found two false-positive classes the design did not predict, and
both were fixed in the RULE, not the fixture:

1. `src/config_edit/brazen/providers/tests.rs` carries the operator's real
   `bz --list-providers --json` table, in which `credential` is a STATUS field:
   `"credential":"not required"`. Fixed structurally rather than by
   allowlisting the four values — a multi-word ALL-ALPHABETIC value is prose,
   not a secret. A real credential carries a digit, a symbol, or no space.
2. `src/world/mod.rs:59` records an empirical `bl claim` worktree path, in
   which the operator's home appears MIRRORED mid-path
   (`.../bl-delivery/home/<user>/dev/yog/bl-c68f`). Correctly flagged: it names
   the operator just as a leading path does. Rewritten with the rest.

Nine illustrative lines rewritten to `/home/u` across `docs/DESIGN.md` (2),
`src/binding/tests.rs` (4), `src/opslog/tests.rs`, `src/opslog/line/tests.rs`,
`src/world/mod.rs`. `docs/drive-logs/` untouched and exempt from that rule
alone.

Proof the regression test can fail (mutation-tested, three ways):
  - broke ONE alternative inside the nine-way vendor-token pattern (AKIA) ->
    self-test named lines 7 and 8 of the fixture unflagged;
  - emptied ipv4-routable's EXCEPT list -> clean.txt flagged, "a rule is
    over-broad";
  - dropped `jsonl` from forbidden-path -> line 20 unflagged.
Restored, all green. The commit-msg hook was exercised both directions: a
message carrying a fabricated vendor token on a routable address exits 1
naming both, a clean one exits 0.

Gate: `make fmt-check`, `make lint` (line-cap 765 files, leak-scan 814 files
clean, clippy, rules-audit, cargo-deny) and `cargo test` (2050 pass) all green
on the merged tree. Scan cost ~5s.

Operator actions: run `make install-hooks` in the main checkout once, to seat
`.githooks/commit-msg`. The fixture tokens are fabricated but well-formed; if
GitHub push protection ever objects, allowlist them there.
