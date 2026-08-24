+++
title = "DESIGN: a client holds many servers, and the client-side workspace is what names one — separate material, separate chats, separate everything"
created = 1787544372
updated = 1787544372
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design"]
+++
## The ruling to work through

**A client can be a client of MANY servers, and the thing that defines one is a
workspace.** Its own mTLS material, its own address, its own conversations —
separate everything. This is an operator ruling and the design's job is to make
it coherent, not to relitigate it.

The deliverable is an amendment to `docs/REMOTE.md` (the authority for the
wire, the client and the trust model) plus a proposed ball breakdown for the
implementation. Design produces a living document edited like code — not a
growing task body.

## What stands today, and why it does not survive the ruling

**Client material is one set per box.** A client reads its certificate material
from a single directory under the yog data root: one `ca.pem`, one `ca.key`,
one server leaf, one client leaf, one window leaf, and ONE `address` file
(`src/wire/material.rs`, `src/wire/provision.rs`). So a box can be a client of
exactly one engine. A machine that already runs its own engine has
self-provisioned a loopback root at boot (§8), so it cannot also be a client of
somebody else's engine without breaking its own window — both read the same
directory.

**The window cannot be pointed anywhere.** `Engine::window_seat`
(`src/engine/window.rs`) builds its seat as:

    crate::wire::client::Seat::open(&crate::wire::material::Material {
        address: crate::wire::loopback(&bound),
        ..material
    })

`bound` is the address this process's own listener actually bound, and
`loopback()` overrides whatever `address` says. A window is therefore always a
client of the engine in its own process. Nothing reads a stated address on the
way in and no flag names one.

**The word is already taken, and that is the crux.** §1.5 makes the workspace
the trust domain **server-side**: "A client registered in one workspace is
invisible in another — the corporate machine participates in the corporate
workspace without seeing the personal ones, and vice versa." One server hosts
MANY workspaces. If a client-side workspace also names a server, the noun means
two things, or it means one thing and registration is per `(server, workspace)`
pair. **Resolving this is the first job of the design** and everything else
follows from it. §2's noun table is where the answer lands.

## The reframe worth testing

"Local" should probably stop being a special case: the loopback engine bare
`yog` boots is just one entry among N, and the window picks one. A special case
is usually a missing reframe. If that holds, `window_seat`'s forced `loopback`
dissolves rather than gaining a flag, and both halves of bl-320b go with it.

## Questions the design must answer

1. **The noun.** Is a client-side workspace the same object as §2's workspace,
   or a second noun that CONTAINS a (server, workspace) pair? What does §1.5's
   invisibility rule mean once a client holds several? lernie bans "session"
   (DESIGN §1) precisely to avoid this collision — do not add one.
2. **Where material lives**, keyed how, and what happens to the single
   `address` file. One fact, one home.
3. **What becomes of `Engine::boot` on a window box.** Does bare `yog` still
   boot an engine? §8's "one world, one engine, whichever face started it" was
   decided for a box that is both server and seat; state whether it survives,
   and if it does not, replace the prose and cite this ball.
4. **What a window shows.** One workspace at a time, or a roster spanning
   servers? What §6 attention and the §11 surfaces mean across servers.
5. **Where a remote workspace's chats live.** DESIGN I0 is "disk is the bus"
   and the file religion is authoritative server-side (§1.1); a client holding
   no world for a remote workspace either caches or reads through. Say which,
   and what that costs the invariants.
6. **How material reaches a client.** §1.4 is absolute — bootstrapping is
   out of channel, forever, and yog carries no enrollment or pairing protocol
   in the channel. Whatever the operator does to add a server must stay an act
   performed ON the boxes.
7. **Migration.** A box that already holds one self-provisioned loopback root
   must keep working, and the path from it to the new shape must be stated.
8. **What this does NOT solve.** Attack the design before committing it, and
   fill the holes with a default or a principle rather than a feature.

## Constraints

House rules are not negotiable: no new dependencies without explicit approval,
the 300-line cap with a pre-split at 200, the 100% coverage floor, and the
style rules in CLAUDE.md. Prefer subtraction — a new flag, verb or config key
is a smell, and an existing explicit signal is better. Where the amendment
contradicts standing prose, REPLACE the prose and cite this ball; do not leave
two answers in the tree.