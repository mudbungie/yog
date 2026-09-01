+++
title = "yog: the DHT client — bencode, KRPC iterative lookups, BEP 44 get/put, pure client only"
created = 1788232800
updated = 1788233559
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-4d56"
on = "claim"
+++
REMOTE $13.2: both ends are pure clients of the mainline DHT — outbound-UDP iterative lookups and BEP 44 signed mutable get/put, never a node: no routing answers, no storage, no listener. bencode and the KRPC query shapes in-house; ed25519 via ring (already in the graph); zero new crates expected — a proposed dep is a rule-6 ruling. Tested against a fake DHT on loopback UDP, designed for that from the first line (the coverage floor is the gate). $13.7 ruling 2 rides here: build inside yog first, measure true size, then rule on whether seat/foot/app reimplement or consume a published crate.