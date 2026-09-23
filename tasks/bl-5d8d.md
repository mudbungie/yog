+++
title = "yog: the first live mainline walk — tune dht::Config on evidence from a box whose egress passes UDP/6881"
created = 1790144363
updated = 1790144363
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-4263"
on = "claim"
+++
bl-4263 was to make the first live walk against the mainline DHT its first act and record the numbers in REMOTE §13.7 ruling 3. It ran the walk once from the development box, four configurations (BEP 5 defaults; alpha 5; a 1 s round; 128 queries): every bootstrap node was silent for the whole round, every time, so the client reported 'no DHT node answered' in exactly one round — 2.0 s at the default, 1.0 s at the shortened one. A raw KRPC `ping` sent from a Python one-liner to three bootstrap nodes (router.bittorrent.com, dht.transmissionbt.com, router.utorrent.com) answered nothing either, so it is the box's egress dropping UDP/6881 (or its replies), not the client's datagrams — the same client answers the suite's fake node on loopback in milliseconds. Nothing was tuned on that evidence because it is evidence about a firewall. Run the same walk from the deployed engine box (the one that will carry the wire — §13.6's own criterion) with `Config::default()` and the three variants, record lookup/get/put latencies and ack counts in REMOTE §13.7 ruling 3, and move `Config`'s defaults only if the numbers say to. Keep it out of the suite: an ignored test or a throwaway binary, deleted after, as bl-4263 did.