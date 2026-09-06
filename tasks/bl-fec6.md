+++
title = "the enroll envelope carries the engine's own wire/address verbatim, so a device that reaches the engine by any other route is enrolled pointing somewhere it cannot dial"
created = 1788673813
updated = 1788675046
claimant = "Cantaloups-Y2"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 multitenant lane, REMOTE §8.4 (the QR envelope).

## Gesture and reply

    $ lernie enroll alpha phone-emu operator
    phone-emu — operator at <loopback>:<port>
    [QR]
    not written down anywhere — scan it now, or enroll again

and the envelope itself (read out with `lernie ask`) carries an `address` field
holding the engine's `wire/address` verbatim — loopback, here. `enroll` takes
no address and there is no flag for one.

(Literal quads are omitted throughout: the disclosure gate refuses a routable
address and is right to, since no rule can tell one quad from another.)

## Why that is the wrong address for the device being enrolled

The device being enrolled is by definition not this box, so the address it must
dial is by definition not necessarily the one this box wrote for itself.
Concretely in this lane: an Android emulator reaches its host only through the
emulator's own host alias, never through loopback, so the enrolled device came
up dialling itself. The same holds for a phone on the LAN and a box on an
overlay.

`yog wire-certs` already understands this — `WIRE_HOST` is a comma-separated
LIST so a box "reachable by name, by overlay address and on the LAN says so
once, every entry rides the server leaf" — and the server leaf minted for this
run does carry both the loopback and the alias as SANs. So the TLS half is
ready for a second route and the envelope is not: `address` takes the first
entry alone and `enroll` copies that one.

Worked around with an adb reverse tunnel, which exists on an emulator and on
nothing else.

## Expected

Either `enroll` takes the address the device will dial (an argument, not a
setting — the same reasoning `make deploy-phone ADDR=` already carries in the
android repo), or it offers the SAN list off the server leaf, which is exactly
the set of routes the operator already stated when minting.

## Severity

p3. There is a workaround for any device you can reach a shell on, and the
material can be hand-edited after it lands. But enrollment is the one place the
design most wants to be a single scan, and today a scan is only correct for a
device that shares the engine's own loopback view.