+++
title = "conversation display name: the §3.3 three-rung ladder — stamped name, first payload line, root id — one function"
created = 1785201512
updated = 1785374603
claimant = "entrance-1b5a"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["implementation"]

[[blockers]]
id = "bl-2706"
on = "claim"

[[blockers]]
id = "bl-df65"
on = "claim"
+++
Implementation of the bl-df65 design ruling (DESIGN §3.3 as amended).
Needs bl-2706 (the stamp it parses is minted and worded there). Read
DESIGN §3.3 and §11 first. This body REPLACES the superseded bl-68d9 scope
(display name = first payload line); bl-df65 gave conversations real
minted names and demoted the payload line to a preview.

## The ruling being implemented

DESIGN §3.3 (as amended at bl-df65):

> **Display: the name is the title; the first payload line is the
> preview.** One function derives what a conversation is called, as a
> ladder: the goal's stamped name → the first payload line (the goal with
> the stamp stripped) → the root agent id. The §11 row title, the center
> header, and the composer's `message <x>` target label (bl-2f30) all read
> rung one and fall through; foreign and hand-typed roots (no stamp) land
> on rungs two and three. The strip is the inverse of the stamp — compose
> and strip live in one module — and recognizes both retired shapes
> (`Your workspace is <x>.`, and the pre-bl-68d9 `You are <x>.` whose
> `<x>` was a *workspace*).

DESIGN §11 altitude 0: the row shows "the conversation's **name** with its
preview weak beside it (§3.3's display ladder: the stamped minted name is
the title, the first payload line is the preview/subtitle, the root id
when neither exists)". §11 altitude 1: the center header leads with the
display name, root id weak beside it.

Accepted cosmetic misread (do not "fix"): a legacy root stamped with the
retired `You are <ws-name>.` shape parses that workspace name as if it
were the conversation's own, until lernie's 30-day retention ages the goal
out. Bounded, display-only, ruled accepted in §3.3.

## Today (the defect)

The preview is `steps/<id>/001/request.json` first-message content
(`src/git_tree/detect.rs` `extract_request_preview` + `truncate_preview`,
which collapses ALL whitespace before capping at 80), so every stamped
conversation previews as its identity line; and the row/header/composer
have no name at all, only the raw root id.

## Changes

1. `src/start/goal.rs` — beside the compose (bl-2706), the strip/parse:
   given a goal's raw text, recognize the first line as a stamp iff it
   matches `You are <x>.` or the retired `Your workspace is <x>.`; return
   (stamped name if any, payload = the rest after the blank line).
2. `src/git_tree/detect.rs` — `extract_request_preview` strips BEFORE
   whitespace-collapsing (the strip is line-wise).
3. One display-name function implementing the three-rung ladder (name →
   first payload line → root id), the single source for: the §11 row
   title, the navigator fallback (`src/shell/navigator.rs`'s inline
   `if row.preview.is_empty() { row.root_id }`), the center header, and
   bl-2f30's composer labels. Natural home: beside `ConvRow`
   (`src/nav/convs.rs`).
4. §11 row renders name as title with the payload-line preview weak beside
   it; center header leads with the display name, root id weak beside it.

## Acceptance

- Goal "You are <workspace>.\n\nBall bl-1: fix" → name "<workspace>",
  preview "Ball bl-1: fix". Legacy "Your workspace is <workspace>.\n\nX"
  → no name rung, preview "X", display name "X". Hand-typed goal with no
  stamp → untouched payload, display name = its first line. Nothing at
  all → root id.
- Row title, navigator fallback, center header, and composer labels all
  derive from the one ladder function.
- `make check` green.