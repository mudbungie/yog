+++
title = "the drive front door loses its red-run report when failure happens before the first verdict"
created = 1787206331
updated = 1787275498
claimant = "Zircons-Drive"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "drive", "testing"]
+++
## Deterministic reproduction

Put a synthetic executable named `Xvfb` first on `PATH` that exits immediately, then run:

```sh
DRIVE_ROOT=/tmp/yog-drive/evidence scripts/drive/drive.sh ladder run-s5s8
```

Preflight accepts executable presence. Seat acquisition then reports that no display was claimed. The wrapper calls the report generator, which fails because no `verdicts.jsonl` exists, and leaves a zero-byte `drive-log.md`. The secondary “no verdicts” error obscures the primary seat failure.

## Contract

`scripts/drive/drive.sh` says:

> “The skeleton ... exists whether the ladder was green or red — a red run is exactly when a report gets written.”

A pre-verdict failure is still a red run.

## Required invariant

The front door must preserve the primary failure and emit a nonblank report even when zero beats ran. The report should explicitly say “no verdicts produced” and carry the failing stage; report generation must not become the final error.