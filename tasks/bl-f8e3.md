+++
title = "the README never says how to install yog: cargo install appears nowhere, though the published crate installs clean from a bare container in under a minute"
created = 1788673635
updated = 1788673665
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 install lane (STORIES S0/S9).

## The gap

`README.md` has these top-level sections: Architecture, Running, The world,
Building and contributing, Delivery, Publishing. There is no Install section and
the string `cargo install` does not appear in the file. "Building and
contributing" is a Makefile target table addressed to a contributor, and the one
row that installs anything reads:

> | `make install` [`INSTALL_PREFIX=<p>`] | Release-build and drop `yog` into `$INSTALL_PREFIX/bin` (default `~/.local/bin`) |

— which requires a clone, a toolchain and a reader who has decided to
contribute. The "Running" section opens straight into `yog` / `yog gesture`
without ever saying where the binary comes from.

## Why it matters, given that the route works

Measured this round on a bare `ubuntu:24.04` with only `curl git build-essential`
and rustup:

    cargo install brazen --locked    41s
    cargo install litany --locked    39s
    cargo install yog    --locked    39s
    cargo install thrall --locked    10s
    cargo install lernie --locked    78s

All five succeed with **no system dependency beyond a C toolchain** — no
`pkg-config`, no `openssl` dev headers, no GL or font packages even for the
seat. The published route is in excellent shape and the README does not mention
it exists. litany's README, by contrast, opens with a four-route install table;
brazen's opens with `cargo install brazen`.

## Expected

An Install section near the top, naming `cargo install yog --locked`, the image
route (already documented, under "The image", 500 lines down), and the one host
prerequisite the container run proved is real — `git`, plus `openssl` on the
box for the wire mint the boot performs. The container image documents its own
runtime layer needing exactly those; a host install needs the same and says so
nowhere.

## Severity

p2. It is a doc-only fix on a working path, but it is the first thing a stranger
looks for and its absence reads as "this is not for you".

## Note

thrall's README has the same gap and is filed on its own board.

---

Doc rot in the same file, found while walking the image route: README "The image" shows `podman run … yog:0.0.5 gesture /attention`, many versions behind the tag `make image` actually produces (0.0.38 this round). Worth fixing in the same pass — an example naming a tag the reader does not have fails at the copy-paste.
