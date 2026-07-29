+++
title = "bump lernie pin to =0.0.3 when published — carries the compaction-recursion fix (lernie bl-a9eb) and toolset fixes"
created = 1785288298
updated = 1785288392
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
lernie main at 1e9bf51 holds: compactor-never-eligible + founding-commit checkpoint clock + single-point max_depth enforcement + loud compaction-merge conflict refusal (fixes yog bl-ebbd's incident class), with toolset/descriptor fixes (bl-bd9d/bl-55b1 lernie-side) landing behind it. When lernie 0.0.3 publishes: bump Cargo.toml pin, commit Cargo.lock, cargo test green, scripts/drive/stories.sh run-s7 green (the bl-d82f acceptance carries over). Then bl-ebbd/bl-bd9d/bl-55b1 in this store can close against the consumed fix.

## Migration caveat (from lernie bl-38c2)

The widened worker grant only reaches NEWLY created workspaces: providers.yaml is authored into the first config commit at `lernie new` and frozen. After the pin bump, the existing workspaces (<workspace>, <workspace>, <workspace>, native-album, <workspace>) keep the old three-tool grant until their `config/default:providers.yaml` is hand-edited or they are recreated. Operator has declared current world state disposable play-state (2026-07-28) — recreating workspaces is acceptable; confirm at bump time. The descriptor prune (bl-18a9) needs no migration: it applies at dispatch time to every new agent branch immediately.
