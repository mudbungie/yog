+++
title = "design investigation: thin-client/backend split — one yog backend on the home server, multiple UI clients sharing chats, each client exposing its machine-local tools"
created = 1786682408
updated = 1786682408
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
the operator's question (2026-08-13): what would it take to keep a split in yog — a thin UI client talking to a backend — so one yog instance on the home server serves multiple clients on multiple systems, sharing chats, with each client able to expose local tools to interact with its own system/environment.

Deliverable of the first pass: an architecture assessment (what already supports the split, what has to change, rough sequencing). If the direction is adopted, the follow-on deliverable is a living design doc (docs/), not a growing task body.