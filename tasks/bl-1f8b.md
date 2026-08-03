+++
title = "the mark in the top-left of the shell, medium size"
created = 1785731170
updated = 1785731170
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator request 2026-08-03, verbatim: 'I would like the new logo in the top left, medium size.'

The new logo is the tangent-circles mark (bl-f60e, commit 030fcfc; generator emits assets/yog.svg plus sized PNGs assets/yog-{16,32,48,64,128,256}.png per bl-764e/bl-8cac). Render it at the top-left of the shell — likely src/shell/top_bar.rs, verify against the tree — at a medium size (roughly 24-48px on screen; pick what reads cleanly against the top-bar height, and use a suitably sized asset or SVG-derived texture rather than upscaling a small PNG). Embed the asset at compile time (include_bytes! or existing icon plumbing if the app already loads an icon for the window) — do not read from an install path at runtime; reuse the existing icon wiring if there is one rather than adding a second loading path. If DESIGN records the top-bar contents, amend it.