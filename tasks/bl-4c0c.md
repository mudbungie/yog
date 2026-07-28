+++
title = '＋ conversation renders as a tofu square — label it "new"'
created = 1785201065
updated = 1785201194
claimant = "tofu"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["usability"]
+++
UX testing, 2026-07-27 (the operator).

## Symptom

The side-panel button that starts a new conversation is labelled with the
fullwidth plus `＋` (U+FF0B), in `src/shell/navigator.rs:156`:

    if ui
        .button("＋ conversation")
        .on_hover_text("clear the target and type (n)")

egui's default font has no glyph for U+FF0B, so it paints as a tofu box: the
operator sees "□ conversation" and has to guess what the box means. The same
glyph is on the workspace mint in the top bar (`navigator.rs:44`,
`ui.button("＋")`), where it is worse — that button is glyph-only, so it renders
as a bare square with no word beside it at all.

## Ask

Replace the glyph with the word. `new conversation` in the side panel, and
`new` (or `new workspace`) on the tab-bar mint. Words survive font fallback;
a decorative codepoint does not.

## Scope note

The `＋` also appears in DESIGN §11 prose and in the doc comments that quote it
(`navigator.rs:15/23/118`, `keys.rs:117/127`, `start_pane.rs:23`). Fix the doc
alongside the code — DESIGN is the architecture authority, so the label change
lands there too, not just in the widget.