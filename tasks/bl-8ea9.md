+++
title = "a native-binary box has no CD: the deployment seats only the container shape, so a laptop engine is hand-launched and never upgrades"
created = 1788673583
updated = 1788673591
claimant = "Cantaloups-H"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
The deployment in `scripts/deploy/` seats exactly one shape: `seat.sh` copies
`yog.service`, which runs `docker run ghcr.io/mudbungie/yog:<tag>`, and
`reconcile.sh` reconciles that box against the ghcr tag list. A box that runs
the engine as a NATIVE binary — a laptop, or any box with no container engine —
has no unit, no timer and no upgrade path at all. Observed on a live box: the
engine was a hand-launched `setsid nohup yog` from a shell three days old,
running 0.0.10 while 0.0.39 was published, with no systemd user unit of any
kind and `Linger=no`.

Operator ruling 2026-09-05: every device should run effectively full CD — any
new publication should result in an upgrade of the running versions.

The prior art is in this repo history. bl-bf35 shipped exactly this — a binary
`yog.service` plus `yog-update`, a crates.io sparse-index reconciler with a
pure `decide()` and a `--self-test` — and bl-c6e2 DELETED it when the server
cut over to the image, because one seating path cannot seat two shapes. The
deletion was right for the server and wrong for the laptop: it removed the only
shape a box without a container engine can run.

Restore the native shape as a second seating beside the container one:

  * `scripts/deploy/yog-local.service` — the binary unit, `ExecStart=%h/.local/bin/yog`,
    `Restart=always` with a StartLimit, seated AS `yog.service` (a box runs one
    engine over one world, so the unit NAME must stay single).
  * `scripts/deploy/local-reconcile.sh` — read the newest live version from the
    crates.io sparse index (yank-filtered, so a yank is the rollback lever),
    `cargo install --root $HOME/.local --locked --version <v> --force`, then
    restart the unit ONLY when the ss8.5 boundary says idle: `yog gesture
    {"op":"workspaces"}` with no `running:true` and no `stale`. Same deferral
    and same `reset-failed`-before-restart as `reconcile.sh`. A `failed` unit is
    its own arm: it has no turn to protect and restarting it onto a version it
    has not run is the one useful act.
  * `scripts/deploy/local.sh` + `make deploy-local` — seat it on THIS box, no
    ssh, enable lingering, and end by running the reconciler once synchronously
    so the seating either lands the newest release or says why.
  * a self-test in the gate, both directions, driving the real script under
    fake `curl`/`cargo`/`systemctl` — the shape `lernie` `scripts/deploy/update-selftest.sh`
    and the retired `yog-update --self-test` both hold.