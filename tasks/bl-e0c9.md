+++
title = "bz arm: a role whose provider is <row>@<client> makes its model call as an invocation on that machine"
created = 1794985623
updated = 1794986027
claimant = "Rabid-yog2"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["inference"]

[[blockers]]
id = "bl-d414"
on = "claim"
+++
What to build (REMOTE section 5.7: 'Selection rides in the row's name', 'The turn, in the order it happens', rule 'Nothing is retried on the wire'). One runner seam in the bz arm: a command line and a standard input in, three streams out, with two executors: linked brazen for a bare row (today's path, untouched) and the routed leg for a qualified one. Reading the name: match the suffix after `@` against the clients registered in THIS workspace (longest registered name wins; a client registered elsewhere is absent, and the name then falls to the linked brazen as an unknown row). Routed leg: deposit `invoke` for tool `bz` on that client with input {args: [--json, --provider, <row>], stdin: <request bytes>} through the gestures inbox, poll `capture` with `since`, write each appended piece to stdout as it arrives and stderr likewise. Never taken within the router's existing patience: emit one canonical error event (non-retryable kind) naming the client and its last_seen, then the terminator. Ended in doubt after bytes were relayed: add nothing, exit non-zero, so litany reads a half stream. Verify, and fix if false, that `invoke` cannot reach a client not registered in the calling workspace. The price table needs no change: confirm a qualified provider name prices by its own (provider, model) row. litany is not changed; if its config grammar or yog's block grammar rejects `@` in a provider value, that is this ball's finding to report before working around it.

Tested by: unit tests on the name resolution (bare, qualified, unregistered client, two candidates); an integration test driving a real turn with a fixture foot process on loopback whose bz entry is a script that writes canonical events slowly: response.json grows before the child exits, usage is recorded, the step commits; a stopped turn stops the polls; an absent client yields the error event. Amend REMOTE 5.7 and DESIGN 16.2 as landed.

Order: after the mailbox ball. The live two-machine proof waits for thrall's streaming release and brazen's envelope release; the fixture foot covers this ball's own gate.