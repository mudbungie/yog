+++
title = "claimed ball renders a stray bare-id row ('bl-d0df') with no title or verbs in the balls section"
created = 1785646890
updated = 1785646890
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Repro: one ready ball, press 's' (start). While the ball is bound and the start draft pending, the balls section shows the 'Continue bl-…' row AND, below the new-ball form, a bare grey 'bl-d0df' with no title, no state, no affordance. Verify in current code what that row is (the bound-balls list under the focused workspace?) — then give it its full row rendering (id: title, state colour, verbs) or drop it if it duplicates the Continue row. Single source: one ball should not render as two half-rows.