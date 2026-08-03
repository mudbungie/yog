+++
title = "round the insertion lobes, brighten the green, seat the beads on the inner curve"
created = 1785719438
updated = 1785719438
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design", "icon"]
+++
Adjust the icon landed by bl-4ab4 (DESIGN §11).

**the operator's note, verbatim:** "very good direction. Adjust the arcs a little
earlier, so that they look less like a jester's hat. I think I was wrong on the
green: more electric, other colors are good. The green balls got moved entirely
off the tentacles, though. Put them on the inner curve, equally spaced along
it."

**Diagnosis of the jester's hat:** it is `T_EASE`, not `SWEEP`. With the sweep
easing at 2.0 the limb shoots almost straight out before any turn accumulates,
so the lobe comes to a point. Lowering it to 1.6 starts the turn earlier and
rounds the lobe — but the sweep easing alone costs ring coverage, so `R_EASE`
rises to 4.0 (radius plateaus sooner, longer run against the ring) and `SWEEP`
to 175. Ring closes at 365 degrees.

**Beads:** they were spaced back from the tip, which walked them off the flesh
as it flattened. Space them along the INNER EDGE instead, at k/(BEADS+1) of its
arc length — which also deletes `BEAD_GAP`, since the spacing now follows from
the count.

**Green:** driven to 1.15 rather than 0.92; purple and gold stay as landed.