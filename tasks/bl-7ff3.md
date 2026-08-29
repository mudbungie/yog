+++
title = "enforce the foot grade: read it off the leaf, mint it, and raise on it at the chokepoint"
created = 1787977716
updated = 1787977983
claimant = "OrderInverter"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-1dd3"
on = "claim"
+++
REMOTE §4.2 (bl-1dd3) rules the grade; nothing in the tree reads or writes it yet. Three halves, all small, all in yog:

1. **Read it.** `registry::leaf` walks the subject for the common name already; it must also yield the grade — a subject carrying `OU=foot` is foot grade, anything else is operator grade. Default-operator is load-bearing: a leaf minted before this existed must keep working, and a silently demoted seat would be an outage with no sentence.

2. **Mint it.** `src/wire/provision.rs` is the one openssl recipe; it needs a way to mint a foot leaf. `yog wire-certs` is the same recipe reached by a verb, so the flag surfaces there.

3. **Raise on it.** `ConsumerCtx::answer_as` already spends the client identity for scoping; a foot-grade caller sending anything outside {advertise, invocations, complete} refuses **in band, naming the grade** — not absent-shaped, per §4.2's reasoning (a foot learns nothing about the world from being told it is a foot).

The set is enumerated in code, never configured: adding an Action adds no row anywhere. That is what keeps this out of §11's per-verb ACL rejection.

Until this lands, §4.2 is a ruling with no enforcement and every leaf is operator grade in practice.