+++
title = "five beads a limb and a flatter insertion: soften the elbow"
created = 1785719589
updated = 1785719594
claimant = "Quinces"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design", "icon"]
+++
Adjust the icon landed by bl-af0c (DESIGN §11).

**the operator's note, verbatim:** "add two more spheres each, evenly distributed. And
flatten the curve just a little bit more. Reduction of the sharp elbow is what
I'm looking for."

**Beads:** `BEADS` 3 -> 5. Nothing else changes — the spacing is already derived
from the count (even fractions of the inner edge, ends included), which is what
that earlier deletion of `BEAD_GAP` bought.

**Elbow:** the peak-to-mean curvature along the arc is the measure. `R_EASE` 4.0
-> 3.5 spreads the turn instead of banking it late; `T_EASE` 1.6 -> 1.5 starts it
marginally earlier; `SWEEP` 175 -> 190 pays back the ring coverage both cost.
Elbow 1.95 -> 1.83, ring 365 -> 353 degrees.