+++
title = "the empty-world masthead splits across two alignment axes: the wordmark is left-aligned while the tagline and name prediction beneath it are centred"
created = 1786163347
updated = 1786163347
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
QUALITY.md §1 criterion **G3** ('One grid per surface … one row is one scan line'), aggravated by **G4** at the large capture. Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out.

SYMPTOM. The bootstrap placeholder paints three stacked elements that are meant to read as one masthead — the mark + 'yog' wordmark, the tagline, and the greyed name prediction — but only two of them are centred. The wordmark sits hard against the left edge of the centre panel.

MECHANISM (read, not guessed). `src/shell/bootstrap.rs` wraps all three in `ui.vertical_centered`, but `crate::theme::wordmark` (`src/theme/mark.rs:46`) is itself a `ui.horizontal(…)`. egui centres each direct child widget; a horizontal child claims the full available width, so its contents start at the left. The two `ui.weak(…)` labels below it centre normally.

WITNESS: `Q-S0-default.png` — wordmark at x≈270 (the panel's left edge), tagline centred at x≈705. `Q-S0-max.png` — the same split at 2560px wide puts roughly 900px between the wordmark and the two lines that belong to it. `Q-S0-small.png` shows it at 420x320 too, so it is not a size-specific artifact.

Also visible in the same shots and worth the fixer's eye, though NOT filed separately: the tagline ('the key and the gate') and the name prediction ('will be named growing') are consecutive `ui.weak` labels in the same size and colour with no separation, so they read as one run-on sentence — 'the key and the gate will be named growing'.

REPRODUCTION: launch yog on an empty scratch world (`XDG_DATA_HOME=<scratch>` with only `yog/world/lernie/models.yaml` + `template/providers.yaml` seeded). The placeholder is the first frame; no gesture needed.

TRIAGE ONLY — filed by the first quality audit, not fixed by it.