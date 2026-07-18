+++
title = "Y9: marks completion + inbox view + budgets fold"
created = 1784349557
updated = 1784349557
parent = "bl-4e66"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-dd2a"
on = "claim"
+++
DESIGN.md §15 Y9. Extend git_tree/marks.rs to all four refs/lernie/* namespaces (add abandoned + notify, with oids exposed for watermark comparison); inbox deposit parsing (---from/deposited_at/epitaph/terminal_ref--- frontmatter) as a view-model; budget-spent fold over Usage events across steps/<root>/ + steps/<root>-*/ response.json files. Files: src/git_tree/marks.rs (+50), src/inboxview/mod.rs (~130), src/budgets/mod.rs (~170).