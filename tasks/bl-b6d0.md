+++
title = "a named sender's deposit still paints the raw id: header_line floors it at ladder rung three and never asks the name fact"
created = 1786685252
updated = 1786685252
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Surfaced by bl-45c7's value-derived naming scan (Lintel, 2026-08-13) — a live §3.3 leak the scan does not yet forbid. inboxview::header_line routes the deposit's sender through id_floor (rung THREE of the display ladder) and never through rung one, the name fact: a deposit from an agent that HAS a name paints the raw id while the name sits in the roster. Paint-layer repro with a named peer 'peregrine': the inbox row painted '✉ 20260731T101112Z-abcdef01 · t0' with 'peregrine' painted in the roster of the same frame.

Seat: src/inboxview/mod.rs:151 (header_line), painted from render.rs::render_deposit and src/composer/mod.rs:91. Candidate fix: display_name_of(agents, from) instead of id_floor(from) — correct for 'user' too, no branch — but it threads the agents snapshot through composer::rows, shell/inbox_queue.rs and the inspector's plumbing, and it changes what §5.1 #11's '✉ from · at' MEANS. That semantic shift is the design half of this ball: rule on it in the body before implementing. bl-45c7's landed invariant ('no more of an id than the floor spells') stays true either way. Verify premises against the tree first.