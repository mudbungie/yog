+++
title = "REMOTE $13 rework: the punched wire — no anchor, no port-forward, the engine never listens publicly; DHT rendezvous, punched TCP data path, held connections"
created = 1788233488
updated = 1788233488
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Three operator rulings 2026-08-31 reshape bl-4d56's design: no rented anchor; no port-forward — a non-publicly-dialable engine is desirable in itself; the data path is hole-punched. Presence and signaling ride the BitTorrent mainline DHT as a commons (both ends pure clients, BEP 44 signed items, payloads sealed under entry material); the punch is TCP simultaneous open and is now the primary path, which puts the source-port-reuse dependency ruling on the critical path and meets $10's held-connection criterion (a rendezvous costs seconds, so the punched connection is held). The relay/moor design is preserved verbatim as a parked carriage rung with a stated criterion. No splice and no TLS-in-TLS remain: a punched stream carries the inner mTLS wire directly.