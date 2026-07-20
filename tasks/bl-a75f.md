+++
title = "DESIGN amendment: conversation-first start ladder + name-minted claim binding"
created = 1784523333
updated = 1784523334
claimant = "blunderers"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Amend docs/DESIGN.md per the lightweight-interactions discussion:

1. Start ladder: one composer, three prefills (nothing / path / ball); ball-start becomes the fully-loaded case of the general flow.
2. Binding dissolves into ball claimant metadata: yog mints a unique two-word name per conversation (embedded wordlist), claims are made --as that name, ball->conversation binding = claimant join. Explicitly late-mutable via bl claim/unclaim.
3. Delete the $XDG_DATA_HOME/yog/balls/<mirrored-path>/<ball-id> path-arithmetic binding; conversations live at a single named root.
4. Rework the join-state table on the claimant join; N balls per conversation replaces 1:1.
5. Follow-on implementation tasks filed separately after the amendment lands.