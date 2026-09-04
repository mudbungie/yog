+++
title = "probe the NAT classes before ruling on DHT-only: is the core's NAT (and the cellular path's) endpoint-independent for TCP?"
created = 1788232976
updated = 1788493053
claimant = "Spellbind-W"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Validation for REMOTE $13 as reworked (bl-aad5): with no carriage, punch viability IS reachability, and the question is empirical per NAT pair. Measured 2026-09-03. Every number below was observed; nothing is inferred from a datasheet. Addresses and device names are deliberately absent -- shapes only.

## What was measured, and how

Three paths. The laptop and the deployed engine box turned out to sit behind **one and the same residential NAT** (identical public v4, distinct external ports), so they are one NAT measured twice, from two hosts. The phone seat is the third.

- **Mapping class, UDP**: one socket on one bound source port, STUN binding requests to five distinct servers -- five distinct destination addresses across three distinct destination ports. Endpoint-independent iff the reported external port is the same for all five.
- **Mapping class, TCP**: three *concurrent* STUN-over-TCP connections from one bound source port (SO_REUSEADDR/SO_REUSEPORT) to three distinct servers, all held open together so the mappings coexist. Run twice, once from each host, with a fresh source port each time.
- **Port preservation**: whether the reported external port equals the bound internal port, per protocol.
- **Filtering class, TCP**: an outbound TCP mapping held open from a port, a listener bound to that same port, and then three independent third-party HTTP fetchers aimed at (public v4, that port) -- addresses the mapping had never seen. Endpoint-independent filtering iff the listener accepts.
- **Hairpin / simultaneous open**: synchronized-start TCP simultaneous open between the two residential hosts through the public v4 address, 20 s window, retry every 250 ms; and a UDP variant over 15 s.
- **v6**: global addresses, default v6 route, and an actual connect() to a public v6 address.
- **Phone**: the overlay's netmap endpoint set, RDAP on the prefixes it publishes, overlay ping (20 probes) from each residential host, and a shell hunt (adb connect, adb mdns services, TCP probes of six candidate ports over the overlay).

## The table

| | laptop (residential) | engine box (residential) | phone |
|---|---|---|---|
| UDP mapping | endpoint-independent (5/5 destinations, one external port) | endpoint-independent (5/5, and again 3/3 on a fresh port) | not measurable |
| TCP mapping | endpoint-independent (3 concurrent held connections, one external port) | endpoint-independent (3 concurrent held connections, one external port) | not measurable |
| port preservation | yes, external == internal, TCP and UDP | yes on a free port; **rewritten when the other host behind the same NAT already held that external port**, still constant across destinations | not measurable |
| TCP filtering | **not** endpoint-independent -- 3/3 third-party fetchers timed out, listener accepted nothing; host firewall ruled out (input policy accepts, only the overlay's interface chain installed) | same NAT | not measurable |
| port-mapping protocol | none (UPnP, NAT-PMP, PCP all absent) | none | n/a |
| routable v6 | **none** -- no global address, no default v6 route, connect() returns ENETUNREACH | **none**, same three ways | **two global v6 addresses**, prefixes attributed by RDAP to a US mobile carrier |
| cellular v4 | n/a | n/a | a 464XLAT translator address only (RFC 7335, not routable); **no carrier-observed public v4 endpoint in the published set** |
| hairpin | absent -- TCP simopen through the public address connected in neither direction over 20 s; UDP silent over 15 s | absent, same trial | n/a |

Direct-vs-relay, per pair:

| pair | result |
|---|---|
| laptop -- engine box | same LAN; direct, no punch involved |
| laptop -- phone | **direct**, on a residential LAN address -- a LAN path, not a punch |
| engine box -- phone | **direct**, on the same residential LAN address -- a LAN path, not a punch |
| phone-on-cellular -- engine box | **NOT MEASURED** |

## What could not be measured, and why

The phone was on the residential Wi-Fi for the whole window, so every direct path the overlay established to it is a LAN path and proves nothing about a punch. Its cellular interface was up -- the netmap endpoint set carries the translator address and the two carrier v6 addresses -- but the overlay's STUN goes out the default route, so no carrier-observed mapping was ever produced to read.

Getting the cellular measurement needs the phone off Wi-Fi *and* code running on it. There is no shell: every candidate port answers connection-refused over the overlay, `adb connect` is refused, and `adb mdns services` discovers nothing. So item (3) of the original ask -- the live TCP simultaneous-open trial between the two real endpoints -- is **NOT MEASURED**, and no number in this body is a phone-side NAT observation.

Also unmeasured: whether the residential NAT's TCP filtering is *address-dependent* (peer's SYN admitted after our SYN to that peer) or address-and-port-dependent. Distinguishing them needs a cooperating peer on a second external vantage, and there is none. It does not change the verdict -- both classes are exactly the case simultaneous open exists for.

## Punch verdict against $13.6's criterion

$13.6 fires when *a client the operator actually needs measurably cannot punch*. **The criterion is not met, and it is not refuted.**

The residential end -- the one $13.1 was least worried about and the one that carries the engine -- measures green on every property a punch needs: endpoint-independent mapping on both TCP and UDP, port preserved so the fixed punch port $13.2 publishes is the port a peer sees, and no reliance on a router favour. Its filtering is address-dependent, which is normal and is precisely what simultaneous open is designed to defeat; the two negative results (no hairpin, no unsolicited inbound) cost nothing the design was counting on, because $13.4's direct-address rung already covers two devices behind one NAT.

The cellular end is the one that could not be measured, so the verdict on the needed pair stays open. But what the phone *did* disclose sharpens the risk into something different from what $13.1 assumed. The risk is not "the cellular CGNAT might be symmetric". It is that **the two ends have no address family in common**: the cellular path is IPv6-native with IPv4 by translation and publishes no v4 address of its own, while the residential path holds no routable IPv6 at all. $13.3's v6-first rung -- the one that punches most reliably -- is dead on this pair, and it is dead on the *residential* side, not the cellular one. On v4 the engine has nothing to aim a SYN at, so simultaneous open degenerates to a single one-directional SYN from the phone whose fate rests entirely on the carrier's translator, which neither end can observe or influence.

The actionable consequence: **the cheapest intervention is not standing up $13.6's relay -- it is routable IPv6 on the residential side.** That single change turns the risk case into the easy case $13.3 already describes (a stateful v6 firewall on one end, a v6-native carrier on the other, no port rewriting anywhere). Standing up carriage should wait on that being tried and on the one trial nobody has run: the phone off Wi-Fi, with code on it, punching at the engine.

REMOTE $13 amended in the same change: $13.2 (the observed port is a separate fact from the local one -- measured), $13.3 ("both ends" is the whole of the v6 rung and the residential end is the one that fails it), $13.5 (the no-carriage cost restated against measurement), $13.6 (evidence status of the criterion), and a new $13.8 recording the measurements.