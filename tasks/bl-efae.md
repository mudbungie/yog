+++
title = "yog: presence carries the OBSERVED endpoint — read the reflected address off KRPC replies (BEP 42 `ip`) and publish it beside the route-local one"
created = 1790144360
updated = 1790144360
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-4263"
on = "claim"
+++
REMOTE §13.2 rules that presence must carry the observed endpoint, not the local one (§13.8: on a cellular path the observed one is the only one there is, and behind a residential NAT a second host already holding the preserved external port makes the two differ). bl-4263's loop publishes the box's route-local addresses (`punch::local_ips`, a UDP connect that sends nothing) at the punch port — the direct-address and LAN case, and the port-preserving residential case §13.8 measured — and nothing else, because the engine has no reflector: the ruling on bl-4263 was to consume `src/dht`'s interface and nothing behind it, and the reflected address lives behind it. The commons already answers the question: every KRPC reply may carry the querier's observed `ip` (BEP 42, six or eighteen compact bytes), which the walk reads and discards today. Read it in `src/dht` (a `Dht::observed() -> Vec<SocketAddr>` over the last walk, or handed back beside a `put`), and have `wire::rendezvous::cycle::publish` put it in the presence list beside the local addresses. Two cautions to carry into the design: it is a UDP-observed endpoint and the punch is TCP, so the observed PORT is the DHT socket's mapping and not the punch port's — publish the observed ADDRESS at the punch port and trust port preservation (§13.8 measured it on the residential NAT), and state that the CGNAT case (a rewritten port, §13.8) is still open; and an `ip` field is one node's claim, so take the value most nodes agree on, never the first.