+++
title = "WIRE_FOOT is the only way to mint a foot-grade leaf and no shipped page names it: wire-certs --help omits it, and the one in-binary statement is a line the founding mint prints once"
created = 1788673492
updated = 1788673492
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round 1, devadmin lane.

`yog wire-certs --help` (src/multiplex/help.rs) is one paragraph of about 400
words. It names `WIRE_HOST`, `WIRE_PORT`, `WIRE_DIR`, `FORCE` and `WIRE_LEAF`.
It does not contain the string `WIRE_FOOT`.

`WIRE_FOOT=1` is the only way `wire-certs` mints a foot-grade leaf, and
foot-grade is not a nicety: `thrall` "refuses to open at all on a certificate
that is not foot-grade" (thrall README), so an operator who follows the help
page carries an operator-grade leaf to the box and the foot declines it.

Where the fact IS written:
- the Makefile of the source repository (not shipped: `Cargo.toml`s `include`
  allowlist keeps `docs/**` and the Makefile out of the crate);
- `docs/REMOTE.md` 4.2 (same, not shipped);
- ONE line printed by `yog wire-certs` on the founding mint:

    issue another client with: WIRE_LEAF=<common-name> yog wire-certs, and a
    tool host's with WIRE_FOOT=1 beside that

That line is printed exactly once in a directory`s life — the founding mint —
and never again. Re-running to see it costs `FORCE=1`, which rotates the CA.
So for anyone who installed the crate and provisions a foot a week after the
first boot, the fact is unreachable from the binary.

EXPECTED: `WIRE_FOOT` in the `wire-certs` page beside `WIRE_LEAF`, in one
sentence saying what a foot leaf may do and that thrall refuses anything else.

While the page is open: it is a single unbroken paragraph carrying five
environment variables, two modes and three refusal conditions. Every other
help surface in this suite (thrall`s, lernie`s) is a usage line and a short
indented table, and reads far better. `yog wire-certs --help` is the page an
operator reads at the worst moment — provisioning a second box, out of channel
— and it is the least readable one.