+++
title = "the mark becomes tangent circles on pinned arcs: endpoints fixed, swell the only knob"
created = 1785730776
updated = 1785730785
claimant = "Quinces"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design", "icon"]
+++
**the operator's ruling:** "5 sharp curve is the winner ... ship it. it's a strict
improvement."

**The construction.** Three circles tangent to the centre one, 120 degrees
apart, one at bottom dead centre. Each arm ends 60 degrees of arc away at
`END_R`, which puts the top-right arm's last circle directly over the main
circle. Both endpoints are **pinned**; the circles between them ride an arc
whose sagitta (`SWELL`) is the only free parameter, and everything else falls
out of it — including equal legs, since even spacing along a circular arc gives
equal chords for nothing.

**A correction to record.** the operator observed each arm is "ostensibly a perfect
semicircle". It is not, quite: at `SWELL` 0.448 the arc sweeps **167.4 deg**. An
arc is a semicircle exactly when its sagitta equals the HALF chord, which is
`SWELL` 0.5 — and that sweeps 180 deg but drops the departure to 41.8 deg off
the radial, losing the 45 deg that was the original ask. The two properties are
mutually exclusive; 0.448 is shipped because it is what was chosen and seen, and
because it leaves more frame margin (0.465 against 0.477). Record both so the
swap is one constant.

**Reuse:** `icon/arc.rs` already names an arc by two endpoints and a sagitta, so
this needs no new primitive — `arc`, `stroke` and `lune` all carry over, and
`raster.rs`/`vector.rs` are untouched.