+++
title = "cellular result for the NAT probe: the engine box punches a direct UDP path to the phone on cellular over carrier IPv4; the laptop behind the same NAT reaches it by relay only"
created = 1788493530
updated = 1788493568
claimant = "Spellbind-Z"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["remote"]
+++
Follow-up to bl-a9b0 (closed), whose cellular pair was recorded NOT MEASURED because the phone was on the home Wi-Fi. The operator then put the phone on cellular with the overlay still on, and the overlay's own punch attempt was re-run from both residential hosts.

- **Engine box → phone on cellular: DIRECT.** The overlay's ping settled on a direct path to the carrier's public IPv4 endpoint within its first few probes, and the path was still direct on re-checks minutes later. The carrier-side external port was a LOW number, which is consistent with a carrier-grade NAT allocating from a port block; that is an observation, not a classification.
- **Laptop → phone on cellular: RELAY ONLY.** Forty probes over about three minutes, every reply via the relay, 'direct connection not established'. The laptop sits behind the SAME residential NAT as the engine box (bl-a9b0 measured it endpoint-independent for UDP and TCP, port-preserving, no IPv6, no port-mapping protocol), so the asymmetry is not the home NAT's class. Candidates worth one look, not asserted: the laptop's overlay daemon logged a gateway/self-IP change during the run (a fresh mapping mid-probe), and two hosts behind one NAT contend for the same preserved external port (bl-a9b0 saw the NAT rewrite the second host's port).
- The phone's cellular path DOES publish a carrier IPv4 endpoint when on cellular — bl-a9b0's 'no carrier-observed public v4' line was a Wi-Fi artefact and must be corrected.

**Verdict against REMOTE §13.6:** for the pair the design exists for — the engine and a phone on cellular — the criterion is NOT met: the punch works, UDP, over IPv4. TCP simultaneous open on that pair is still unmeasured (the overlay's punch is UDP), and the home side still holds no routable IPv6, so §13.3's v6-first rung is still dead on the residential end.

Scope: amend REMOTE §13.8 (the measurement record) with the cellular rows and the v4 correction, §13.6's evidence status, and §13.3 where it says the v6 rung is the only reliable one; pull the phone's advertised endpoint list from the engine box's overlay status (addresses stay OUT of the doc — shapes only) to state whether the carrier v4 port allocation looked block-based. Doc-only; no code.