+++
title = 'a workspace made through the local yog gesture boundary is registered to nobody, so every seat answers "unknown workspace" and no verb can repair it'
created = 1788673482
updated = 1788673482
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round 1, devadmin lane.

GESTURE, on the box holding the engine:

    $ yog gesture --ws fleet "/prepare dir ~/proj/fleetops"
    {"kind":"prepared","ok":true,"prepared":{"binding":"~/proj/fleetops",...}}
    $ yog gesture /workspaces
    {"kind":"workspaces","ok":true,"rows":[{"workspace":"fleet",...}]}

From the seat on the same box, over the wire, with the local client leaf:

    $ lernie workspaces
    (this box's own engine)
        {"kind":"workspaces","ok":true,"rows":[]}
    $ lernie conversations fleet
    {"error":"unknown workspace \"fleet\" — none is enumerated here","ok":false}
    $ lernie enroll fleet devbox foot
    unknown workspace "fleet" — none is enumerated here

REMOTE 9.6`s scoping is doing exactly what it says: `Snapshot::scoped` filters
to the workspaces the calling common name is registered in, registration is
auto on CREATION, and the creator here was the reserved `local` identity, not
`yog-client`. On disk:

    <world>/state/yog/clients/yog-client/workspaces/ops     # made by the seat
    # nothing under clients/yog-client/workspaces/fleet

WHY IT MATTERS

`yog gesture` is documented as the control boundary and is the only interface
on a headless box with no seat yet. Everything it creates is invisible to every
seat that later dials in, and the workspace name is a GLOBAL namespace
(REMOTE 9.6), so the name is now taken by something no seat can see, reach,
rename or delete. `/delete-workspace` is a scoped gesture too.

There is no `register` verb. `/enroll` registers, but only as a side effect of
minting a NEW leaf, and it refuses when the name already has one.

EXPECTED, in preference order:
1. `local` creating a workspace registers every client that already exists, or
   the boundary refuses the create and says a workspace must be made by a
   client that can see it; or
2. a registration act exists — `/register <client> <workspace>` — so the
   operator on the box can hand an existing leaf a workspace it should see.

Severity p2 rather than p1 because the workaround (make workspaces from the
seat) is discoverable once you know; the refusal sentence gives no hint that
the workspace exists and the caller merely cannot see it, which is the part
that costs the hour.