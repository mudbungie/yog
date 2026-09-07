+++
title = "a PROTOCOL bump publishes ahead of its consumers, so every release window leaves the suite un-composable: today crates.io holds engine 15, foot 14, seat 13"
created = 1788745785
updated = 1788745841
claimant = "Cantaloups-Y8"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r2"]
+++
Round-2 install lane, clean-room container (ubuntu:26.04, rustup, `cargo install <crate> --locked` for all five published crates, nothing else).

VERSIONS INSTALLED FROM crates.io, 2026-09-06: brazen 0.0.17, litany 0.0.11, yog 0.0.49, thrall 0.0.12, lernie 0.1.34.

WHAT WAS DONE, exactly the yog README "The wire: this box, and the next one" recipe for a same-box seat:

    WIRE_HOST=127.0.0.1 WIRE_PORT=<port> yog wire-certs
    yog &
    cp <yog-data>/wire/{ca.pem,client.pem,client.key,address} <lernie-data>/wire/
    lernie workspaces

VERBATIM:

    (this box's own engine)
        wire protocol mismatch: this seat speaks version 13, the engine speaks 15.
        There is no negotiation — upgrade the older component until both speak one version.

And the foot leg, provisioned from `/enroll f1 foot`'s own envelope:

    home: wire protocol mismatch: this foot speaks version 14, the engine speaks 15.
    There is no negotiation — upgrade the older component until both speak one version.
    thrall: home: wire protocol mismatch: this foot speaks version 14, the engine speaks 15. …
    EXIT=1

So on a genuinely clean box, following only the shipped READMEs, a stranger who installs the current published suite can connect NEITHER a seat NOR a foot. There is no version combination available on crates.io today that composes.

WHY THIS IS FILED HERE AND NOT AS A NINTH PIN BUMP. The same defect has now been filed eight times across two repositories and closed eight times: thrall bl-e0f0 (2 vs 8), bl-f88f (8 vs 13), bl-f0b1 (duplicate sighting), bl-dc5f (13 vs 14, whose body records that bl-f88f "went stale within the hour"); lernie bl-d774 (2 vs 4), bl-e6ee (4 vs 5), bl-675e (5 vs 6), bl-2604 (7 vs 8), bl-249b (8 vs 13). Each close is correct and each is stale by the next engine release. The recurrence is the finding: a per-instance re-vendor ball is a fact about one afternoon, and the class is that yog publishes the number and its consumers learn it afterwards.

Round 1 had the same headline (thrall bl-f88f, three independent lane sightings). Round 2 re-drives it and finds it worse — the spread is now three-way rather than two-way, and it has moved onto the seat leg, which is the ONE leg a first-time user must cross.

WHAT IS NOT BEING ASKED FOR. Not negotiation: fail-closed on mismatch is right and the refusals are among the best sentences in the suite (fact, both versions, the one remedy). The question is what makes the pin stop going stale — a shared corpus crate the three depend on rather than three vendored copies, a release train that will not publish an engine whose consumers have not re-vendored, or an engine that accepts a stated window of older protocols. That is an architecture ruling, not a bump.

SEVERITY. p1: it is the first act of the Settler and Stranger stories (STORIES S0/S9) and it fails at published versions on a clean box, with no workaround a user can find. Everything downstream of the handshake in this lane had to be driven on hand-built binaries pinned to one protocol.

Main tips at the time of the drive: yog 5cd95986 = 16, lernie 1763302 = 13, thrall 034f6db = 13. lernie bl-183b (re-vendor the wire corpus) is open and ready and covers the seat half of today's instance.