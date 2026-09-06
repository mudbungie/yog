+++
title = "the same-box seat still cannot be provisioned from anything the suite says: bl-e058 closed the engine half, and the seat's refusal names no remedy"
created = 1788673626
updated = 1788673665
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 install lane (STORIES S0/S9). This is the half of bl-e058 that its close did not reach.

## What bl-e058 fixed

Its journal: *"the boot names the address it bound on stderr; the :0 default and
the address file are unchanged"*. Confirmed working:

    $ yog
    yog: wire: listening on 127.0.0.1:39353

## What still fails

A stranger with one box follows lernie's README, which states the seat reads
`<data root>/wire/` holding `ca.pem`, `client.pem`, `client.key` and `address`,
and that certificates arrive by the operator's hand. The only hand available is
a copy out of the engine's own wire directory:

    $ yog &
    $ mkdir -p <lernie-data>/wire
    $ cp <yog-data>/wire/{ca.pem,client.pem,client.key,address} <lernie-data>/wire/
    $ lernie workspaces
    (this box's own engine)
        <lernie-data>/wire/address names 127.0.0.1:0 — a kernel-chosen port only
        that engine's own window is told; a seat wants a stated address

The refusal is accurate and names no remedy. The two remedies that exist are
each undiscoverable from here:

1. Hand-edit `address` to `127.0.0.1:<port from the stderr line>` — but the
   stderr line is gone if the engine was backgrounded by a supervisor, and the
   port changes at every boot, so the edit must be redone every restart.
2. `FORCE=1 WIRE_HOST=127.0.0.1 WIRE_PORT=<n> yog wire-certs` — a full rotation.

Route 2 is the only stable one, and it is a rotation: `wire-certs --help` says
so itself (*"a rotation distrusts every certificate already issued"*). The
non-destructive spelling does not help — on a box that already holds material,
a stated host re-issues the server leaf and says so explicitly:

    $ WIRE_HOST=127.0.0.1 WIRE_PORT=7740 yog wire-certs
    yog wire-certs: re-issued the server leaf over the CA already in <dir>
      … <dir>/address is unchanged — it names the one endpoint the engine binds

So there is no act that states an address on an already-booted box without
distrusting every leaf on it.

## Expected

One of:

- The refusal (engine-side or seat-side) names the act: *state an address with
  `FORCE=1 WIRE_HOST=… WIRE_PORT=… yog wire-certs`, or write `<host>:<port>`
  into `wire/address` and restart the engine*.
- `wire-certs` gains a spelling that writes `address` alone, over the CA already
  there — the same shape `WIRE_LEAF` already has (an act over an existing trust
  root, standing outside the rotation guard). Stating where the engine binds
  distrusts nothing.
- The boot writes the bound port somewhere a local seat reads. bl-e058 weighed
  this ("Write it") and single-source-of-truth argued against a second address
  file; that argument stands, which is why the two above are preferred.

## Severity

p1. This is the entire single-box install — the default shape of the Settler
story — and it dead-ends on the fourth act with no way forward that any shipped
page names. Neither yog's README nor lernie's carries the same-box recipe at
all: lernie's unprovisioned refusal names `yog wire-certs WIRE_LEAF=<name>`,
which is the VISITING-box recipe, and mints a leaf registered in no workspace
(see bl-6b14, bl-bd48).

---

The image route reproduces this and adds a second turn of the screw. `make image` builds and `image-scan` passes both directions (43.9 MB, 603 authored paths; the self-test catches a layer secret, an ENV secret and an undeclared binary). A bare `podman run --rm -v <state>:/state/yog:Z yog:<v>` then boots correctly and prints `yog: wire: listening on 127.0.0.1:38113` — but that is loopback inside the container network namespace, on a kernel-chosen port, so it is reachable from nothing at all: no seat outside the container, and no `-p` mapping is even expressible for a port that is not known until after the bind. A containerized engine is therefore unusable until an address is stated, which is this ball. `make deploy` handles it (the unit and deploy.env state one), but the README section "The image" shows the bare `podman run` with no mention that the engine it starts can be dialled by nobody.
