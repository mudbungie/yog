+++
title = "a deposit records no client identity: every seat's message is 'from: user' and the ops trail has no client field, so a shared workspace cannot say who said what"
created = 1788673800
updated = 1788673800
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 multitenant lane. REMOTE §4.1 (the identity is the leaf's common name),
§5 (the roster renders registered clients).

## Gesture

Two seats with two leaves, `seat1` and `seat2`, both registered in one
workspace, both depositing into one conversation within the same second —
seat2 a `message`, seat1 an `interrupt`:

    $ S2 lernie message alpha SlipperAcorn "Also cover the 1983 redefinition."
    $ S1 lernie interrupt alpha SlipperAcorn "STOP. Forget the metre. …"

## What the record says

Both landed, in order, and both are in the transcript. The raw frontmatter of
every delivered entry, from either seat:

    ---
    from: user
    deposited_at: 2026-09-06T03:37:52Z
    ---
    Also cover the 1983 redefinition.

    ---
    from: user
    deposited_at: 2026-09-06T03:37:52Z
    ---
    STOP. Forget the metre. …

The ops trail beside them records `argv`, `cwd`, `origin` and `exit` — and no
client field:

    {"argv":["litany","message","<workspace>","<agent-id>","Also cover …"],
     "origin":"conversation","exit":0,…}
    {"argv":["litany","stop","<workspace>","<agent-id>"],…}
    {"argv":["litany","message","<workspace>","<agent-id>","STOP. Forget …"],…}

## Why this is a defect

The engine knows the identity at every one of these acts. It reads the common
name off the presented leaf, it narrows the whole published derivation by it
(`Snapshot::scoped`), and it prints it in the client roster — `lernie clients
alpha` correctly listed `seat1`, `seat2` and `phone-seat` with live presence at
the same moment. Then it drops the name at the deposit, which is the one place
a person later has to reconstruct what happened.

The two facts are already separated correctly (advertisement durable, presence
live). The missing one is a third and it is the most durable of the three: WHO
performed this act. It changes at the rate of the act, not of the connection,
so nothing about the presence reasoning argues against writing it.

## Expected

A deposit carries the depositing client's common name — the seat's own
gestures included — and the ops trail rows name it too. `local` is already
reserved for the certificate-less in-world callers, so the vocabulary exists.

## Severity

p2. It is the defect that makes a multi-seat workspace not actually
multi-seat: the entire point is that somebody else is also working, and the
record is silent about them. It is also unrecoverable after the fact — no
later ball can reconstruct who deposited a line that was written `from: user`.