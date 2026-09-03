+++
title = "the corpus ledger stamps 'since' at first regeneration, not at the bump: a shape edited before PROTOCOL is raised keeps the pre-bump number, and two shapes on main carry the wrong one"
created = 1788416400
updated = 1788416408
claimant = "Spellbind-Q"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["boundary", "corpus"]
+++
`src/boundary/corpus/ledger.rs::advance` refuses a signature change only when the entry's `since` is already the protocol being generated. So a shape edited at protocol N whose `since` is older gets stamped N — and if N is then found to be already spent (another ball bumped in the same cycle, or N shipped) and PROTOCOL is raised to N+1, the next regeneration sees an unchanged signature and keeps `since: N`. The ledger then says the shape changed at a version it did not change at.

Seen twice today: `reply/config` came out at `since: 11` for a change that shipped at 13 (corrected by hand in bl-dc3f), and on main right now `reply/attention` and `reply/acknowledged` read `since: 11` though bl-09ef moved them at 12.

The rule the ledger wants is not 'one move per shape per version' (REMOTE §9.9 already records that test as wrong) but 'a signature may change only across a bump'. Shape: the ledger records the PROTOCOL it was last generated at as one top-level field; `advance` refuses any signature change unless the protocol being generated is greater than that recorded one, and stamps `since` with the new protocol. No per-shape reasoning about 'first regenerated'. Correct the two stale entries in the same change (to 12), and state the rule in REMOTE §9.9 where the old test is recorded as wrong.