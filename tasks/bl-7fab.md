+++
title = "Theme the UI: iridescent-spheres palette + egui visuals (make it look alive)"
created = 1784523387
updated = 1784523604
claimant = "defecates"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["ui"]
+++
Branding pass — LANDED. yog is Yog-Sothoth, canonically 'a congeries of iridescent globes' (the balls thesis); lernie the hydra; brazen the brazen head. Previously the app rode stock egui dark grey with zero visuals.

What shipped:
- src/theme/{mod,tests}.rs — the congeries palette, yog's single colour authority. Lore-named hues: HYDRA (liveness/ok green), SPECTRE (in-flight/streaming blue), BRAZEN + BRAZEN_DIM (pending/warn bronze, quiescent tarnish), ICHOR (error red), ASH (stopped), SIGIL (uncertainty magenta), GATE (yog's selection/wordmark violet). Plus: visuals() deriving the whole egui Visuals (void-violet background strata, gate-violet selection, moonlit lavender text ramp), apply() installed in main.rs before first paint, pulse() — the one shared in-flight animation (git_tree + transcript now beat in step), integration_hue() (lernie→HYDRA, bz→BRAZEN, bl→GATE), and wordmark() (three iridescent spheres + "yog").
- Rewired git_tree/transcript/steps_view renders off their duplicated RGB triples onto the palette; inbox ✉ header now brazen like the tree's ✉n badge.
- Shell seats: attention strip wears the wordmark ("⚑ nothing stirs" when quiet, brazen count when not); empty workspace shows wordmark + "the key and the gate"; ops ⚠ rows in ichor; toolchain rows hue-keyed with hydra/ichor health; config-editor headings in their tools' hues.
- ToolState::permits bumped to pub(crate) for the toolchain health hue.
- DESIGN.md: §11 visual-identity paragraph + §12 module-map row for src/theme. README: congeries paragraph.

Gate: make check (fmt + clippy pedantic + ast-grep rules + cargo-deny + tarpaulin 100%) green; 623 tests.