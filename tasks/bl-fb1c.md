+++
title = "the empty-world masthead splits across two alignment axes: the wordmark is left-aligned while the tagline and name prediction beneath it are centred"
created = 1786163347
updated = 1786678332
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
QUALITY.md §1 criterion **G3** ('One grid per surface … one row is one scan line'), aggravated by **G4** at the large capture. Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out.

SYMPTOM. The bootstrap placeholder paints three stacked elements that are meant to read as one masthead — the mark + 'yog' wordmark, the tagline, and the greyed name prediction — but only two of them are centred. The wordmark sits hard against the left edge of the centre panel.

MECHANISM (read, not guessed). `src/shell/bootstrap.rs` wraps all three in `ui.vertical_centered`, but `crate::theme::wordmark` (`src/theme/mark.rs:46`) is itself a `ui.horizontal(…)`. egui centres each direct child widget; a horizontal child claims the full available width, so its contents start at the left. The two `ui.weak(…)` labels below it centre normally.

WITNESS: `Q-S0-default.png` — wordmark at x≈270 (the panel's left edge), tagline centred at x≈705. `Q-S0-max.png` — the same split at 2560px wide puts roughly 900px between the wordmark and the two lines that belong to it. `Q-S0-small.png` shows it at 420x320 too, so it is not a size-specific artifact.

Also visible in the same shots and worth the fixer's eye, though NOT filed separately: the tagline ('the key and the gate') and the name prediction ('will be named growing') are consecutive `ui.weak` labels in the same size and colour with no separation, so they read as one run-on sentence — 'the key and the gate will be named growing'.

REPRODUCTION: launch yog on an empty scratch world (`XDG_DATA_HOME=<scratch>` with only `yog/world/lernie/models.yaml` + `template/providers.yaml` seeded). The placeholder is the first frame; no gesture needed.

TRIAGE ONLY — filed by the first quality audit, not fixed by it.

---

GROUNDWORK (verified 2026-08-11, Ferrule, read-only — NOT claimed, NOT worked).

The ball's mechanism read is CORRECT and still current — rare enough among these audit balls to be worth stating. `src/shell/bootstrap.rs` wraps three children in `ui.vertical_centered`, and `crate::theme::wordmark` (`src/theme/mark.rs:46-52`) is:

    pub fn wordmark(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            draw(ui, &icon::Tints::rest());
            ui.add_space(3.0);
            ui.heading(egui::RichText::new("yog").color(GATE).strong());
        });
    }

`ui.vertical_centered` centres each child by the width that child REQUESTS. `ui.horizontal` requests the full available width, so its contents begin at the left edge; the two `ui.weak` labels request their own text width and centre normally. Hence one masthead on two axes.

FIX SHAPE: the row needs a known width before it is placed. Measure the heading galley (`egui::TextStyle::Heading.resolve(ui.style())`), add `MARK_PT` (28.0) + the 3.0 gap, and place the row with `allocate_ui_with_layout` at that exact width and `Layout::left_to_right(Align::Center)`. Then `vertical_centered` has a real width to centre and the mark travels with its own word. Keep it inside `theme::wordmark` — the ball notes the wordmark is used as the empty-workspace placeholder's identity seat, so the fix belongs to the wordmark, not to bootstrap's call site.

EVIDENCE NEEDS NO NEW MACHINERY: `paint_probe::collect` already returns `(String, Rect)` per galley — `Painted` — and `painted_of`/`painted_settled` hand back the positioned list. The ball's own witness is x-coordinates (wordmark x≈270 vs tagline x≈705), so the regression test is a direct assertion that the three galleys share a centre within a tolerance. `src/shell/acceptance/name_column.rs` is the precedent for measuring galley positions rather than asserting strings, and its reason is the same: a string assertion would pass on a tree that had deleted the element outright.

SECOND DEFECT NAMED BUT NOT FILED, from the ball body: the tagline ('the key and the gate') and the name prediction ('will be named growing') are consecutive `ui.weak` labels in the same size and colour, so they read as one run-on sentence — 'the key and the gate will be named growing'. Both are `ui.weak` in `bootstrap.rs`'s `vertical_centered` block. Whoever takes this ball is in the file already; fixing the alignment while leaving two lines that misread as one sentence would be a half-delivery.

---

PREREQUISITE, found under bl-36c3 (Larkspur, 2026-08-12): this masthead has no fixture. `src/shell/bootstrap.rs` is never rendered by any test in the tree — grepping its own literals (`say what you want done`, `start a conversation:`, `theme::TAGLINE`) finds nothing outside the file, and `src/shell/*` is coverage-excluded so nothing reports it. `fixture::world_unfocused()` does NOT reach it: it builds a workspace and declines to focus an agent, so `focused_workspace()` is Some and the frame paints `start_pane`, not `bootstrap`. I wrote an alignment/order assertion for this ball's surface and withdrew it for want of that fixture; the gap is filed as **bl-37bf**. Ferrule's note above is right that the evidence needs no new machinery — `paint_probe` returns positioned galleys — but it does need a world with no workspace in it.

---

PREREQUISITE LANDED (Larkspur, 2026-08-12): bl-37bf is closed as `7642946`. `fixture::world_empty()` builds a world with no workspace in it, so `shell::bootstrap` — this ball's surface — is now reachable from an acceptance test for the first time. `src/shell/acceptance/masthead.rs` renders it at all four sizes and pins the three runs whole and stacked in order.

It deliberately does NOT pin their horizontal alignment: that is this ball's claim, and one written to pass today would encode the defect. Add it there, beside the order assertion, using `paint_probe::seen_of`'s laid rect — Ferrule's note above is right that no new machinery is needed, and now no new fixture is either.

Two things the fixture work turned up that bear on the fix. First, `world_unfocused` is gone: it withheld the startup-focus argument while leaving the workspace in the roster, so `AppModel::startup_focus` derived a focus onto it — nothing that called it ever saw the bootstrap. Second, the masthead assertion had to match the wordmark **whole** (`text == "yog"`), not by prefix: the side panel's balls hint paints `yog exec bl prime` in the same frame, and a prefix match takes that for the mark. Worth knowing before writing an x-coordinate claim about `yog`.
