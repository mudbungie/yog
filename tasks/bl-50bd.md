+++
title = "W11 embed lernie: the driver is yog"
created = 1784784299
updated = 1785124290
claimant = "waxier-50bd"
parent = "bl-b5d1"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
DESIGN §16.7 W11. Needs W12 + U-lernie (lernie repo: adapter_target via Fx + crates.io publish). Multiplex arm reproduces lernie's thin bin (preludes, Fx with driver_target=adapter_target=self, Outcome::Exec); all yog→lernie spawns retarget.

## STATUS (bl-50bd, waxier-50bd)

- Ruling applied: no crates.io wait; shim off the local lernie build via exact git-rev pin.
- U-lernie inspection: /home/u/dev/lernie HEAD 39c10a5 == origin/main (github.com/mudbungie/lernie), clean tree. `Fx::adapter_target: Option<PathBuf>` IS landed upstream (cmd/mod.rs:156, resolution order in prompt/resolve.rs: models.yaml adapter: > Fx host > bz on PATH). NO upstream work needed. Publish is lernie's own bl-9253 (claimed); yog registry adoption queued as bl-89a4 — neither is this task.
- Pin: lernie = { version = "=0.0.1", git = ..., rev = 39c10a5f2e41d4544016ee78e88d064f6cc8a956 } (balls-pattern comment). brazen parity: lernie pins =0.0.4, yog pins =0.0.4 — one brazen in the graph.
- Key as-built decision: Fx driver_target/adapter_target are single PathBufs spawned verbatim (Command::new(target) args...), so a bare yog exe CANNOT carry the namespace word. Targets are the world's `lernie`/`bz` shims (W9 ensure_shim pattern, generated from Cli::exec_words), seeded at Step::EnsureSeeded beside bl's. DESIGN W11/W9 notes amended to match.

### Plan / progress
- [ ] Cargo.toml lernie git pin + deny.toml allow-git
- [ ] src/multiplex/lernie.rs: arm = thin bin (parse cmd::Cli, preludes, Fx, conclude/perform incl Outcome::Exec); multiplex.rs stub + LERNIE_UNEMBEDDED(90) deleted
- [ ] world/tools.rs: LERNIE + BZ shim consts; start/run.rs EnsureSeeded seeds both (Deps grows bz Cli)
- [ ] resolve.rs: Binary::Lernie self_multiplexed -> true (all spawn sites retarget via the one switch; sites: main.rs x2 Cli::resolve_in_world)
- [ ] tests: multiplex/tests.rs rework; new tests/multiplex_lernie.rs (own process, unsafe set_var LERNIE_HOME/EDITOR — rules-audit scans src only) driving prime/new/config/dispatch through dispatch()
- [ ] DESIGN §16.5 + §16.7 U-lernie/W9/W11 as-built amendments
- [ ] make check green; close (mention bl-89a4 adoption queued)