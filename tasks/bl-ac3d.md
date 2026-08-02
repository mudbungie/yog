+++
title = "left panel width explodes to fit the project's absolute path and never shrinks back"
created = 1785646879
updated = 1785647094
claimant = "Ferrick"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Repro: prime a project whose path is long (e.g. under a deep /tmp scratch). The balls section's '+ new ball · <absolute path>' row sizes the egui SidePanel to the path's full width (~690px of a 1150px window observed), the panel keeps that width forever (egui SidePanels grow and never shrink), and at 800x500 the center pane is left ~110px — unusable. STORIES already names this as a harness lesson ('a layout is a coordinate too'); it is equally an operator bug. Fix: render the project by basename (full path on hover/expansion), elide long labels, and clamp the panel to a fraction of the window with shrink allowed. Acceptance: with a 120-char project path, the panel stays ≤ ~35% of window width at 1150x760 and the center pane stays usable at 800x500.