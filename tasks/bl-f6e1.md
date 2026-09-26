+++
title = "yog: BEP 44 get/put walk from the bootstrap routers, which never answer get — every live publish and poll is dark in one round"
created = 1790392847
updated = 1790393009
claimant = "Urinalyses-T"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Measured by bl-5d8d from the deployed engine box (REMOTE §13.7 ruling 3). `Dht::get` and `Dht::put` start their `get` walk straight from the bootstrap list, and the mainline bootstrap routers answer `ping`, `find_node` and `get_peers` but never BEP 44 `get` (raw KRPC probe: dht.transmissionbt.com answered the first three on both its addresses, silent on `get`; the other two routers silent to everything from that box). So on the live mainline every `put` and `get` fails with 'no DHT node answered get' after exactly one round, 24 of 24 times across four configurations. The rendezvous loop (`src/wire/rendezvous/cycle.rs`) calls exactly these verbs from `mainline()`, so presence never publishes and the inbox never reads on the live commons; the suite cannot see it because the fake node answers `get` from the bootstrap position.

Evidence the fix works: seeding a second client with the nodes a `find_node` lookup toward the item's target returned, `put` stored at 3-7 nodes and `get` read the item back at seq 1 in every run whose seed walk converged (put 3.5-6.6 s, get 3.2-6.2 s).

Fix shape to design: the BEP 44 walk should reach the neighbourhood of the target by `find_node` (which the routers do answer) before asking `get`, e.g. the search asks bootstrap addresses `find_node` and everything it learns `get`; one walk, not two. A fake node that stays silent to `get` in the bootstrap position pins it in the suite.