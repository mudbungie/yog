+++
title = "cargo deny fails on a webbrowser advisory, so the pre-commit hook is red and bl close aborts for every ball in the repo"
created = 1786600116
updated = 1786600169
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

---

## Rulings (Pintle, 2026-08-13)

Fix landed as a lockfile bump only: `cargo update -p webbrowser`, 1.2.1 -> 1.2.4
(it also dropped a now-unneeded `core-foundation v0.10.1`). No `Cargo.toml`
edit. `cargo deny check` afterward: **advisories ok, bans ok, licenses ok,
sources ok**. `make lint` green; `cargo test` green (1920 + 68 + 5 + 2 + six
single-test binaries, 0 failed).

**1. Does yog reach a browser-opening path? No — unreachable.**
`webbrowser` enters only through `egui-winit v0.29.1`, which calls it from
exactly one place (`egui-winit-0.29.1/src/lib.rs:999`):

    fn open_url_in_browser(_url: &str) {
        #[cfg(feature = "webbrowser")]
        if let Err(err) = webbrowser::open(_url) {

and that function has exactly one caller, in `handle_platform_output`
(`lib.rs:836`):

        if let Some(open_url) = open_url {
            open_url_in_browser(&open_url.url);

`PlatformOutput::open_url` is set only by egui's `Context::open_url` or a
`Hyperlink`/`Link` widget. yog has **no** call site for either — a grep of
`src/` for `open_url` / `OpenUrl` / `Hyperlink` / `.link(` returns nothing but
`theme/mod.rs:133` setting `hyperlink_color` (a palette value on a widget yog
never instantiates) and the word "hyperlink" in `names/words.txt`. So yog never
populates that field and `webbrowser::open` is dead code in this binary.

yog *does* have a browser sign-in flow, and it is a different mechanism:
`src/login/mod.rs` spawns `bz --login --provider <p> --browser` as a child
process. The browser is opened by brazen in its own process, not by yog's
linked `webbrowser`. `cargo tree -i webbrowser` confirms the only reverse edge
is egui-winit; brazen does not depend on the crate.

**2. Does yog want the `hardened` feature? No.**
It exists in 1.2.4 (`[features] hardened = []`) but yog cannot reach it without
adding `webbrowser` as a *direct* dependency purely for feature unification —
a new dependency, which AGENTS.md rule 6 forbids without explicit user
approval ("Zero new dependencies without explicit user approval"). Buying
defence-in-depth on a code path this binary cannot execute, at the price of a
new direct dep and a permanent line in `Cargo.toml`, is mechanism for its own
sake. Revisit only if yog ever grows a real `ctx.open_url` / `Hyperlink` call —
at that point the feature is worth the direct dep.

**3. Can a transitive advisory recur silently? No — it cannot recur silently,
and that is the whole design; no new mechanism is warranted.**
`cargo deny check` runs inside `make lint`, `make lint` runs inside the
`pre-commit` hook, and the hook gates every `bl close`. The failure mode here
was never silence — it was the opposite: maximum, repo-wide noise.

The stated principle, for the next person who meets a red `cargo deny` on a
tree they did not change: **an advisory is a time-varying fact about the
world, not a fact about the tree.** `cargo deny` refetches the advisory DB on
every run, so a lockfile that was green yesterday goes red today with no local
edit. A red `advisories` section on an unchanged tree is therefore an advisory
database advance, not your regression — do not diagnose your own diff. File a
lockfile-bump ball (this shape: `cargo update -p <crate>`, no manifest edit)
and land it first; every other ball in the repo is blocked behind it.

The exact-pin discipline (`lernie = "=0.0.6"`) is not weakened by this: yog
pins its own *direct* substrate deps exactly, while transitive versions are
lockfile-resolved and free to move under `cargo update -p`. That is what made
this a one-command fix rather than a manifest negotiation, and it should stay
that way.

No `deny.toml` entry was added — the advisory is *fixed*, not ignored, so the
`[advisories] ignore` list stays exactly as long as the tree justifies (that
file's own rule).
