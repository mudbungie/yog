+++
title = "REGRESSION of bl-0ec2: subcommand help executes the command or mutates the world"
created = 1786510240
updated = 1786513023
claimant = "Cinder"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["regression", "help"]
+++
bl-0ec2 required help to be higher-order and world-free: every command must answer `--help` in place. On `96d5f4e`, only some namespaces do.

Against fresh scratch `XDG_DATA_HOME` roots:

- `yog env --help` exits 0 but prints export lines, not help.
- `yog exec --help` exits 127 after trying to spawn a program literally named `--help`.
- `yog headless --help` starts the engine and remains parked; `timeout 2` exits 124 with no help.
- `yog bz --help` exits 64 with `no workspace in this environment`.
- `yog bl --help` prints help only after `world::tools::ensure_tools`; it created six shims under the fresh world. On a read-only root it fails before help.
- `yog lernie --help` is the working control: exit 0, help text, no world requirement.

Top-level help says `Every command answers --help`, and README says help reads the interface rather than the world. Intercept help before env, exec, headless, bz, and bl do any composition, spawn, wait, or write. Add per-namespace zero-side-effect tests.