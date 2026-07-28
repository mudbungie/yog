+++
title = "deny.toml's advisory ignore list is stale — two RUSTSECs no longer match any crate"
created = 1785201669
updated = 1785201834
claimant = "denyfix"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["hygiene"]
+++
Noticed in passing by the bl-24d3 implementer, 2026-07-27. Pre-existing, not
introduced by that work.

## Symptom

`cargo deny check` (run by `make deny`, itself run by `make lint` and so by the
`bl close` gate on every delivery) emits two warnings on every run:

    advisory-not-detected: RUSTSEC-2026-0194
    advisory-not-detected: RUSTSEC-2026-0195

They are in `deny.toml`'s `[advisories] ignore` list and no longer match any
crate in the tree — the dependency that carried them is gone or moved past
them. AGENTS.md records the ignores as "the three ignored eframe-stack
RUSTSECs" (see the `deny` target's comment in the Makefile), so at least one of
the three is still live; check each individually rather than clearing the list.

## Why bother

An ignore list that warns on every gate run trains everyone to read `cargo deny`
output as noise. The next real advisory arrives into a channel nobody reads.
That is the whole cost, and it is enough.

## Ask

1. For each entry in `[advisories] ignore`, determine whether it still matches a
   crate in `Cargo.lock`. Drop the ones that do not.
2. Update the Makefile `deny` target's comment, which currently says "the three
   ignored eframe-stack RUSTSECs" — if the count changes, the comment is wrong.
   AGENTS.md keeps the pinned-tool rationale in that comment; keep it truthful.
3. `cargo deny check` should exit clean with no `advisory-not-detected` output.

## Note

`deny.toml` pins cargo-deny 0.20.2 and the gate is reproducible by design — do
not bump the pin as part of this. If a dropped ignore turns out to still be
needed under a different id, say so rather than re-adding it blindly.