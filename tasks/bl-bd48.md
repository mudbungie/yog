+++
title = "a workspace created from the engine's own box is invisible to that box's seat, and no verb registers an existing leaf: wire-certs mints a window leaf that /enroll cannot adopt"
created = 1788673440
updated = 1788674297
claimant = "Cantaloups-Y2"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 swdev lane. yog 0.0.38 and lernie 0.1.30, both main tips, one box, isolated root.

The sequence is every step a first-time single-box install takes, in the order the READMEs give them:

1. `WIRE_HOST=127.0.0.1 WIRE_PORT=<p> yog wire-certs` — mints ca/server/client/window leaves and the address.
2. `yog` — boots, prints `yog: wire: listening on 127.0.0.1:<p>`.
3. Provision the seat by hand: `ca.pem`, the window leaf as `client.pem`/`client.key`, and `address`, into `<lernie-data-root>/wire/`. `lernie entries` then names the address, so the channel is up.
4. `yog gesture --ws swdev '/prepare'` — creates the workspace. `yog gesture '/workspaces'` lists it.
5. `lernie workspaces` — verbatim:

       (this box's own engine)
           {"kind":"workspaces","ok":true,"rows":[]}

   and every scoped verb refuses:

       {"error":"unknown workspace \"swdev\" — none is enumerated here","ok":false}

The mechanism is REMOTE §4 working as written — 'A workspace created over the wire auto-registers its creating client' — and step 4 was made by the reserved `local` identity, so nothing registered the window leaf. `<yog-state-root>/clients/` holds only the enrolled foot.

What makes it a defect rather than operator error is that there is **no way back**. `/enroll` is the only registrar and it MINTS: it will not adopt the leaf `wire-certs` already wrote, and its own page says a second enrollment under one name is refused because the certificate is still there. So the recovery is to enroll a second identity and overwrite the seat's material with it — which is what this lane did (`/enroll swdev-seat`, then copy cert/key/ca/address over the window leaf; `lernie workspaces` then answers in full). The window leaf minted in step 1 is now dead weight that nothing can register.

Expected, one of: (a) `wire-certs`' window leaf auto-registers in every workspace on the box that issued it — it is that engine's own material, minted from its own CA, and §1.5's separation argument is about leaves carried to other boxes; or (b) a gesture that registers an existing common name into the focused workspace, so `/enroll` stays the minting act and registration stops being welded to it; or (c) the refusal names the cause and the cure, since `unknown workspace — none is enumerated here` describes a workspace that does not exist, not one this leaf cannot see.

Severity p2: recoverable, but the recovery is undiscoverable from the refusal and leaves a stranded leaf. A first-time operator reads it as 'the engine did not create the workspace' and re-runs `/prepare` from the seat, which works, so the trap only bites the person who used the local `yog gesture` the README documents.