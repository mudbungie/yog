+++
title = "context-window percentage per conversation in the status bar"
created = 1785650734
updated = 1786513273
claimant = "Ingot"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator request (2026-08-01, codex-comparison follow-up), verbatim: "I'd like to see context window percentage shown in the status bar for each chat."

## Dependency
brazen bl-75f7 (filed same day): the model row's context window size becomes a brazen-owned fact on its read surface. This ball renders percentage = context tokens / window; it does not land before bl-75f7 provides the denominator (no capability theater — without a real window, render nothing, not an estimate).

## The honest numerator
NOT the cumulative budgets fold (src/budgets/ sums usage across steps — that's spend, not fullness). Context fullness is the LAST step's input-token Usage — brazen's per-step counters are already committed in step records and folded by the budget inspector; the latest step's prompt size is the true "how full is this context now". Verify the counter fields carry input tokens distinctly before wiring.

## Placement
Per-conversation status (the header line where spend already renders, and/or the roster row) — decide per DESIGN §11's altitude rules; derivation in the pure VM, glue thin, as ever. Model identity for the window lookup is the conversation's frozen model (src/model_pick/header.rs already derives it).