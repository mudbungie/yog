# yog — The Remote Seat (client/server split)

Status: normative for the split; direction adopted by operator ruling
2026-08-13 (bl-b9a2). §9 is the build sequence and each step lands under its
own ball; steps 1–5 are in the tree. `docs/DESIGN.md` remains the
architecture authority for the engine; this document governs the wire, the
client, and the trust model. Where the two collide, DESIGN wins until DESIGN is
amended. The same amendment doctrine applies here: prose is replaced, the ball
id is cited, the path to the ruling is not narrated.

---

## 1. The ruling

1. **The file religion is authoritative for the server side.** The engine keeps
   every DESIGN invariant (I0–I9): disk is the bus, the world converges over
   files, all durable state lives in the world. The server is the only process
   that touches the world.
2. **The UI operates entirely via RPC.** The window is a pure client of the
   control boundary (DESIGN §8.5). **One method, one channel: even the local,
   single-machine version does the hard split** — the window talks to the
   engine over the same wire a remote seat uses. There is no in-process face
   and no file-transport fallback for a seat; a second execution path would be
   a second implementation, which VISION §8 refuses.
3. **The channel is mTLS**, and the wrapper must be strong enough to leave on
   the public internet. Both ends authenticate with certificates.
4. **Bootstrapping is explicitly out-of-channel.** The starting assumption is
   an operator who administers both machines. Key and certificate provisioning
   is an act the operator performs on the boxes; yog carries no enrollment,
   pairing, or account protocol in the channel, ever.
5. **Client registration is per-workspace; the workspace is the trust
   domain.** A client registered in one workspace is invisible in another —
   the corporate machine participates in the corporate workspace without
   seeing the personal ones, and vice versa.
6. **Conversations are aware of registered clients and the tools they
   advertise.** Client presence and tool availability are rendered facts of
   the workspace and enter agent context.
7. **Agents run on the server, in the background**, independent of any client
   connection. Seats attach and detach; the work does not.

The canonical scene: a home server runs the engine and keeps every log; a
phone seat talks to a conversation; the conversation teleoperates a work
laptop through the tools that laptop's client advertises into its workspace.

## 2. Nouns

lernie bans "session" (DESIGN §1), and the transport/connection collision is
exactly the reason — the word appears nowhere here. The split adds three nouns
and reuses one:

| Noun | Definition |
|---|---|
| **client** | A machine holding an operator-issued certificate. One certificate = one client identity (its leaf name). A client is a fact about a machine, not a person — v1 has one human, and every certificate is operator-grade within its registrations. |
| **seat** | A client connection acting as an operator face: it asks queries, paints replies, dispatches gestures. (The word already names GUI/headless/line faces in DESIGN §8.5 — a remote seat is the fourth face of the same surface.) |
| **tool host** | A client advertising tools into the workspaces it is registered in. One client may be seat and tool host at once (the work laptop usually is). |
| **registration** | The durable fact that client C participates in workspace W. Server-side, in the world, a file — the file religion applies; the wire only ever transports the gesture that writes it. |

## 3. The wire is the boundary — and adds nothing to it

The protocol **is** the existing gesture surface: `Act(Action) | Ask(Query)`
in, `Reply` out, in the JSON serialization the codec already defines
exhaustively (`src/boundary/codec.rs`, strict decode, compile-gated). The wire
is a transport for that surface, not a vocabulary:

- **No wire-only verbs.** A capability a client needs and the boundary lacks
  is added to the boundary — every face gains it — never to the wire alone.
  One dispatch surface, N serializations, never two implementations (VISION
  §8).
- **Framing** *(decided and landed, bl-b6fa — `src/wire/frame.rs`)*: **a
  big-endian `u32` byte length, then that many bytes of JSON.** A request is
  one frame; an answer is **N ≥ 1 reply frames followed by a zero-length
  frame**, which is the terminator. Length-delimited rather than newline: a
  reader never scans, so no property of the *encoder* is load-bearing in the
  *framing*; the allocation is bounded before it is made, so a peer on the open
  internet cannot make a reader grow to meet it; and a zero-length frame is not
  a JSON value, so nothing a payload can say collides with the terminator.
  **The streaming form is not a second form.** Every answer is a stream, so a
  follow-class read is the general path with more than one frame in it — no
  flag, no version, no second reader, and nothing to add when the first
  follow-class `Query` lands. Today N is always 1, because none is
  follow-class: the seat polls (below).
- **`Reply` gains a decode side.** Today the reply codec is encode-only. The
  client is in-crate (§8), so the hand-codec discipline extends to decode;
  serde derive stays a non-dependency (existing stance, `src/ui_state`).
- **Cadence: the seat polls.** The window today adopts a snapshot per frame;
  a remote seat asks at human cadence and rides the follow stream for the hot
  path. A push/subscription channel is deliberately deferred (§10) — it is an
  optimization of the same surface, not a different one.
- **The disk inbox survives for in-world callers.** Agents drive yog through
  the `yog` PATH shim and the `gestures/` deposit inbox — same machine, same
  world, disk is the bus. The wire is for cross-trust-domain callers; the
  inbox is for the world's own residents. One boundary, two intakes, each the
  religion of its domain. (Certificates for every spawned agent would be an
  enrollment surface §1.4 forbids.) *(Landed, bl-b6fa: the two intakes are
  `yog gesture` and `yog seat`, and they share one argv reader and one
  answering function — `ConsumerCtx::answer` — so the second door opens onto
  the same room and can add no verb.)*

## 4. Identity, authentication, authorization

- **mTLS both ways.** The operator runs a private CA out-of-channel and
  issues the server certificate and every client certificate. Presentation of
  a valid client certificate is the entire authentication story: no
  passwords, tokens, or accounts in-channel, and therefore nothing in-channel
  to phish, rotate, or leak. An unauthenticated connection gets a TLS refusal,
  not a yog reply — the surface exposed to the open internet is the TLS
  handshake and nothing behind it.
- **Authorization is registration.** A connection may see and gesture into
  exactly the workspaces its client identity is registered in. Everything
  else is not "forbidden" — it is **absent**: enumeration replies simply do
  not contain unregistered workspaces, the same shape as a workspace that
  does not exist. Scope errors that confirm existence are a disclosure.
- **Unscoped gestures.** Reads enumerate the registered set. A workspace
  created over the wire auto-registers its creating client. The first
  registration on a fresh server is out-of-channel like the certificates: the
  operator writes the registration file on the box. (A special "first client"
  flow would be the bootstrap-in-channel §1.4 forbids; the general path with
  an operator-seeded file dissolves it.)
- **Revocation** is deletion: remove the registration (per-workspace) or
  distrust the certificate at the CA (per-client). A lost phone costs a
  certificate, never data — the client holds nothing durable (§6).

## 5. Tools follow the client

- **Advertisement is durable; presence is live.** (Amended by bl-bc7c; the
  original ruled advertisements connection-scoped RAM, which put a
  connectivity-rate fact inside the model's cached context prefix.) A tool
  host presents its tool set — name, description, input schema — when it
  connects; the engine writes it into the client's registration (world, file)
  when it differs from what is stored. **Presence** — connected right now —
  is connection-scoped RAM. Two facts because two rates of change: a tool set
  changes when the operator reconfigures a machine; presence changes with
  every network blip.
- **Context is structurally append-only, so client facts are point-in-time.**
  The model's stable prefix carries exactly one client-facing surface: a
  **client-management tool**. Its operations: `list` — the workspace's
  registered clients and which are live, now; `get` — one client's detail and
  the tools it advertises. Every reply is a dated observation appended to
  context, free to go stale, never a prefix mutation — a presence flap
  cannot touch what the model already read, and the prompt cache (keyed on
  the prefix) survives every blip.
- **Loading is the agent's own point-in-time act.** From a `get`, the agent
  loads a client's tools; loaded definitions are callable from that turn on.
  Whether the driver re-declares them in the tool block (one deliberate,
  paid prefix rebuild at a moment the agent chose) or carries them
  append-only is an implementation choice at the lernie seam; the invariant
  is that nothing but an explicit load ever changes the tool surface.
- **Use is attempt.** A loaded tool, when called, is attempted: routed to
  the client if it is live, refused in-band if not — an error tool result is
  an appended message the model reacts to, never a prefix change. The same
  path corrects staleness: a client refuses a tool it no longer carries.
- The workspace surface renders its registered clients — present or absent —
  and each one's advertised tools, live: the seat sees the flap; the model's
  prefix does not.
- **Invocation path:** the agent's tool call hits lernie's tool seam →
  server-side adjudication (`yog tool-control`, unchanged, fails closed) →
  the engine routes the invocation down the tool host's live connection → the
  client executes locally → the result returns up the same channel. A routed
  invocation carries a deadline; a vanished client is a visible refusal, not a
  hang.
- **Honesty about containment:** execution happens on a machine the
  adjudicator cannot inspect. Adjudication judges the invocation exactly as
  today; any containment beyond that is whatever the client enforces locally,
  and the design must not claim otherwise.
- **The driver-side seam is a lernie ask.** lernie's driver executes tools;
  routing a designated tool to a remote executor needs an upstream seam. yog
  owns the registry, the advertisement, the routing, and the adjudication
  chokepoint. This piece gets its own design pass before its ball is filed.

## 6. What lives where

Everything durable is the server's: the world, the logs (`ops.jsonl`), the
UI-state documents, the agent processes, the LLM calls (brazen in the engine
process), the sentry, the pilot, the clock. A client holds exactly two things:
its key material (operator-provisioned, out-of-channel) and RAM. Losing a
client loses nothing; the server's disk remains the single history.

## 7. Per-seat UI state

`ui.json` today is one last-writer-wins document holding two kinds of fact.
The split cuts along that line:

- **Facts about the world** — seen watermarks, pins, acks — are operator
  facts, shared across every seat: one document, as today. (Attention answered
  on the phone must clear on the desktop; that is I0's whole point.)
- **Facts about the pane of glass** — panel sizes, collapsed sets, view knobs
  — become per-client documents keyed by client identity, held server-side so
  the client stays stateless and any two seats of the same client converge.

## 8. Shape of the code

One crate, one multi-call binary, as today: `yog serve` runs the engine (the
former `headless` boot plus the wire listener); bare `yog` runs the window. A
TUI or web seat later is another consumer of the same wire and needs nothing
new from the engine.

**`headless` is now `serve`, and the rename is the point** *(bl-b6fa)*. The
face did not change — it is still the one `Engine::boot` with no window — but
the engine now carries the listener, so what it is to anything off the box is a
server. Two spellings of one face is the drift the const exists to prevent, so
there is one word and §8's is it.

**The listener rides the ENGINE, not one face** *(decided, bl-b6fa)*. §8.5
already runs the deposit consumer on both faces, for I0's reason: *a deposit
converges whichever face is up*. A seat wants the identical guarantee, so the
listener boots beside the consumer in `Engine::boot`, and a windowed yog serves
the wire exactly as `yog serve` does. That decides the local boot question §9.5
was asked to settle, and it decides it by **dissolving it**: nothing spawns
anything, nothing refuses with a remedy, and there is no ladder — because there
is never a second engine to arrange. One world, one engine, whichever face
started it; every other seat is a client of that one. The alternatives were both
worse and both were considered: bare `yog` *spawning* `yog serve` gives one
world two engines (two pilots, two sentries, two derivation workers — the
instance-coordination shape DESIGN §14 rejects), and *refusing with a remedy*
puts a terminal instruction in front of a desktop launch that has no terminal.

**Absence is the off switch.** With no material provisioned there is no
listener and nothing is said about it — removing the directory deletes config,
not code (the severability test). A *half*-provisioned wire is warned about and
still does not listen, because silently degrading to no encryption is the one
failure this design exists to exclude.

**Certificates are minted out of channel, by `make wire-certs`** *(bl-b6fa,
`scripts/wire-certs.sh`)*. It shells to `openssl` — yog links no certificate
library and mints nothing itself, ever (§1.4) — writing a private CA and a
server/client leaf pair into `<yog-data-root>/wire`, plus one `address` file.
`WIRE_HOST`/`WIRE_PORT` name what the server binds and a seat dials; the SAN
and the address file are derived from the same `WIRE_HOST`, because two
spellings of one host is the drift §8's name resolution removed from the
boundary. It refuses to overwrite: a rotation distrusts every certificate
already issued, so it is `FORCE=1` and never a silent re-mint. Test material is
minted the same way at test runtime (`src/test_support/wire.rs`) — **a
certificate fixture is never committed**, which `make leak-scan` would refuse
anyway and which would be a private key in a public repository whether or not
it guarded anything.

**The material sits BESIDE the world, not inside it** — `<yog-data-root>/wire`,
the sibling of `<yog-data-root>/world`. The world subtree is a generated
artifact yog seeds, wipes and reseeds; key material is operator-provisioned and
irreplaceable by anything yog can do. Nesting it under a directory yog rebuilds
would make a reseed a revocation.

**The address is one fact with one home.** A server binds `address` and a local
seat dials it; a seat on another machine has its own `wire/address` naming the
server it belongs to. There is no second file and no flag — and no server-name
knob either, because the name a client verifies is read off the address it
dialled (an IP literal is an IP identity, anything else a DNS one).

**Paths never cross the wire.** *(Landed, bl-f5f6.)* Boundary types addressed
workspaces and projects by absolute `PathBuf`; across machines those are
meaningless and a disclosure besides. The wire spelling is now the **name**, on
the types themselves — `Action`, `Query`, the nested payloads they carry
(`monitor::Verb`, `fleet::Verb`, `fan::Obligation`, `config::ConfigFile`,
`start::Payload`, `start::Prepared`), the line's `Context`, and the two `Reply`
fields that identify rather than locate. The engine resolves a name to a path at
the dispatch chokepoint. Four rulings came out of it and belong here rather than
in a ball body.

- **The name of a workspace is its directory leaf; the name of a project is
  derived.** The rule differs because the nouns do (`src/naming`). §3.1 already
  says a workspace's leaf *is* its name and §3.2 already makes that same leaf
  the `--as` identity every ball claim is stamped with — so there is nothing to
  invent, and **a foreign workspace needs no special case**: lernie's
  `workspaces/`/`replays/` leaves are the auto-ids the tab strip already paints
  as their identity. Two roots holding one leaf is a world whose §3.2 join is
  already ambiguous — both would claim `--as home` — so the resolver refuses
  naming the token instead of inventing a disambiguator for a world that is
  broken one level down. A **project**, by contrast, has no name at all: its
  identity is the decoded balls invocation path (§5.1 #1) and two checkouts of
  one repo legitimately share a basename, so its name is the shortest trailing
  run of components no other enumerated project shares — the basename wherever
  that is already unique. The §11 roster label is that same name, elided, so
  what the operator reads off the left panel is the word they may type at
  `--project`.
- **The resolution is one function, at the chokepoint, ahead of the table.**
  `Action::workspace()` / `Action::project()` / `Query::workspace()`
  (`src/boundary/address.rs`) are tables *on* the enums; `dispatch` and `answer`
  each resolve once, before their match, so no arm re-derives an address and an
  unresolvable name refuses — naming the token, the codec's own strict-decode
  discipline — before anything runs. A gesture that names no workspace resolves
  to nothing and no arm reads it: the general path with no input, not a case of
  its own. The mapping is read backwards at exactly one place too — the frame
  spells `Snapshot::ws_name` / `project_name` where a seat's *selection* becomes
  a gesture (`AppModel::line_context`, the click-glue), over the same enumerated
  sets the resolvers read. Two directions, one mapping.
- **A reply speaks the name where it IDENTIFIES and the path where the path
  IS the answer.** `WsRow` identified a workspace by carrying the whole
  `Workspace` — path, kind and, for a named one, a `name` beside the path. It
  now carries the name and the kind: §3.1 makes the leaf the name, so the field
  that was already the identity became the whole of it and the path went. A
  field whose *subject* is a filesystem location, by contrast, keeps path
  semantics, because answering it by name would answer a different question:
  `Reply::Applied { file }` (what was written), `Reply::Marks { space }` (which
  balls space the branch is a branch of — the operator reads it to tell a
  project's board from an agent's own universe), the worktree paths in
  `WorkDiff`/`Files`, and `Prepared::binding` — lernie's `--cwd`, minted by the
  engine and handed back to the engine verbatim by a seat that never reads it.
  Those remain absolute paths on the wire and they are a residual, not a
  ruling: they are unhelpful to a client on another machine and they disclose a
  home root. Narrowing them — an opaque handle where the client only relays,
  a workspace-relative path where it renders — belongs with the transport that
  makes the distance real (§9.5), not with the addressing.
- **`Prepared::name` is gone, not migrated.** It carried the §3.2
  `--as`/`YOG_NAME` stamp beside a `workspace` path. Once `workspace` became the
  name, §3.1 and §3.2 made the two fields one string twice — so the second went
  rather than being kept in step.

## 9. Build sequence

Each step is its own ball; boundary-surface work serializes (shared-surface
merges cost more than the work). Steps 1–4 are pure boundary-completion,
valuable with no network at all — they finish VISION V5 teleop parity.

1. **Boundary-complete the reads.** *(Landed, bl-6233.)* The
   transcript/inspector surface read disk on the frame thread and had no
   boundary spelling — the chats themselves were unreachable by any face but
   the window. Six Query/Reply pairs now cover it: `Transcript`, `Steps`,
   `Step`, `Files`, `Rail`, `Inbox`, each addressed by workspace + agent
   (name-based addressing is step 3, not this one), in all three
   serializations. Two rulings came out of it and belong here rather than in
   a ball body:
   - **The in-flight tail is folded, not dropped.** A live model call's tail
     is the snapshot's own `Stream`, which the `Deps` already carry, so
     folding it costs no read. Answering the committed half alone would have
     been cheaper and wrong — the window folds it, so a headless seat that
     did not would describe a different moment than the GUI does of the same
     instant, which is the divergence §3's "one dispatch surface, N
     serializations" exists to prevent.
   - **The pin is not answered.** A pin is a viewport fold, and DESIGN §8.5
     files folds under views, so `Rail` answers the notches and every
     inspector read answers unpinned — the same ruling `Conversations`
     already makes by answering the all-collapsed list.
2. **`Reply` decode**, with encode↔decode round-trip tests over the whole
   reply surface. *(Landed, bl-7067.)* Strict, on the gesture codec's own
   terms: an unknown `kind`, a missing field and an unknown token each refuse
   naming the offender, and the refusal `{"ok": false, "error": …}` reads back
   as the `Err` side. The discriminant is `kind`, **not** `ok` — a captured run
   spells its own exit verdict there, so a `bl close` that failed its gate is
   `ok: false` and is an answer, not a refusal. Three rulings came out of it:
   - **Round-trip failures were fixed in the ENCODING, never in the test.**
     Four facts left the window and never reached a headless seat: the search
     answer's own needle (bl-648a's whole point), the conversation row's depth,
     tone and standing alignment verdict, the display-only rung, and the §3.5
     figure's four token counters and its attribution class. Each is now on the
     wire. The reply file has no consumer that a widened object breaks — its
     readers parse JSON forgivingly — and the alternative was a codec that
     narrowed the answer silently.
   - **Derived text rides beside the fact, never instead of it.** `usd` beside
     `micro_usd` was already the tree's shape; the attribution clause and the
     fleet label now follow it, and the decode reads the fact and re-derives
     the text.
   - **Help resolves against the seat's own roster.** A `HelpRow` is four
     `&'static str`s out of a `const` table and no decoder can mint one — but
     help's subject is *the interface, not the world*, so the roster is an
     answer every seat already holds. An unknown verb refuses, naming it.
3. **Name-based addressing:** `PathBuf` leaves the boundary types.
   *(Landed, bl-f5f6 — the rulings are in §8.)*
4. **The shell paints only boundary payloads.** *(Landed, bl-1eb0 — see §9.4.)*
5. **The wire.** *(Landed, bl-b6fa — see §9.5.)* The mTLS listener, the seat
   transport, the framing, the certificate bootstrap, and `yog seat` — the
   wire's first shipped consumer. The deposit inbox remains for in-world
   callers (§3).
6. **Registration and scoping:** the per-workspace registry, reply filtering,
   auto-registration on create, per-seat `ui.json` split (§7).
7. **Tool hosts:** advertisement, rendering, routing, the lernie seam —
   after its own design pass (§5).

### 9.4 Where orchestration stops and paint begins

The line, stated once because §9's step 4 is where it had to be drawn:
**`AppModel` may orchestrate; what crosses into paint is a payload the wire
could carry.** Orchestration is holding the focus, resolving it against the
published snapshot, memoizing a heavy build per derivation, handing a search
off and painting whatever landed — none of it crosses the boundary, and all of
it stays. What went is the frame deriving *out of* `GitTree`: names, marks,
liveness, descent and verb gates folded on the render thread from the engine's
own agent set, which is a thing no client will ever hold.

Mechanized as `rules/no-engine-tree-in-paint.yml` over `src/shell/**` plus the
three modules that render rather than derive (`inspector`, `composer`,
`inboxview`). The test is **not which module a type lives in** — it is whether
a `Reply` can say it: `AgentState`, `AgentMark`, `Flight` and `Tone` ride the
wire today and are lawful in paint; `GitTree`, `Agent` and `CommitNode` ride
nothing. Every other module under `src/` is engine and folds the agent set
freely, because folding it is what the server does.

Four rulings came out of it.

- **The fat `Agent` gets a projection, not a diet.** `Query::Agent {workspace,
  agent}` → `Reply::Agent(AgentView)` is the family's seventh member, sharing
  the six inspector reads' address and envelope: the selection's id, its §2.3
  conversation root and the ancestor chain above it, the §3.3 name with its
  addressability rung, the tip the §5.1 #17 governing derivation takes, its own
  §3.5 liveness, the §6 marks it wears with the parked invocation's own
  sentence beside them, the §5.1 #28 class in flight anywhere in the
  conversation, and the four §8.2 gates (present, nudgeable, stoppable,
  children). Every field is a fold the boundary already owned; nothing was
  derived here for the first time. That is the shape the rest of §9 wants:
  **what was missing was never the derivation, it was the spelling.**
- **A gate that is not derivable from a row goes ON the row.** `ConvRow` gained
  `stoppable` and `stop_children`. Neither follows from what it already
  carried: `state` is the badge aggregated over the whole subtree, so a quiet
  root with a working child paints Live and has no driver to kill, and the
  cascade's membership is the Stop menu's looser prefix test rather than the
  strict §5.1 #8 descent `direct` counts. Deriving them per row against a
  roster the seat had to be holding is what the ban forbids; deriving them once
  where the row is built costs one pass instead of one per row.
- **A name is resolved against a table, not a tree.** Painting a *third
  party's* name — a deposit's sender, in the two seats mail appears in — was
  `display_name_of` over `&[Agent]`. The ladder did not move (§3.3 still has
  one home); its input narrowed to `nav::convs::Titles`, id→title, which is
  buildable from the engine's agent set **or** from a conversations reply's own
  `root_id`/`display` pairs. The second constructor is the claim made
  executable: a face holding replies and no world resolves the same names.
- **A residual is named rather than papered over.** The model picker's
  config-lineage tip crosses as `model_pick::ConfigTip` — two strings, plainly
  wire-carryable — but it is handed over by an `AppModel` accessor and has no
  `Query` of its own. The §11 picker family (lineage browse, `providers.yaml`
  reads at an oid, the write claims) is not wire-complete, and saying so is
  worth more than a query minted to close a checklist. It belongs with §9.5's
  transport, beside the path-typed reply fields §8 already lists as residual.

### 9.5 The wire, and what it does not yet carry

Landed (bl-b6fa): `src/wire/*` — the framing (§3), the two rustls
configurations, the engine's synchronous accept loop, a seat's transport, the
material reader, and `yog seat`. `rustls` is a direct dependency by operator
ruling; **no tokio** (the listener is `std::net` and blocking threads, so
AGENTS.md rule 8 stays vacuous) and **no rcgen** (§8's make target). Four
rulings are in §3 and §8 rather than in a ball body: the framing, the
listener riding the engine, the material's home, and the address as one fact.

**What the wire adds to the boundary: nothing** — and that is structural, not
a promise. A connection's request goes to `ConsumerCtx::answer`, the same
function the gestures inbox calls, which decodes with the one codec and runs
the one `dispatch`/`answer`. The listener never sees an `Action` or a `Query`,
so there is no place a wire-only verb could be added.

**The residual, stated plainly: the window is not a client yet.** §1.2 rules
that the UI operates entirely via RPC and that even the local case does the
hard split. What landed is the channel, the server and a *terminal* seat; the
window still holds the engine it serves. So the arrangement today is
**server-and-seat in one process**, not seat-over-wire — which is the honest
half of §1.2, because the half that is missing is the **read path**, and it is
the larger half:

- A frame paints the published snapshot (§7.2), not replies. §9.4 finished the
  *taxonomy* — paint may name only what a `Reply` can say — but the delivery is
  still an in-process derivation, and a window pointed at a foreign engine
  would have to derive a world it does not have.
- Per-seat UI state (§7) is undivided: `ui.json` is still one document holding
  both operator facts and pane-of-glass facts, so two seats of one client have
  nowhere to converge.
- The path-typed reply fields §8 lists as residual are still absolute paths —
  `Reply::Applied { file }`, `Reply::Marks { space }`, the worktree paths, and
  `Prepared::binding`. The transport that makes the distance real now exists,
  so narrowing them has a consumer to be narrowed for; it did not land here.
- One connection per gesture, and no follow-class `Query`. The framing carries
  a chunked stream already (§3), so the live tail needs a query, not a wire
  change.

Registration, scoping and reply filtering are deliberately **not** here: a
connection is trusted at certificate grade this step, and per-workspace scoping
is §9.6 (bl-8bbc). Until it lands, a valid client certificate sees the whole
world — which is the correct posture for a one-operator box and the wrong one
for the second human, exactly as §1.5 says.

## 10. Open questions (living)

- ~~The follow/streaming frame shape~~ — settled by bl-b6fa (§3): every answer
  is a frame stream terminated by a zero-length frame, so a follow-class read
  is the general path with more frames. What remains open is **when polling
  graduates to a subscription channel**, and beside it whether a seat should
  hold one connection across gestures rather than dialling per ask (today it
  dials; the seat polls at human cadence, so the connection cost is not yet a
  cost).
- The tool-advertisement schema (the exact shape of name/description/input
  schema). How availability is spelled is settled (§5, bl-bc7c): definitions
  frozen in the prefix, presence answered at invocation.
- Whether pins are world facts or pane facts (§7 defaults them to world).
- Certificate hygiene: lifetime, rotation cadence, whether the CA distrusts
  or registrations carry the whole revocation load.

## 11. Rejections

Recorded so they are not relitigated:

- **A second in-process face for the local window** — the split is the point;
  one method, one channel (§1.2).
- **Any in-channel authentication besides the client certificate** —
  passwords, bearer tokens, OAuth flows: each is an enrollment or secret
  surface §1.3–§1.4 exist to exclude.
- **An in-channel pairing/enrollment protocol** — bootstrap is out-of-channel
  by ruling; convenience flows reopen the exact surface mTLS closed.
- **A separate client crate or binary for v1** — the multi-call single binary
  stays (DESIGN §0); a foreign-language client is a future consumer of the
  wire, not a reason to split the crate now.
- **Syncing or mounting the world at clients** (network filesystem, rsync,
  shared checkout) — the world never leaves the server; clients receive
  replies, not files. Two full engines over one networked disk is the
  instance-coordination shape DESIGN §14 rejects, kept rejected.
- **Per-tool or per-verb ACLs in v1** — registration is workspace-grade
  trust; a finer policy layer is speculative until a second human exists.
