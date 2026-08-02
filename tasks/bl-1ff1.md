+++
title = "Raw toggle missing on Steps/Inbox/Files/Config tabs — STORIES §S7 promises it on every tab"
created = 1785646892
updated = 1785646892
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
STORIES.md S7 point 3: 'every tab has a Raw toggle showing the verbatim bytes.' Live UI: only Transcript renders 'Raw (verbatim bytes)'; Steps, Inbox, Files, Config have none (Files previews per-file — fine if the preview itself is verbatim, but the other tabs summarize with no bytes escape). Per repo rule, code and doc may not disagree: either land the toggle on the remaining tabs (S7-T1's assertion: Raw yields the underlying file's bytes unaltered) or amend STORIES to scope the promise to the tabs that have it, with the reason. Check Z13's planned test rows before choosing.