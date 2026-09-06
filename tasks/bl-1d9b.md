+++
title = """a second engine on one world does not exit when its listener cannot bind: two mailboxes mint colliding inv-N handles, so a routed tool call answers "not in flight" or hands back another invocation's capture"""
created = 1788673469
updated = 1788673469
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round 1, devadmin lane. Scenario: a foot (thrall in a container) enrolled in a workspace, an agent asked to administer that box through the routed tools.

WHAT HAPPENS

Start an engine on a world. Start a second `yog` on the SAME world while the
first still holds the port:

    $ XDG_DATA_HOME=$R/data XDG_STATE_HOME=$R/state yog
    yog: wire: bind <lan-ip>:7743: Address already in use (os error 98)
    $ echo $?            # never printed — the process does not exit
    # `timeout 8 yog` under the same env exits 124: still running.

It printed the bind refusal and then kept running. It has no listener, but it
still consumes the world`s `gestures/` inbox and still holds its own in-RAM
`registry::mailbox::Slots` — its own `seq`, its own `live` map, its own
presence map.

Two consumers now drain one inbox, so a routed tool call is answered by
whichever engine picked the envelope up. Three sequential invokes, one after
the other, from one shell:

    $ yog gesture --ws ops "/invoke box2 shell {\"command\":\"echo MARKER-1\"}"
    {"invocation":"inv-17","kind":"routed","ok":true}
    $ yog gesture --ws ops "/invoke box2 shell {\"command\":\"echo MARKER-2\"}"
    {"invocation":"inv-7","kind":"routed","ok":true}
    $ yog gesture --ws ops "/invoke box2 shell {\"command\":\"echo MARKER-3\"}"
    {"invocation":"inv-18","kind":"routed","ok":true}

inv-17, inv-7, inv-18. Two independent counters minting into one namespace.
Kill the listener-less one and the same four gestures answer inv-19, inv-20,
inv-21, inv-22.

WHAT THE AGENT SEES

A `Mailbox::post` on engine A and a `capture` poll answered by engine B is
`collect` on a map that has no such handle, so the model reads:

    box2_shell: no invocation "inv-5" is in flight; it was answered already,
    it expired, or this engine restarted since it was posted — the box may
    have run it; read the world before acting again

Seven of about seventeen routed calls in one conversation failed that way.

WORSE, AND THIS IS THE REASON FOR p1: when the id COLLIDES rather than
missing, the poll collects the other engine`s slot and the model is handed a
DIFFERENT invocation`s output as the answer to its call. Verbatim from the
transcript of one conversation:

    call:   box2_shell {"command":"free -h"}
    result: === queue.log ===
            queue: starting worker pool (4)
            ...
            === journalctl if exists ===
            /bin/sh: journalctl: not found

That is the capture of an EARLIER call — the one that had just been reported
as "no invocation inv-5 is in flight". The model then re-ran `free -h` and got
the real memory table. It also ran `dmesg | tail -50` and was handed the same
queue.log text. A tool result that silently belongs to a different question is
worse than a failure: the model reasoned on it and only caught it because it
had added an echo marker.

A THIRD SYMPTOM, same cause: presence is per-process RAM, so the roster read
answered by the listener-less engine reports every client "not connected right
now" while a `get` one second later, answered by the real one, says "connected
right now". Both sentences reached the model in one conversation.

EXPECTED

A `yog` that cannot bind its listener must exit non-zero naming the address —
it cannot be an engine, and a half-engine that still drains the world`s
gesture inbox is worse than no engine. Whatever the ruling on the exit, one
world must have exactly one mailbox: an `inv-N` handle is a promise that the
handle addresses one call.

SEVERITY p1: silent wrong answers on the routing leg, reachable by the most
ordinary operator accident there is (starting the engine twice — a supervisor
restart, a second terminal, a stale nohup). No error is printed anywhere after
the first line, and the world keeps working well enough to look healthy.