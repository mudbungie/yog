+++
title = "yog: DHT walks still end dark at two rounds ~1 in 3 live, so BEP 44 put/get fail ~4 in 10 from the deployed engine box"
created = 1790393524
updated = 1790393524
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Live measurement (bl-65dc verification) of origin/main ae209c0e — after bl-f6e1 and bl-9408 — run from the deployed engine box with a throwaway static probe using the crate's own `mainline()` roster, default `Config` (alpha 3, K 8, 1 s round, 64 queries), a fresh client and UDP socket per verb, 10 trials: `lookup` of a random id, `put` of a fresh `Keypair::generate` item (salt `probe`, seq 1, 16 random bytes), then `get` of it from a third fresh client.

| verb | success | median | p90 | notes |
|---|---|---|---|---|
| lookup | 7/10 | 5.8 s (ok) | 7.4 s (ok) | 3 errs "no DHT node answered find_node", each at 2.05 s |
| put | 6/10 | 8.8 s (ok) | 10.8 s (ok) | acks 6,8,8,7,8,8; 4 failures at 2.05 s (3 "no node answered get", 1 "no DHT node stored the item") |
| get | 5/10 hits | 7.9 s (hit) | 8.6 s (hit) | 5 of the 6 successful puts read back; the 6th returned Ok(None) at 2.08 s |

Bootstrap: only `dht.transmissionbt.com` answered (2 of its 3 addresses; the third is v6 and the v4 socket cannot send to it). `router.bittorrent.com`, `router.utorrent.com` and `dht.aelitis.com` were silent on every trial — widening the roster to four bought nothing from this box.

What improved: BEP 44 put/get went from 0/24 to 6/10 put and 5/6 get-after-good-put; the bl-f6e1 fix is real.

What did not: 10 of 30 walks ended at exactly two rounds (~2.05 s) — the bl-9408 stall still happens, now about one walk in three, and bl-9408 turned it into an honest Err rather than preventing it. Reading `src/dht/lookup.rs` `search`: next picks are `pool.values().take(k).filter(not asked).take(alpha)`, so asked-but-silent nodes keep occupying the K window forever; once the few distinct nodes the single answering router named are asked and silent, picks is empty and the walk breaks although the pool may hold unasked nodes past K. Likely fix direction: rank the window over responders plus unasked candidates (drop silent nodes from the K window), and/or re-seed from the bootstrap when a round yields no responder. Unverified hypothesis — confirm against the trace before building.

Impact: a presence publish or inbox poll fails roughly 4 times in 10 per attempt; bl-65dc's live proof is not reliable on this.