+++
title = "/enroll's ':0' refusal prints a wire-certs remedy that always refuses on the box it is printed for, and omits the restart the new address needs"
created = 1788235121
updated = 1788235517
claimant = "Forge"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["wire"]
+++
The out-of-box path to a working install runs through `/enroll` — it is the only
in-boundary way to get a tool host, and without one the engine can execute
nothing. On a box that has only ever booted `yog`, `/enroll` refuses:

    127.0.0.1:0 names no port a device can dial: a `:0` is a request the
    listener answers in RAM, and its answer changes at every boot. State the
    endpoint — `yog wire-certs WIRE_HOST=<host> WIRE_PORT=<port>`

The diagnosis is exact and the remedy is right in kind. The remedy as printed
does not work. Run it and you get:

    yog wire-certs: <dir> already holds material; rotating distrusts every
    certificate already issued. Re-run with FORCE=1 if that is what you mean.

…because the engine's own boot minted the material this refusal is about
(bl-ae05). Every box that can produce the first sentence is, by construction, a
box the second sentence refuses. The operator is handed a command that is
guaranteed to fail on the one machine it was printed for.

There is a third step too, and neither sentence mentions it: the address is read
at bind time, so the running engine keeps its `:0` listener until it is
restarted, and an `/enroll` retried against the live process before that restart
answers with material naming a port nothing is listening on.

So the real ladder is three rungs and the boundary teaches one:

    FORCE=1 WIRE_HOST=<host> WIRE_PORT=<port> yog wire-certs
    restart the engine
    /enroll <name> foot

## Why it matters more than a wording nit

This is the first refusal an operator meets on the way to a usable install, and
the drive's whole workflow is behind it. A refusal that names a remedy is the
house standard; a refusal that names a remedy which cannot succeed is worse than
one that names none, because the operator spends their trust on it first.

## Shape of the repro

    XDG_DATA_HOME=<scratch> yog &            # boots, mints 127.0.0.1:0
    yog gesture --ws <ws> '/enroll foo foot' # refuses, prints the remedy
    yog wire-certs WIRE_HOST=127.0.0.1 WIRE_PORT=<port>   # refuses, exit 1

## The fix

The `:0` refusal is the only site that knows the box already holds material —
it is refusing *because* of what the boot minted. Its sentence should name the
command that works from here (`FORCE=1` in front) and say that the engine must
be restarted before the new address is bound. Both facts are known at the
refusal; neither needs a new mechanism.