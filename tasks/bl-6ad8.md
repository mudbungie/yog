+++
title = "pending ball-start draft stacks a second composer above the regular one — two 'You are <name>.' lines at once"
created = 1785646891
updated = 1785647682
claimant = "Whin"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Repro: press 's' on a ready ball. The center-bottom now shows the start-goal draft box (Send (detached prompt) / Cancel) AND the regular 'start a conversation' composer below it, each with its own identity preview line ('You are traction-flatness.' twice). Two live composers is ambiguous — which box does Enter fire? Fix: while a start draft is pending, it replaces the composer (one box, one Enter — the S0 principle); Cancel/Escape restores the composer. Acceptance: at most one goal-entry box visible at any time.