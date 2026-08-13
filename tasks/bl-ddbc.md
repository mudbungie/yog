+++
title = "cargo deny fails on a webbrowser advisory, so the pre-commit hook is red and bl close aborts for every ball in the repo"
created = 1786600116
updated = 1786600123
claimant = "Pintle"
priority = 5
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
`cargo deny check advisories` fails on a clean tree at main `2229182`, so the
`pre-commit` hook fails, so **`bl close` aborts for every ball in this repo**.
Found 2026-08-12 by a worker standing down from an unrelated ball; reproduced
on a clean working tree before filing.

    advisories FAILED

    ├ Announcement: https://github.com/amodm/webbrowser-rs/security/advisories/GHSA-2ph8-5cr8-hr33
    ├ Solution: Upgrade to >=1.2.2 (try `cargo update -p webbrowser`)
    ├ webbrowser v1.2.1
      └── egui-winit v0.29.1
          ├── eframe v0.29.1
          │   └── yog v0.0.1
          └── yog v0.0.1 (*)

The advisory, in the registry's own words: argument injection via the `BROWSER`
env template — the URL is substituted into the template without tokenizing it
first, so on a template containing `%s` a crafted URL becomes additional
browser arguments.

    Version 1.2.2 fixes the issue by tokenizing the `BROWSER` template before
    substituting the URL, preserving the URL as part of a single argument. Users
    should upgrade to version 1.2.2 or later. Applications that only need HTTP(S)
    URLs can also enable the crate's `hardened` feature as defense in depth.

## Why this is urgent beyond the advisory itself

Nothing lands while it stands. The hook runs `make lint` verbatim and `make
lint` runs `cargo deny check`, so this is not a per-ball problem — it is a
repo-wide delivery stop. Balls in flight will each discover it at their own
close gate and burn a full context on it.

## The fix

A **lockfile bump only** — no `Cargo.toml` edit. `cargo update -p webbrowser`
moves 1.2.1 → 1.2.4. Verify `cargo deny check` then reports all four sections
ok, and that the tree still builds and tests green.

## Decide, don't just bump

- yog reaches `webbrowser` only transitively, through `egui-winit`/`eframe`.
  Confirm whether yog opens a browser at all. If it does, that path is the one
  the advisory describes and it deserves a look, not just a version number.
  If it does not, say so — a transitive advisory yog cannot reach is worth
  recording as such so the next person does not re-investigate.
- The crate ships a `hardened` feature named in the advisory as defence in
  depth. Rule on whether yog wants it; if not, say why in one line.
- yog pins its own substrate deps exactly (`lernie = "=0.0.6"`). Check whether
  a transitive advisory of this shape can recur silently, and whether anything
  cheap prevents it. Do not add mechanism for its own sake — a stated principle
  is an acceptable answer.