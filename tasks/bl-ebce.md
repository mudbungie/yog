+++
title = "Y16: action verbs — message/stop/scan/close/unclaim/create/update + opslog wiring"
created = 1784349560
updated = 1784352845
claimant = "filtered"
parent = "bl-4e66"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-9fc6"
on = "claim"

[[blockers]]
id = "bl-c2b8"
on = "claim"

[[blockers]]
id = "bl-be28"
on = "claim"
+++
DESIGN.md §15 Y16. Dispatchers for lernie message/stop/scan and bl close/unclaim/create/update (exact argv per DESIGN §8.2, cwd = project for bl), each short-piped with its outcome appended to ops.jsonl; enablement predicates extending the existing actions pattern (Stop iff Live/InFlight; Close iff claimed AND bound; Message always — it is the resume gesture). Replaces the stderr-and-drop spawn_detached for short verbs. Files: src/actions/verbs.rs (~180), src/actions/mod.rs growth, src/shell/input_bar.rs (excl.).