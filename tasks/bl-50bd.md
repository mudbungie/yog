+++
title = "W11 embed lernie: the driver is yog"
created = 1784784299
updated = 1785124472
claimant = "waxier-50bd"
parent = "bl-b5d1"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
DESIGN §16.7 W11. Needs W12 + U-lernie (lernie repo: adapter_target via Fx + crates.io publish). Multiplex arm reproduces lernie's thin bin (preludes, Fx with driver_target=adapter_target=self, Outcome::Exec); all yog→lernie spawns retarget.

## STATUS (bl-50bd, waxier-50bd)

- U-lernie: NO upstream work needed. /home/u/dev/lernie HEAD contains Fx::adapter_target; pinned rev 39c10a5f2e41d4544016ee78e88d064f6cc8a956 (== the rev inspected; origin/main has since moved to 0a6a36a with only release chores + a coverage fix). lernie is PRIVATE on GitHub — cargo could not fetch anonymously; fixed by `gh auth setup-git` (gh credential helper now feeds git https auth). crates.io holds only a 0.0.0 name reservation; 0.0.1 publish is lernie's own bl-9253 (claimed upstream); yog registry adoption queued as bl-89a4 (NOT this task).
- brazen parity holds: lernie pins =0.0.4, yog pins =0.0.4, one brazen in Cargo.lock.

### Done
- [x] Cargo.toml lernie git pin (balls-pattern comment) + deny.toml allow-git grows lernie
- [x] src/multiplex/lernie.rs: the arm = lernie's thin exec binding (parse cmd::Cli via try_parse_from, preludes fold, Fx over locked stdio, conclude/perform incl. Outcome::Exec via CommandExt::exec). LERNIE_UNEMBEDDED(90) + unembedded() deleted from multiplex.rs.
- [x] As-built ruling: Fx targets are the world/tools/{lernie,bz} shims (ensure_shim, W9 mechanism), converged by the arm on the way into every verb — a bare yog exe cannot carry the namespace word (Fx targets are single PathBufs spawned verbatim). Byproduct: world PATH now carries all three tools.
- [x] world/tools.rs: LERNIE + BZ consts
- [x] resolve.rs: Binary::Lernie self_multiplexed -> true (all spawn sites retarget through the one switch)
- [x] toolgate: left as-is for lernie (W10 bz precedent — probes now answer from the embedded clap surface, always 0; bl alone needed emptying because its arm refuses verbs)
- [x] tests: multiplex/tests.rs reworked; src/multiplex/lernie/tests.rs (parse/edit_with/conclude/perform); tests/multiplex_lernie.rs end-to-end (own process env: prime/new/config editor both ways/dispatch prelude) — PASSES
- [x] DESIGN §16.5 lernie bullet, §16.7 U-lernie, W9 shim note (superseded), W11 Landed + as-built note
- [x] cargo test full suite: 741+ pass, 0 fail; fmt, clippy -D warnings, rules-audit, cargo deny all green
### Remaining
- [ ] make coverage (tarpaulin 100%) — running
- [ ] bl close (mention bl-89a4 adoption queued)