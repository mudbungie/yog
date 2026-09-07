+++
title = "a routed tool with no command line is opaque and held on every call: capability.yaml rules: gains a row keyed on the host-qualified name, and the hold names it as the way out"
created = 1788744427
updated = 1788746123
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

---

Round-2 devadmin lane: the cost of this, measured on the three-box scenario, and the shape it gives the operator.

Two feet, deliberately different tool documents. `alpha2` advertises one shell-shaped `Bash`; `beta2` advertises a narrow admin set with no shell at all — `disk_usage`, `service_status`, `read_log`, each an executable reading its own small JSON object.

One goal spanning both machines ("compare disk usage and failing services across BOTH and write me one report"). Result: **11 holds, 11 operator releases, one per call**, over 11 steps and 319k tokens. Every single one was a `beta2_*` call. Not one `alpha2_Bash` call was held — a shell command line is readable, so `df`, `du`, `find`, `ps` and `swapon` all passed unattended.

The inversion is the finding. The box whose operator wrote three narrow, argument-checked, path-confined tools is interrupted on every call. The box whose operator exposed a raw shell runs unattended. The safer tool document is the one that is punished, and an operator who notices will fix it by adding a shell.

Two details for the `rules:` design:

- The reason sentence is otherwise excellent and names everything except the way out: `beta2_disk_usage {"path":"/"} classified opaque (beta2_disk_usage is not a tool this control implements and its input carries no command line, so what it reaches cannot be read — held rather than passed)`. It tells the operator what happened and gives them nothing to do about it beyond answering again.
- The hold is per CALL, not per tool: `beta2_read_log` was held six separate times in one conversation with six different paths. Whatever the row keys on, releasing once has to be able to stand for the tool on that host, or the ledger is the call count.

Same run under `claude -p` with podman exec standing in for ssh: same goal, 1m43s, one shot, zero interruptions.
