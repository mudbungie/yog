+++
title = "nothing is refused because a machine is busy: a read aimed at a working machine queues one ask behind its job, and a machine heard from a moment ago is there"
created = 1795322544
updated = 1795322544
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The rule: nothing is ever refused because something is busy. Busy means queue. Refusal is for what is not there or not allowed.

Three corrections to the listings and the named model read:
- a read aimed at a busy machine deposits one ask in that machine's mailbox, collected at its next read; the answer lands in the mailbox's remembered answer, so a seat that asks again is answered. The same read is never in flight twice for one machine.
- a registered machine that spoke within the thirty seconds a job is believed alive is there, so the instant between its last post and its next read no longer hides it.
- every refusal whose reason is that something is working is audited, and the rule is stated once in REMOTE.

What cannot be said without a wire key: a read with no earlier answer has no ok-shaped reply meaning queued, so the state is carried by the reply's sentence.