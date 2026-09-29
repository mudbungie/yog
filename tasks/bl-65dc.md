+++
title = "deploy: prove the punched path live from a laptop and the phone, then walk the engine off the overlay"
created = 1788232804
updated = 1790657374
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-4d56"
on = "claim"

[[blockers]]
id = "bl-df31"
on = "claim"

[[blockers]]
id = "bl-4263"
on = "claim"

[[blockers]]
id = "bl-653a"
on = "claim"

[[blockers]]
id = "bl-d00f"
on = "claim"
+++
REMOTE $13's cutover: provision the rendezvous material into the deployed engine's wire root and the client entries, prove first contact + held connection + re-punch live from a stable-connection laptop and from the phone on cellular (the risk pair — bl-a9b0's verdict feeds expectations), then retire the third-party overlay binding. If the cellular pair cannot punch, that is $13.6's criterion firing: decide accept-wifi-only or stand up the parked carriage rung (bl-89d2), not a silent workaround.

---

Live verification of bl-f6e1 + bl-9408 (origin/main ae209c0e) from the deployed engine box, 10 trials each: lookup 7/10, put 6/10 (acks 6-8), get 5/10 hits (5 of 6 good puts read back); median ok latencies lookup 5.8 s, put 8.8 s, get 7.9 s. Only 1 of the 4 mainline() bootstrap hosts answered (dht.transmissionbt.com). put/get no longer 0/24, but 10 of 30 walks still end dark at two rounds. Defect filed as bl-d00f (p1, blocks this claim) with the table and a hypothesis.

---

WIFI LEG PROVEN 2026-09-28 (phone on wifi behind a residential NAT, engine behind another): with the entry's direct address closed, a debug build of the phone app beside the operator's own read presence off the DHT, wrote its call, and punched a TCP connection to the engine's punch port across both NATs — six dials, each landing 1–2 streams ~30 s after launch, mTLS completed, NoodleZoo's roster and conversation list rendered. Only the phone's OBSERVED public address ever connected; its LAN address never did (expected). Laptop leg NOT provable from this laptop as configured: it routes through the engine's own box as an overlay exit node, so its call carries overlay addresses and the engine's own public IP (bl-f612 records the trap; the engine punched correctly, verified end to end with a probe from both vantages). Cellular leg pending. Defects found: yog-android bl-58a0 (concurrent ladders on one entry overwrite the single inbox slot), bl-f677 (held stream dies ~5 s after backgrounding), bl-df05/lernie bl-3e5b/thrall bl-3958 (client rendezvous path logs nothing); yog bl-355c landed the engine's log lines and /doctor counters.
