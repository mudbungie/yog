+++
title = "transcript: user messages render collapsed as '▶ user:---' — show the input text"
created = 1785645552
updated = 1785645552
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-02, literal rendering:

    ▶ user:---
    · gpt-5.4: Yes

The user's own message is hidden behind a collapsed '---' row while the model reply shows. User input must be visible by default in the transcript — the operator should read the exchange, not expand every turn by hand. Find where transcript rendering (src/transcript/render.rs or its current home post-splits) decides a user turn is collapsible/elided and why it defaults collapsed ('---' smells like an empty-body projection or a fold marker). Check whether the '---' is yog eliding, or lernie storing the user turn oddly (e.g. body in a place the renderer doesn't read) — fix the real fact, not the symptom. Default: user turns expanded, showing their text; keep any collapse affordance as opt-in.