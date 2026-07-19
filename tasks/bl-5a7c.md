+++
title = "W3: lernie home seeding via the upstream bootstrap verb"
created = 1784435199
updated = 1784435199
parent = "bl-1a3c"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-c68f"
on = "claim"
+++
DESIGN §16.6 W3. On the first Start against an unseeded world, invoke lernie's own bootstrap verb (upstream lernie bl-6d83 — read its landed verb name/contract from /home/u/dev/lernie when claiming; if not yet landed, this ball is blocked in fact even though claimable — check first) to populate LERNIE_HOME; yog never reproduces lernie's seed logic (§14 rejection). Skipped when the world is already seeded — the general path with the seed present, not a bootstrap special case (§3.4). Files: src/world/seed.rs (~90), start-flow wiring. Outcome to ops.jsonl. Gate as always.