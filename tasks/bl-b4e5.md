+++
title = "login pane: every provider exits 78 — wrong flow (needs --browser), un-loginable rows offered, reason line never shown"
created = 1785287184
updated = 1785287184
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator hit this live, 2026-07-28: every Login button fails exit 78 (EX_CONFIG) and the pane only offers the run-by-hand bz syntax.

## Repro (all four rows on this box)

    yog bz --login --provider codex                 → 78: this provider has no device endpoint; use `--browser`
    yog bz --login --provider claude-session-direct → 78: (same)
    yog bz --login --provider local                 → 78: provider `local` has no `oauth` config; add an `oauth` block to its row
    yog bz --login --provider claude-code           → 78: (same)

## Three defects

1. **Wrong flow.** src/login/mod.rs hardcodes `--login --provider <row>` — the device-code flow. Rows like codex have oauth but no device endpoint; brazen says outright to use `--browser`. yog is a GUI app: the browser flow is arguably the RIGHT default here. Fix: use the flow the row actually supports — ask brazen (it's linked in-process; BzRunner::providers already projects the table — check whether the projection exposes oauth/device capability, and extend the projection if not; do NOT duplicate brazen's classification in yog).
2. **Un-loginable rows offered.** local / claude-code have no oauth block at all — a Login button for them can only fail. Don't offer it (or offer it disabled with the reason). Single source: derive loginability from the same brazen projection.
3. **The reason never reaches the operator.** bz prints the exact remedy on stderr(?) but the pane apparently shows only the fallback command — verify whether LoginView.lines carries stderr, and make the terminal error line(s) visible in the pane. The fallback command shown must also be one that would actually work (e.g. include --browser when that's the remedy).

## Constraints

- DESIGN §8.3/§16.2 stand: ambient config, streamed-piped spawn class, credentials stay bz's. The fix is flow selection + capability projection + surfacing, not a config override.
- If brazen's published pin cannot express a needed projection, record that in this task and do the best surfacing possible without it.