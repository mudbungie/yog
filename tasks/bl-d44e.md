+++
title = "the live mark belongs to the conversation, not the window: move it out of the top bar into the altitude-1 header"
created = 1785736969
updated = 1785736969
claimant = "Dejectedly"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ruling, 2026-08-02 (this session), verbatim: *"bring the logo down into
the pane. Not in the top right top right. It's part of the conversation, so it
should be inside the block, not up on the top top"*.

bl-b768 seated the live mark in the **window's** top-right corner, beside the
workspace tab bar. That was the wrong altitude and the ruling names why: the
mark states what the **open conversation's** agents are doing (§5.1 #28b — its
seats are the focused root and its subtree). A fact scoped to one conversation
does not belong in altitude-0 chrome, which is about workspaces and totals.

## The seat

`shell::workspace::header`'s first line — the conversation's own headline row,
which already carries the §3.3 display name, the started-at stamp and the §5.1
#28 flight badge. The mark goes **right-aligned on that line**, so it reads as
belonging to the conversation whose name it shares a row with.

It therefore paints only when a conversation is open: `header` runs past the
nothing-selected early return, so the empty case never reaches it. The mark
appears exactly when there is something for it to describe.

## The wordmark text goes with the seat, not with the mark

In the top bar the mark was painted with "yog" beside it — chrome branding the
window. Inside a conversation that word means nothing, so the pane seat is the
**mark alone** with its hover roster. `theme::live_mark` drops the heading;
`theme::wordmark` (mark + "yog" + tagline) is untouched and keeps its own seat,
the empty-workspace placeholder.

## What the top bar loses

Nothing replaces it. The pre-bl-b768 left-hand resting wordmark does **not**
come back: two marks on screen at once — one inert, one live — would be the same
glyph meaning two things. One mark, always the live one; identity is carried by
the window icon, the desktop entry and the empty-state placeholder. Altitude 0
goes back to being only what it is for: the attention strip and the workspace
walls.

## Known adjacency (not fixed here)

The flight badge on that same line states the conversation's one §5.1 #28 class
in words ("◐ inference — a model call is streaming"), and for a childless
conversation the mark's eye now says the same thing in colour beside it. They
are different questions at different granularities — one class for the subtree
versus every agent's own state — and the badge carries the words the glyph
doctrine requires. Left as is; if it reads as noise once used, the badge is the
one to reconsider, not the mark.

## Change

- `src/shell/workspace.rs` — the mark on the header line, right-aligned.
- `src/shell/top_bar.rs` — the seat removed; the corner returns to the tab bar.
- `src/theme/mark.rs` — `live_mark` paints the mark and its hover only.
- DESIGN §11 altitude 0 (the seat leaves) and altitude 1 (it arrives), and the
  live-mark paragraph's siting sentence.
- README's siting sentence.