+++
title = "no yog image has published since 0.0.55: build.rs reads corpus/shapes.json (bl-1be7) and the Containerfile never COPYs it, so the ghcr image job fails on every release and noodlezoo's reconciler sits on 0.0.54"
created = 1789711784
updated = 1789711784
claimant = "Cruises"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Observed 2026-09-17: ghcr.io/mudbungie/yog lists tags up to 0.0.54; releases 0.0.55 through 0.0.58 exist on crates.io and GitHub but the *ghcr image* job of every release run since fails at `make image` with

    error: failed to run custom build command for yog
    Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }

bl-1be7 made `corpus/shapes.json` the third build input (build.rs reads it for the edition and the floor) and added it to the crate's `include`, but the Containerfile's COPY list still names only `Cargo.toml Cargo.lock PROTOCOL build.rs src`, and `.containerignore` excludes `corpus/` outright — DESIGN §10.1's *the build context is the image's include list* was kept on one channel and not the other. noodlezoo's reconciler (scripts/deploy/reconcile.sh) reads the registry's tag list, so it truthfully reports 0.0.54 as the newest released image and has stayed there for nine days.

Fix: COPY the record by name in the Containerfile, stop ignoring it in .containerignore, and add a test that derives the image's required inputs from build.rs's own `rerun-if-changed` lines and asserts each is COPY'd — so the fourth build input cannot repeat this. Verify with a real `make image` before closing; the close gate does not build the image.