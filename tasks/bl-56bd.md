+++
title = "yog pins brazen =0.0.4 / lernie =0.0.2 — take the next ecosystem rung: brazen 0.0.5 + lernie 0.0.3"
created = 1785287536
updated = 1785306247
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
The manual run of the inheritance chain (the dependabot automation is deferred).

Upstream state at filing (2026-07-28): brazen 0.0.5 is queued in brazen release PR #2 (MERGEABLE/CLEAN, all 10 checks green) but UNPUBLISHED — crates.io latest is still 0.0.4. lernie 0.0.3 does not exist; lernie main is exactly v0.0.2 with an empty [Unreleased], and its version iteration IS the brazen bump (lernie Cargo.toml:47, `brazen = "=0.0.4"`).

Work here, once BOTH are on crates.io:
1. Cargo.toml: `brazen = { version = "=0.0.5", features = ["native-host"] }` and `lernie = "=0.0.3"`; refresh both pin comments (they name 0.0.4 / 0.0.2 verbatim).
2. `cargo update -p brazen --precise 0.0.5 -p lernie --precise 0.0.3`.
3. PARITY GATE (DESIGN 16.7, 'skew death is structural'): lernie 0.0.3 must declare `brazen =0.0.5`, the same exact version yog pins, so exactly ONE brazen resolves. Verify from the registry index, not from prose: `curl -s https://index.crates.io/le/rn/lernie`. yog must NOT take brazen 0.0.5 ahead of lernie.
4. Full gate: make build, make test, make lint, close-gate coverage.

BLOCKED until brazen 0.0.5 and lernie 0.0.3 are published — an exact pin cannot resolve an unpublished version.

## What lernie 0.0.3 carries (folded in from bl-d19b)

lernie main at 1e9bf51 holds: compactor-never-eligible + founding-commit checkpoint clock + single-point max_depth enforcement + loud compaction-merge conflict refusal (fixes yog bl-ebbd's incident class), with toolset/descriptor fixes (bl-bd9d/bl-55b1 lernie-side) landing behind it.

## Acceptance (folded in from bl-d19b)

When lernie 0.0.3 publishes: bump Cargo.toml pin, commit Cargo.lock, cargo test green, `scripts/drive/stories.sh run-s7` green (the bl-d82f acceptance carries over). Then bl-ebbd / bl-bd9d / bl-55b1 in this store can close against the consumed fix.

## Migration caveat (folded in from bl-d19b, sourced from lernie bl-38c2)

The widened worker grant only reaches NEWLY created workspaces: providers.yaml is authored into the first config commit at `lernie new` and frozen. After the pin bump, the existing workspaces (<workspace>, <workspace>, <workspace>, native-album, <workspace>) keep the old three-tool grant until their `config/default:providers.yaml` is hand-edited or they are recreated. Operator has declared current world state disposable play-state (2026-07-28) — recreating workspaces is acceptable; confirm at bump time. The descriptor prune (bl-18a9) needs no migration: it applies at dispatch time to every new agent branch immediately.