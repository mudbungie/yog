+++
title = "chat header timestamp is the raw conversation id — render human ISO8601, keep the id subordinate"
created = 1785645781
updated = 1785645781
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-02, verbatim: 'the timestamp at the top of the chat is unconsumable. make it still ISO8601, but less built for the machine. <agent-id>'. The header renders the raw conversation id (compact ISO stamp + hash suffix). Render a human ISO8601 instead — e.g. 2026-08-01 22:54:18Z (separators, space over T is fine; stay ISO8601-derived, no locale prose) — derived by parsing the id, single source of truth, no second stored field. The hash suffix is not a timestamp: drop it from the headline; keep the full raw id discoverable (hover or weak inline) since it's the on-disk key. Same header surface as bl-9786.