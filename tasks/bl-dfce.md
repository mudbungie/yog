+++
title = "the compactor's procedure pair routes to the tool host under the inverted seam, and the engine cannot decide where they belong"
created = 1788060201
updated = 1788068253
claimant = "OrderRouter"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Raised from the engine side while landing the seam inversion (engine ball bl-a00a, the half REMOTE §5's "the engine's driver keeps no local executor" asks for). The engine's router is now total: while a host injection is installed it answers **every** tool invocation and no binary resolution stands behind it. That is the ruling, implemented.

**The consequence nobody had priced.** The engine has a second source of injected tool definitions besides the host: the calling role's own **procedure**. Today that is exactly the compactor pair — `write_summary` and `mark_for_deletion` — which the compaction role calls as ordinary tool_use blocks and which the engine answers through its own front door. Under the inversion those two reach the executor like every other name and are therefore handed to yog's router.

They are not tools on a machine. `write_summary` writes the conversation's summary onto the compactor branch; `mark_for_deletion` nominates that same conversation's files. Both act on world state the server holds — REMOTE §5.4's "yog exposes engine acts only ... agent lifecycle: start a conversation, deposit, interrupt, stop, compact, fork". Compaction is on that list. Shipping them to a thrall would send a machine that does not hold the world a request about it, and REMOTE §5 already forbids the adjacent move ("the server's composed world fold is never shipped across the wire").

So a yog that installs the inverted seam and enrols one thrall would route `write_summary` at the first compaction, and the thrall would refuse it in band — compaction fails, correctly per the pipeline and wrongly per the intent. Nothing regresses until the engine pin is bumped; this is the question that bump has to answer first.

**Three candidate resolutions, none of them the engine's to pick unilaterally.**

1. **The host answers the pair itself, as engine acts.** yog recognises the two names in its router and performs them against the world it already holds — no wire, no thrall. Cheapest, and arguably just naming what §5.4 already says. The cost is that the router then has two classes of name inside it, and "front door only" has to be read as governing execution on a *machine* rather than every answer the router gives.
2. **Procedure injection is exempted from routing, in the engine.** The engine knows which injected definitions came from the procedure rather than from the host, and could answer those itself. Costs a second road — but one decided by the engine from a fact it owns, never by a host declining a name, which is the failure mode the inversion existed to close. Needs a REMOTE §5 amendment: "every tool call the agent makes takes the same road" would become "every tool call on a machine".
3. **The pair stops being executor-shaped.** Compaction's two tools become part of the compaction act rather than tools resolved through the three hops. Largest change, entirely inside the engine, and the one that leaves no carve-out anywhere.

**Why this is filed here and not decided there.** The invariant being amended is REMOTE §5's, which is this document's to amend, not the engine's. The engine has recorded the open question verbatim in its own `docs/DESIGN_TOOL_INJECTION.md` §7 and shipped the inversion green; the tree it ships is unaffected, because the exec binding installs no injection and its compaction path is untouched.