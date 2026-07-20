+++
title = "Theme the UI: iridescent-spheres palette + egui visuals (make it look alive)"
created = 1784523387
updated = 1784523387
claimant = "defecates"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["ui"]
+++
Branding pass. yog is Yog-Sothoth — canonically 'a congeries of iridescent spheres' (the balls thesis); lernie the hydra; brazen the brazen head. Today the app rides stock egui dark grey with zero visuals — dead.

Deliverables:
- src/theme/: single-source-of-truth palette module (semantic constants, iridescent values) + apply(ctx) setting egui Visuals (void-violet backgrounds, gate-violet selection/accent, rounding). 100% covered.
- Rewire duplicated Color32 constants in git_tree/render.rs, transcript/render.rs, steps_view/render.rs to the palette.
- main.rs: apply theme in eframe creation closure; window title/wordmark touches.
- Wordmark seats (navigator strip, empty workspace placeholder): congeries mark + 'the gate and the key' tagline.
- DESIGN.md §12 module-map amendment for src/theme.