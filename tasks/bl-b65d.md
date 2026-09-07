+++
title = "a routed tool with no command line is opaque and held on every call: capability.yaml rules: gains a row keyed on the host-qualified name, and the hold names it as the way out"
created = 1788744427
updated = 1788744621
claimant = "Cantaloups-Y7"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r2"]
+++
Child 3 of the MCP bridge design (thrall DESIGN §6, thrall bl-3b03; REMOTE
§5.4 as amended by the sibling yog design ball). One lane-day.

PREMISE, verified against main: `src/control/classify/routed.rs` sends a name
outside the intrinsic map to the bash ruleset when its input carries a
`command` field and to `Effect::Opaque` otherwise; the shipped table
(`judge.rs`) holds opaque. A pinned MCP tool arrives as `<client>_<tool>`
(`tool_host/loaded.rs`, `presented()`) with an input shaped by the server's
schema — `{"url": ...}` for fetch — so every call of it is held, and the only
standing answer today is `table:` / `opaque: pass`, which reopens the whole
class bl-72bd closed.

CONTRACT

1. `capability.yaml`'s existing `rules:` grammar admits a routed tool's
   host-qualified name as the "program": `rules:` / `  box2_fetch: open-world`.
   `classify/routed.rs`, on an input with no command line, consults the policy
   rows with the request name as the one word before answering opaque —
   operator rows first, first match wins, exactly as a bash line's leading
   word is matched. A row's class is the operator's own statement of what
   that tool on that box reaches, informed by what `thrall mcp pin` printed
   (the server's annotations) and by nothing the wire carries. Host-qualified
   because the same server on two boxes is two trust decisions (REMOTE §5:
   locality rides in the name). No new key, no new file, no new class.
2. A routed tool whose input DOES carry a command line keeps bl-72bd's
   answer: classified by the line. A name row does not outrank a shell's
   own line; it is consulted only where there is no line to read.
3. The hold sentence for an opaque routed tool (`control/hold.rs` and the
   park the operator reads) names the one way out that is one: the row to
   write, spelled with the actual host-qualified name and the class words the
   file accepts — the `NOT_A_REMEDY` discipline of bl-68e1, from the other
   direction.
4. VISION §4.11 item 1's routed bullet gains the sentence; DESIGN §8.6 /
   §12's `policy` row says the ruleset is keyed on a routed name as well as
   on a program.

Tests, each failing on the old tree: a routed name with a policy row
classifies to the row's class; the same name with no row stays opaque; a
routed name with a `command` field ignores the row and classifies by the
line; the hold text carries the name and the row spelling. The
`classify/tests.rs` fixture list already carries a `litany-tool-deploy` name
to extend.

Refusals: no `effect` field on the advertisement (REMOTE §5.1); no class
inferred from a server's annotations by the engine; no per-name grant path
(bl-7fc8 stands — this is a class the operator states, adjudicated per
invocation, not a name allowed).