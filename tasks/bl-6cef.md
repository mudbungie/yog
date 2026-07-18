+++
title = "Y18: brazen config editor — raw text + bz validation + hash guard"
created = 1784349561
updated = 1784350335
claimant = "filtered"
parent = "bl-4e66"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-a5f0"
on = "claim"

[[blockers]]
id = "bl-9fc6"
on = "claim"

[[blockers]]
id = "bl-be28"
on = "claim"
+++
DESIGN.md §15 Y18. Locate via the Y2 fold; raw-TOML text editor; Apply = stage to .config.toml.yog-tmp-<pid> in the destination dir -> bz --config <temp> --dump-config gate (non-zero exit blocks with stderr, draft kept) -> content-hash guard against the loaded snapshot -> atomic rename. Read-only effective pane (bz --dump-config verbatim) + built-in-rows hint + credential-presence booleans + model-cache display with bz --list-models --provider <row> --json refresh. Files: src/config_edit/brazen.rs (~230), src/shell/config.rs (excl.).