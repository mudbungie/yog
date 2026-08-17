+++
title = "text selection: double-click-drag doesn't extend selection by word boundaries — OPERATOR DISCUSSION, do not claim"
created = 1785645008
updated = 1786937231
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator report 2026-08-01: double-click-drag should extend selection by word boundaries (triple-click-drag by lines); egui reverts to character granularity. Investigation (2026-08-02, select-fixer) established: yog owns none of the selection path (all stock egui::Label / TextEdit), egui has never implemented this at any version, labels cannot be fixed from outside egui (LabelSelectionState has no public range access), and egui #2550 has been open since 2023-01 with no PR.

## DECISION (operator, 2026-08-01): Path A — upstream it. EXECUTED.

Upstream PR filed: **https://github.com/emilk/egui/pull/8370** (draft → ready after CI), announced in https://github.com/emilk/egui/issues/2550#issuecomment-5154053083, from fork mudbungie/egui branch `word-drag-selection` (local clone ~/dev/egui).

Root cause found during implementation: egui counts clicks only on pointer *release* (`PointerState`), so at drag-start nothing knows the gesture began with a double-click — that's why the feature never existed. The PR: (1) computes the would-be click count at press time (pub(crate)); (2) TextCursorState selects word/line on the second/third press and keeps it as the drag anchor; drag selects union(anchor, unit-at-pointer); (3) LabelSelectionState carries the anchor across passes for cross-label drags. Unit tests + 5 kittest integration tests (tests/egui_tests/tests/test_text_selection.rs), all local checks green (fmt, clippy incl. workspace, docs -D warnings, typos, lint.py, full egui + egui_tests suites; the 1-2px visuals/{button_image,radio} snapshot diffs are pre-existing on this GPU, confirmed against clean upstream).

## Remaining for this ball

1. Monitor the PR; address review. Fallback per decision: if it stalls indefinitely, close won't-fix-here (Path B costs nothing more).
2. When merged and released, bump eframe/egui in yog — the fix arrives with the dep bump; nothing to change in yog source.
3. Then close this ball.

Path C (TextEdit-only partial in yog) remains rejected: fixes the composer, not the transcript, and becomes deletion debt when the upstream lands.

---

Operator ruling 2026-08-16: keep monitoring the upstream PR (open, unreviewed, quiet since filing). Fallback set amended by the operator: if the PR stalls indefinitely, the options are (a) close won't-fix-here as before, OR (b) fork egui and carry the word-drag patch on the fork, pointing yog's eframe/egui dependency at it. Note before choosing (b): yog's dependency sources are registry-only (code-style rule 6), so a fork path means either publishing the fork to a registry or taking the phase-2 interim git-pin exception, which re-blocks 'make publish' — the fork is a last resort with a named exit, same as any git pin.
