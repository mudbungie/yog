+++
title = "activity accessory: the chip's failure count (· M ⚠) and the row marker ⚠ carry 'failed' by glyph alone"
created = 1785287143
updated = 1785373504
claimant = "entrance-51cb"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["ui"]
+++
Glyph-doctrine follow-up (DESIGN §11 "Glyph doctrine", filed by bl-5013).

src/opslog/live.rs (chip): "activity · N ops · M ⚠" — delete the glyph and the
chip reads "activity · 3 ops · 1", a dangling number; "failures" is said only by
⚠. src/shell/activity.rs (row): the per-op failed marker is "⚠" vs "·", again
glyph-only (the expanded record's exit/stderr recovers it, but the row surface
itself does not say failed).

Fix per the doctrine: say the state — e.g. chip "· M failed" (glyph optional on
top), and/or hover text; keep ⚠ for the glance.