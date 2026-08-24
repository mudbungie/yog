# yog — The Remote Seat (client/server split)

Status: normative for the split; direction adopted by operator ruling
2026-08-13 (bl-b9a2). §9 is the build sequence and each step lands under its
own ball; steps 1–8 are in the tree, and §9.7 records what step 8 opened and
what it left. `docs/DESIGN.md` remains the
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

   **Executed rather than aspirational, by operator ruling 2026-08-14
   (bl-ae05): the local window is a wire client of localhost.** The real
   socket, the real handshake, the real certificate — everything through the
   front door. The cheaper option §9.7 offered, an in-process *transport* to
   the one `Answerer`, was weighed and declined, so §11's rejection of a second
   in-process face stands unlifted and nothing in-process was added. What the
   ruling costs is §1.4's *absence is the off switch*, and §8 records how that
   is paid: the engine's boot performs the same out-of-channel mint the
   operator's own act performs, on the operator's own box, before anything has
   been dialled. Nothing crosses a wire unauthenticated, and yog still links no
   certificate library.

   **Closed to the letter for the window's acts (bl-1747).** The reads crossed
   in bl-ae05/bl-adcb/bl-f297 and the acts in bl-4841/bl-1747; with the last
   four gestures posted, `AppModel::dispatch` — the frame's own way into the
   chokepoint — is deleted, and the only callers of `boundary::dispatch` left
   are the engine's own intakes. The residual is on the read side and is scope
   rather than architecture (§9.7): three standing classes, each blocked on
   named work, none on a decision.
3. **The channel is mTLS**, and the wrapper must be strong enough to leave on
   the public internet. Both ends authenticate with certificates.
4. **Bootstrapping is explicitly out-of-channel.** The starting assumption is
   an operator who administers both machines. Key and certificate provisioning
   is an act the operator performs on the boxes; yog carries no enrollment,
   pairing, or account protocol in the channel, ever.
5. **Client registration is per-workspace; the workspace is the trust
   domain — at both ends** *(amended bl-aaec)*. Server-side as always: a
   client registered in one workspace is invisible in another — the corporate
   machine participates in the corporate workspace without seeing the
   personal ones, and vice versa. Client-side the same boundary is now
   material: a box's participation in a workspace held elsewhere is its own
   directory of channel facts (§8.2) — its own mTLS material, its own
   address, its own conversations — so what a box can even *ask about* is
   separated the same way what a server will answer is.
6. **Conversations are aware of registered clients and the tools they
   advertise.** Client presence and tool availability are rendered facts of
   the workspace and enter agent context.
7. **Agents run on the server, in the background**, independent of any client
   connection. Seats attach and detach; the work does not.
8. **A client is a client of many servers, and the client-side workspace is
   what names one** *(operator ruling 2026-08-24, bl-aaec)*. The unit of
   participation is the workspace, never the server: a box holds one entry
   per workspace it participates in elsewhere (§8.2), each with its own
   material and its own address — and "the server" is nothing but the address
   an entry names. The loopback engine bare `yog` boots is the same shape
   seen from the window's side — one more channel, whose address is learned
   in RAM rather than read from a file (§8) — so the roster a window paints
   spans engines without a client-side "server" object ever existing.

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
| **registration** | The durable fact that client C participates in workspace W, on the server that hosts W. Server-side, in the world, a file — the file religion applies; the wire only ever transports the gesture that writes it. |
| **entry** | A workspace, held from the box that participates in it (bl-aaec): a directory under that box's `wire/workspaces/<leaf>/` carrying the channel facts that reach it — the host engine's anchors, this box's leaf and key for it, the host's address, and the name the workspace bears there (§8.2). The entry is the client's half of the pair registration is the server's half of — possession, where registration is permission — exactly as a channel needs both a certificate and its issuer's trust. |

Multi-server adds **no noun** (bl-aaec). A workspace is one word at both
ends — the trust domain, the unit of conversations — and "entry" above names
a *spelling* of it, not a second object. The collision that would have minted
one (a client-side workspace that names a server, while a server hosts many
workspaces) dissolves because the client-side unit is the (server, workspace)
participation and never the server: nothing client-side enumerates a server
or holds a fact about one, so there is nothing for a second word to name. Two
entries naming one address are two trust relationships that happen to
terminate at one listener.

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
  follow-class `Query` lands. That promise was tested and held by bl-73e7's
  `Query::Follow`, the first read whose N is greater than 1: no flag, no
  version, no second reader were added. What the engine *did* have to change is
  that `Answerer::answer` hands back a lazy iterator rather than a `Vec` — a
  materialized answer must be finished before its first frame can be written,
  and a read that answers as the world changes never finishes. That is a
  signature, not a protocol.
- **`Reply` gains a decode side.** Today the reply codec is encode-only. The
  client is in-crate (§8), so the hand-codec discipline extends to decode;
  serde derive stays a non-dependency (existing stance, `src/ui_state`).
- **Routing does NOT invert the ask, and that is the ruling** *(decided,
  bl-c907; landed, bl-024b — §5.3)*. §5 has the engine route an invocation *down* a
  tool host's live connection, which reads like the one place a server must
  speak first. It is not, and making it one would cost the property this whole
  section exists to keep. **The tool host stays the asker**: it rides a
  follow-class read for its next invocation — the general path with more frames,
  which the framing already carries — and posts each capture back as an ordinary
  act on the same surface. Nothing about the framing, the listener or a client's
  socket posture changes; the engine's *work* flows down a stream the client
  asked for.

  The alternative was weighed and refused. A true inversion — the server writing
  a request frame down an established connection — makes the channel
  bidirectional-symmetric: a second reader at both ends, a correlation id
  because two request streams now share one socket, and an `Answerer` **on the
  client**, which is a second dispatch implementation. VISION §8 refuses that,
  and §3's own "one dispatch surface, N serializations" refuses it here. The
  long poll buys the same capability with the readers that already exist.

  What it costs is a held connection and a thread parked on it per tool host,
  which is exactly the case the bullet below leaves open ("when a seat's ask rate
  exceeds human cadence") arriving from the other side: a tool host is not
  polling faster than a human, it is waiting. And it is the **first**
  follow-class read to have a consumer, which §10 named as the gate on minting
  one. The verbs it needs are boundary verbs like any other — a read that drains
  this client's pending invocations, an act that posts a capture — so every face
  gains them and the wire still adds nothing.

  **As built (bl-024b) it is four verbs, not two, and the second pair is the
  reason.** A tool host's two are what this bullet describes: `invocations`, the
  follow-class read, and `complete`, the act. But the *asking* side needed two
  as well — `invoke` and `capture` — because the engine's intake is **one
  thread for the whole world** (§8.5's deposit consumer), so a gesture that
  waited for a tool to finish would stop every other deposit converging for as
  long as the tool ran. `invoke` therefore queues and answers a handle at once,
  and the waiting is a poll the *driver* does, in a child process that has
  nothing else to do and is the only party that knows how long its call is
  worth waiting for. The follow-class read blocks a **connection** thread, which
  is a thread per tool host and nothing else's.
- **Cadence: the seat polls, and rides a follow stream for the hot path.** A
  seat asks at human cadence over the standing set; the two surfaces that change
  faster than an operator looks are held reads it makes — a tool host waiting
  for work (`Query::Invocations`, bl-024b) and the streaming transcript tail
  (`Query::Follow`, bl-73e7). Both are asks, so **the engine never speaks first**
  and nothing here is a push channel: a push/subscription channel stays
  deliberately deferred (§10, §11), being an optimization of the same surface
  rather than a different one.
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

### 4.1 The registry, as landed (bl-8bbc)

**A registration is a file, and its existence is the fact.**

```text
<yog-state-root>/clients/<client>/pane.json          the §7 pane-of-glass facts
<yog-state-root>/clients/<client>/workspaces/<name>   one empty file per registration
```

Registering writes the file, **revocation deletes it**, and the registered set
is the directory listing — nothing stored that the path already says. It sits
at yog's own state root beside `ui.json`, not under `wire/`: `wire/` holds what
yog can never mint (§8) and this holds what only yog ever writes. The operator's
bootstrap is therefore `mkdir -p` and `touch`, which is the same out-of-channel
act §1.4 already requires for the certificates.

**No gesture manages registrations, and none was added.** §3's ban is absolute
— a capability that exists on the wire and nowhere else is forbidden — and the
whole management surface here is the operator's own file acts, which are the
same out-of-channel acts §1.4 already requires. Seating a *second* client in a
workspace is one `touch`; de-scoping one is one `rm`. The read side is §5's
client-management tool, and that belongs to step 7 with the advertisements it
lists. Minting a boundary verb before that step would be a verb with one caller
and no face.

**The identity is the certificate's subject common name** (§2), read off the
presented leaf by a structural DER walk — yog links no certificate library.
A fingerprint would have been cheaper and wrong twice over: unreadable in a
`clients/` listing, and changed by a renewal, silently de-scoping every
registration the operator wrote. `local` is **reserved** for the
**certificate-less in-world callers** (§3) — the `gestures/` deposit inbox and
`yog gesture` — which hold no certificate but do own a pane document; a
certificate claiming it is refused on the same rule that refuses `.` and `..`,
so the reservation is one rule and not three special cases.

**The window is no longer among them** (bl-ae05). §1.2's ruling gives it a leaf
of its own, `yog-window` (`registry::WINDOW` — one const, spent by the mint that
puts the name on the certificate and by the seating that writes its
registrations), so it is identified exactly as a phone seat is: by the common
name on the certificate it presented. It is scoped like any client, it appears
in its own workspaces' rosters, and its §7 pane document keys on that leaf. The
reservation narrowed rather than dissolving, because the deposit inbox and
`yog gesture` still hold no certificate and still must not be scoped — each
intake the religion of its domain.

**Seating the window is the engine's own act, and it is the general path.** §4
already has the first registration on a fresh server performed out-of-channel by
the operator; this is that act, performed by the engine for the one client it
*is*. The asker seats `yog-window` in every workspace the published derivation
enumerates, on every pass — idempotent, one directory read when nothing is new —
so a workspace founded while the window is up is registered within one cadence
period. No create to detect, no bootstrap flow, and the same `mkdir`/`touch` an
operator performs for a remote client, performed by the process that already
holds the enumeration.

**Scoping is one filter, not twenty checks.** The engine narrows the published
derivation (`Snapshot::scoped`) to the client's registered workspaces before
anything runs, at the same chokepoint §8's name resolution already stands at.
That is what makes absence *structural*: the roster maps the narrowed set, and
`ws_path` resolves over it, so an unregistered name earns the resolver's own
`unknown workspace "x"` — the identical bytes a name nobody ever founded earns.
There is no scope branch to write, so there is no scope error to leak. The
narrowing is exactly the workspace-keyed facts, because §1.5 makes the workspace
the whole trust domain and §11 keeps finer policy rejected.

**Auto-registration needs no create-detection.** Under scope a gesture can name
only a workspace the client is registered in — or one it just founded, which is
the single case the resolver could not resolve and the raise founded anyway. So
a *successful* answer naming a workspace outside the scope is, by construction,
a creation, and seating the client in it is the general path rather than a
branch.

**The raise, and what it costs.** `Action::Prepare` names a workspace that need
not exist: the resolution falls back to `<names-root>/<name>` (§3.1) and
`Step::EnsureWorkspace` founds it — which is what the window has always done by
handing `prepare` a path directly, and what no other seat could do while the
resolution refused every name the enumeration lacked. It can **found or resume**,
never join: a directory already at that path that IS a workspace (§3.1 — it
holds `repo.git`) refuses with the resolver's own sentence, so a create is
never a way into a workspace the scope hides.

**Since bl-6c9e that refusal is about SCOPE and nothing else.** The enumeration
the resolution reads is now the live one (§8), so "exists but is not in my set"
no longer includes *the wall this very caller founded a millisecond ago* — the
case that made a second `Prepare` refuse, and made every non-`Prepare` gesture
naming the newborn refuse with it, since no raise fallback covers a gesture that
founds nothing. What the guard still catches is another client's workspace hidden
by scope — and only that, since bl-c9d2: a directory that is not a §3.1
workspace at all is a birth that died between making the directory and making
the marker, enumerated by no root and inside nobody's scope, and refusing it
wedged the name forever behind an addressing sentence. The raise resolves past
it and the idempotent ensure's own `lernie new` decides — finishing the dead
birth, or refusing in lernie's own words with a logged ops row.

That refusal is the one place existence is observable to a scoped client, and
the ruling is that this is acceptable and bounded: **a namespace with creation
*by name* cannot also make a name's availability unknowable**, and §4 chose
creation. What a collision reveals is that a name is taken. It reveals no
workspace's contents, conversations, clients or registrations — everything §4
makes absent stays absent.

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
  append-only was left to the lernie seam; the invariant
  is that nothing but an explicit load ever changes the tool surface.
  **Settled by bl-c907 (§5.2): re-declared.** The set is a durable document the
  injection reads at every assembly, so the rebuild happens once, at the step
  after the load, and the agent chose it.
- **Use is attempt.** A loaded tool, when called, is attempted: routed to
  the client, and refused in band when it cannot be — an error tool result is
  an appended message the model reacts to, never a prefix change. The same
  path corrects staleness: a client refuses a tool it no longer carries.

  **Presence is NOT the routing predicate, and that is bl-024b's amendment.**
  This bullet used to say *"routed to the client if it is live"*. It cannot be:
  a tool host dials per ask (§10) and holds a connection only while it is
  *waiting*, so it is absent for the whole time it is executing something — and
  a presence test would therefore refuse the second call of a busy host, which
  is the one host that is certainly there. The queue is the predicate instead:
  an invocation waits in the client's mailbox, and what makes a vanished client
  visible is the **caller's own deadline**, in band, never a hang. What *is*
  checked at the invoke is the staleness correction above — that the named
  machine advertises the named tool right now.
- The workspace surface renders its registered clients — present or absent —
  and each one's advertised tools, live: the seat sees the flap; the model's
  prefix does not.
- **Invocation path:** the agent's tool call hits lernie's tool seam →
  server-side adjudication (`yog tool-control`, unchanged, fails closed) →
  the driver queues the invocation in that client's engine-side mailbox → the
  tool host, waiting on its follow-class read, is handed it → the client
  executes locally → the client posts the capture back as an ordinary act →
  the driver's poll collects it. A routed invocation carries a deadline; a
  vanished client is a visible refusal, not a hang. **Nothing in that path is
  the engine speaking first** (§3).
- **Honesty about containment:** execution happens on a machine the
  adjudicator cannot inspect. Adjudication judges the invocation exactly as
  today; any containment beyond that is whatever the client enforces locally,
  and the design must not claim otherwise.
- **The driver-side seam was a lernie ask, and it landed.** lernie's driver
  executes tools; routing a designated tool to a remote executor needed an
  upstream seam. It is lernie 0.0.9's `Fx::tool_injection` (its
  `docs/DESIGN_TOOL_INJECTION.md`): **one** object carrying both halves — the
  definitions prompt assembly and the grant gate read, and the router the
  executor consults ahead of binary resolution — so a tool declared and not
  permitted, or permitted and not declared, is unrepresentable. An injected
  name outranks an elected one, the per-invocation disk record stays the
  executor's, and adjudication is untouched. yog owns the registry, the
  advertisement, the routing and the adjudication chokepoint; §5.2 is what it
  filled the seam with.

### 5.1 The advertisement, as landed (bl-4e08)

**The gesture is a boundary verb, and the wire adds nothing** (§3). `advertise`
carries one field, `tools`, an array whose element is exactly three facts:

```json
{"op": "advertise", "tools": [
  {"name": "Bash", "description": "run a command",
   "input_schema": {"type": "object", "properties": {"command": {"type": "string"}}}}
]}
```

- **`name` is a single path component.** It is the handle the later load act
  addresses one tool by, and a name carrying a separator is a name that
  addresses a filesystem.
- **`description` is one string**, the tool host's own words.
- **`input_schema` is the JSON Schema, verbatim.** yog neither validates nor
  rewrites it: it is the host's statement to a model, and any narrowing here
  would be yog inventing a contract it does not own.

Nothing else. There is no version, no enable flag and no per-workspace list —
each would be a fact yog stores and cannot check.

**It names no client, and that is the gesture.** The identity a set lands under
is the **intake's** — the connection's certificate common name, read exactly
where §4's scoping reads it. A `client` field on the wire would let any
connection overwrite any other's set, which is the authorization the certificate
has already decided. An intake carrying no client identity — the `gestures/`
deposit inbox and `yog gesture` (both `local`, §4.1; the window carries a leaf
of its own since bl-ae05 and is not among them) — **refuses in
band**, with a sentence: a caller who typed this at a terminal made a category
error worth naming, not an authentication failure worth hiding. The threading is
the act-side twin of `ConsumerCtx::answer_as`: one `caller` on the dispatch
`Deps`, carrying who is asking and who else is connected.

**A name collision inside one client's set declines loudly**, naming the token;
a collision *across* clients is legal and ordinary — two laptops both offering
`Bash` — and disambiguating them belongs to the act that loads one.

**The document.** One file per client, beside its §7 pane document:

```text
<yog-state-root>/clients/<client>/tools.json   the advertised set, one JSON array
```

One document per **client**, not one per registration: a tool set is a fact
about a machine, and §2's registration listing already says which workspaces see
it, so writing the array under every registration would be one fact stored N
times. Its element spelling **is** the wire's — one encoder, spent by the codec
and the file alike, because a stored set and a presented one spelled twice drift
within a week. The engine writes it only when it differs from what is stored, so
a re-presentation on every reconnect touches no mtime. An unreadable or
undecodable document reads as the empty set, which is also what a client that
has never advertised reads as — no reader carries two cases.

**Presence lives in RAM and has no leave verb.** The listener takes a
connection-scoped guard the first time a connection names its client and drops
it when that connection ends, however it ends; the map is a **refcount** per
identity, because one client may hold two seats and the second closing must not
unsay the first. The `Mutex` sits in `src/state.rs` (the crate's lock
chokepoint) and the operations beside it in `registry::presence`.

**The roster is the read.** `Query::Clients {workspace}` →
`Reply::Clients([{client, present, tools}])` joins the three at the moment it is
asked — the §4.1 registration listing, the presence map, each client's stored
set — so nothing is cached and a flap needs no invalidation. It is scoped like
every other read: an unregistered workspace earns the resolver's own
`unknown workspace "x"`. The window paints those same rows from the same
function (`registry::roster`), memoized per derivation with the live set in the
key. The §5 **client-management tool** the model reads is a separate surface and
is not this — it is §5.2, and it is built on this read.

### 5.2 The tool, the load, and the client's own config (bl-c907)

**One tool, named `clients`, three ops.** Its subject is the roster, which is
why it is one tool rather than one per op — and why loaded remote tools still
surface as individually named definitions of their own. lernie's
`docs/DESIGN_MCP_BRIDGE.md` §6 ruling binds a host too: a generic
`call {client, tool, arguments}` would collapse the role grant, the grant gate,
the tool control and every future policy into one bit. The declared schema:

```json
{"name": "clients",
 "input_schema": {
   "type": "object",
   "properties": {
     "op": {"type": "string", "enum": ["list", "get", "load"]},
     "client": {"type": "string"},
     "tools": {"type": "array", "items": {"type": "string"}}},
   "required": ["op"]}}
```

- **`list`** — every client registered in this workspace, which hold a live
  connection *right now*, and how much each advertises.
- **`get {client}`** — one client's detail: its presence, each advertised tool
  with its description, and the name that tool would become callable under.
- **`load {client, tools}`** — those definitions become callable from the next
  step on.

Every answer opens with the instant it was observed at, because presence is true
only then and a line that did not say when it was read would be a claim about
now. Every refusal is in band and non-zero: an unknown op, an identity this
workspace has not registered (**absent**, §4 — the same sentence a name nobody
seated earns), a tool the client does not advertise, an engine that did not
answer. Nothing here ever mutates the prefix.

**A load resolves whole or not at all.** Every named tool must be advertised
right now; one miss refuses the whole act, because a partial load leaves the
model believing it holds a tool it does not.

**The presented name is `<client>_<tool>`, always.** §5.1 leaves the
cross-client collision — two laptops both advertising `Bash` — to the act that
loads one. Prefixing *conditionally* would make a tool's own name depend on what
some other machine advertises, so a name the model already learned could change
under it; one rule and no case. A composed name a provider's tool block would
refuse declines at the load, naming it.

**The loaded set is durable, and its document is**

```text
<yog-state-root>/loaded/<workspace>/<agent>.json
```

Durable because a driver is not: each step is a fresh process, so a set held in
RAM would be unloaded by the next hop and "callable from that turn on" would be
false. It sits under yog's own state root rather than in the workspace, which is
the conversation's git repository.

**The definition is frozen at the load act**, name, description and schema
together, and re-read from that document at every assembly — never from the
client's live `tools.json`. That is §5's own rule (bl-bc7c) rather than a
shortcut: a prefix that changed when a client reconnected would put a
connectivity-rate fact inside the model's cached context, which is the defect §5
was amended to remove. The staleness freezing admits is corrected where §5
already corrects it — *"a client refuses a tool it no longer carries"*, in band,
at the call.

**There is no unload in v1, and no inheritance.** The set belongs to the agent
that loaded it; a fresh conversation, and a freshly dispatched subagent, starts
clean and loads what it needs through the `clients` tool every agent always has.
Subtraction is a second act on the one surface §5 says nothing but an explicit
load may change, and nothing has yet needed it.

**Where the injection runs, and what it may touch.** It is installed by the
`yog lernie` arm (DESIGN §16.7 W11) at `Fx::tool_injection`, which puts it
inside the **driver** — a child process, not the engine. Two consequences, both
load-bearing. Declaring touches nothing but disk, so a slow or absent engine can
never change the prefix. Answering a `clients` op needs presence, which is
engine RAM by ruling, so it asks through the door §3 already reserves for the
world's own residents: `Query::Clients` deposited into the `gestures/` inbox and
its reply read back with the one reply codec. **No verb and no transport were
added** — the roster read is the one bl-4e08 landed. The child folds the same
state root the engine writes, because the world hands `XDG_STATE_HOME` down to
every process it spawns (DESIGN §16.2). Every wait carries a bound and ends
early on lernie's stop flag, which is the router obligation lernie states and
cannot enforce.

**The tool host's own config, and why the advertisement is derived from it.**
The far end of the wire holds one operator-authored document, out-of-world
because it describes *that machine* — `<yog-data-root>/tools.json`, the sibling
of `wire/` and of the world subtree, for the same reason the key material is
(§8): a reseed must not take an operator's file with it.

```json
[{"name": "Bash",
  "description": "run a command in a shell",
  "input_schema": {"type": "object",
                   "properties": {"command": {"type": "string"}},
                   "required": ["command"]},
  "command": ["/usr/local/libexec/yog-tools/bash-tool"],
  "cwd": "/srv/work"}]
```

The first three keys **are** §5.1's advertised element, verbatim; `command` and
the optional `cwd` are the local half. The client presents the advertisement by
reading this file and dropping the local half — one document, two readings — so
what a host offers and what it can actually run cannot drift, which is the whole
of why the config is not a second list beside the advertisement.

`command` is an **argv, spawned directly**. There is no shell and no
interpolation of the invocation's input into it: a shell would make the declared
schema advisory and turn an operator's config into a command-injection surface
for anything the model can type. The invocation reaches the command exactly as
lernie's own tool contract already delivers one (its ARCH §3.3): the
`tool_use.input` JSON on stdin, bytes on stdout, the exit code the verdict. So a
tool host's executable is the same kind of program a local pool tool is, and the
capture that comes back is the same three facts.

JSON rather than TOML for one reason: `input_schema` is JSON Schema carried
verbatim (§5.1), and any other syntax would make the operator transcribe it.

### 5.3 The routing leg and the client-side executor, as landed (bl-024b)

**Four boundary verbs, and the wire gained none of them** (§3). Two are the
tool host's — `invocations`, the follow-class read that waits for this
machine's next work, and `complete`, the act that answers one — and two are the
asking side's: `invoke`, which queues a call for the machine that advertised
the tool, and `capture`, which polls for what came back. All four are ordinary
gestures in all three serializations, typable at any seat, and the compile
gates (`codec`, `line::spell`, the dispatch and answer matches) are what
enforced it.

**The identity is the intake's at three of the four**, exactly as the
advertisement's is (§5.1): a connection drains its own queue, answers its own
invocations, and collects its own captures. A `client` field on the read would
let one connection take another's work. An intake carrying no client identity —
the `gestures/` deposit inbox and `yog gesture` (§4.1; the window is a
certificate-bearing client since bl-ae05) — **refuses in band**
on the two that are a tool host's, with a sentence. `invoke` is the exception
and names its client, because there the identity is the **addressee** rather
than the author.

**A handle that is not yours is absent, not forbidden** (§4). A completion
quoting an invocation addressed to another machine, and a poll on a handle this
caller never posted, both earn the sentence a handle nobody minted earns. A
refusal that confirmed existence would be the disclosure §4 excludes.

**One reply for one subject.** `invoke`, `complete` and `capture` all answer
`Reply::Routed { invocation, capture? }` — the slot **as it stands after the
call**, which is the `Marks` discipline (a receipt is a re-read, never an echo).
`capture` is absent rather than empty while the far machine still runs it, so a
reader never has to tell "not finished" from "finished saying nothing".

**The mailbox is RAM, beside the presence refcount, and swept.** An invocation
in flight is a fact about *this process for the seconds a tool takes*, not a
fact about the world, so it is not a file (`src/registry/mailbox/slots.rs`, the
fourth lock carve-out). A slot lives from the post until the capture is read,
and a post older than an hour is swept — a driver that died mid-invocation
costs one entry rather than a leak.

**A capture is text, and the transcode happens once.** A capture ends as a
model's tool result and a model's message is text, so the *executor* transcodes
its child's bytes at the one place bytes stop being bytes, and nothing
downstream carries an encoding case. A tool whose output is not UTF-8 loses
exactly the bytes no string can name, which is the trade every other §11 file
read already makes.

**The executor is `yog tool-host`, a client mode beside `yog seat`.** It reads
the §5.2 config, advertises the projection of it, and then loops: `invocations`
→ run → `complete`. It runs **serially** (one invocation at a time, which is
what makes a busy host absent) and it **does not reconnect** — a channel that
fails is an exit naming the failure, because restart policy belongs to the
supervision the operator's machine already has, and inventing one here would be
yog deciding how a box it does not administer runs a program.

**A tool runs under two deadlines, and they measure different things.** The
host's own bound terminates the child (SIGTERM then SIGKILL, the cascade the
crate already owns) and answers the shell's `timeout` verdict with a sentence;
the driver's longer patience stands behind it for the case where the whole host
process went away. Neither is a knob: an engine that has not answered is down,
and a tool that has not answered is working.

## 6. What lives where

Everything durable is the server's: the world, the logs (`ops.jsonl`), the
UI-state documents, the agent processes, the LLM calls (brazen in the engine
process), the sentry, the pilot, the clock. A client holds exactly two things:
key material (operator-provisioned, out-of-channel — one set per entry,
§8.2) and RAM. Losing a client loses nothing; each server's disk remains the
single history of the workspaces it hosts (bl-aaec). A workspace held
elsewhere is therefore read through, never cached: there is no client-side
copy of a conversation, and a host that cannot be dialled is a workspace
painted unreachable, not one remembered stale.

## 7. Per-seat UI state

`ui.json` was one last-writer-wins document holding two kinds of fact. The
split cuts along that line:

- **Facts about the world** — seen watermarks, pins, acks — are operator
  facts, shared across every seat: one document, as today. (Attention answered
  on the phone must clear on the desktop; that is I0's whole point.)
- **Facts about the pane of glass** — panel sizes, collapsed sets, view knobs
  — become per-client documents keyed by client identity, held server-side so
  the client stays stateless and any two seats of the same client converge.

**As landed (bl-8bbc).** The world document stays exactly where it is,
`<yog-state-root>/ui.json`, so the §7.2 worker's watch, the whole-file `adopt`
and every external editor are untouched. The pane document is
`<yog-state-root>/clients/<client>/pane.json` — §4.1's layout, because a pane
document and a registration are both *this client's*, and one home for a client
beats two. The pane path is **derived, never stored**: `ui.json` sits at the
state root and `clients/` is its sibling.

Which document owns which key:

| Document | Keys | Why |
|---|---|---|
| world (`ui.json`) | `seen`, `pinned`, `identity_last_used`, `ceiling`, `prices` | assertions about the world, or operator policy over it. `identity_last_used` is the §3.2 name the operator last claimed a ball under — a thing they did to the world, and two seats claiming under different names is worse than converging. §10 keeps *whether a pin is a world or a pane fact* open; §7's default to world is kept. |
| pane (`pane.json`) | `panels`, `collapsed`, `zoom`, the two transcript-density knobs, `notify_unfocused` | how one piece of glass is arranged. A phone that puts the roster away must not put it away on the desktop; a desktop notifier is a fact about a desktop. |

**The local window is a client called `yog-window`** (§4.1 as narrowed by
bl-ae05), and that is the whole of its spelling: it presents a certificate
carrying that common name, it is scoped by its registrations, and it owns a pane
document like any other seat — so the window's panel sizes and the phone's are
two documents rather than one that grows a second reader later. The path is
derived from the identity at both ends, so the document the frame writes through
in process and the one a gesture the window sends over the wire lands in are the
same file. It was `local` until the ruling; the one visible cost of the move is
that a window's stored panel sizes reset once, on the boot that mints its leaf.

Both documents are read through **one handle**, so which file owns a key is
stated once, at that key's own accessor, and no caller knows there are two. The
§3.6 unmake subtracts from both; another client's pane keeps its now-inert
collapse override, because a collapse override for a section that no longer
renders costs nothing and readdir'ing every client to delete one would be a
sweep buying nothing.

## 8. Shape of the code

One crate, one multi-call binary, as today: `yog serve` runs the engine (the
former `headless` boot plus the wire listener); bare `yog` runs the window;
`yog seat` and `yog tool-host` are the two **client** modes, the second landed
with §5.3; `yog wire-certs` is the operator's mint (below). A TUI or web seat
later is another consumer of the same wire and needs nothing new from the
engine.

**Two roles in one process, one boundary between them** *(bl-ae05)*. Bare `yog`
boots the engine in its own process exactly as it always has — the listener
rides `Engine::boot`, one world one engine, and that ruling is untouched — and
then talks to it over **nothing but the wire**: a seat presenting the window
leaf, dialling `127.0.0.1` at the port the listener actually bound. The address
is handed over in RAM rather than read back out of a file, because the two roles
share a process and only the listener knows what a `:0` became. The two
alternatives §8 already rejected stay rejected and neither is needed: nothing
spawns a second engine, and nothing refuses a desktop launch with a terminal
instruction.

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
started it — a ruling about the box's OWN world, which bl-aaec leaves
untouched: bare `yog` still boots the engine, a box used purely as a seat on
other boxes is just a window whose local workspace set is empty, and no
window-only mode exists. Every other seat of *this* world is a client of this
one engine; the window is additionally a client of every entry it holds
(§8.2). The alternatives were both
worse and both were considered: bare `yog` *spawning* `yog serve` gives one
world two engines (two pilots, two sentries, two derivation workers — the
instance-coordination shape DESIGN §14 rejects), and *refusing with a remedy*
puts a terminal instruction in front of a desktop launch that has no terminal.

**Absence WAS the off switch, and bl-ae05 is why it no longer is.** It could
not survive §1.2's ruling: the window reads over this listener now, so a box
with no material would be a window that paints nothing — the outcome §8 had
already rejected in both its other spellings. The engine's boot therefore
**founds its own material** when it finds none (`provision::ensure`), and what
it writes is aimed at loopback. Severability is unchanged in substance: delete
the directory and the next boot writes a fresh loopback root; nothing about the
channel, the codec or the boundary is conditional on a file.

**What still distinguishes loopback from wider listening is the address, and
nothing else.** Self-provisioning writes `127.0.0.1:0` *(amended bl-dc14; it
wrote a fixed `127.0.0.1:<port>` until then)*, so material yog minted for
itself serves exactly the window that minted it — and, since the port is the
kernel's answer rather than a process-global number, **no two engines ever
contend for it**. That is DESIGN I0 kept at the wire: two yog instances
side-by-side — two worlds, or two windows on one — each bind their own port
and each get a live window, converging over the world's files exactly as
instances always have (instance coordination stays disk-only, DESIGN §14; the
wire is a seat's transport to ONE engine, never the bus between two). The `:0`
in the file is a **request**, and the request is the fact the file is the one
home of; what it *became* is runtime, held by the listener and handed to the
one seat that needs it — the window, in RAM (above). Nothing publishes the
bound port anywhere else, so a remote seat's known port is never a moving
target: a host that is not loopback is written by an operator — by `yog
wire-certs WIRE_HOST=…` (whose *stated* default port stays `7737`: an explicit
mint is a statement another machine will be told to dial), or by editing
`address` — and *that* is the statement of intent. One fact with one home
(below), no flag, and no ladder deciding how far a listener reaches. A local
`yog seat` on a self-provisioned world is refused legibly at the seat — a `:0`
names no dialable port, and the remedy is the operator's stated address —
which is the scope self-provisioned material always had.

A *half*-provisioned wire the mint **cannot heal** is still warned about and
still does not listen, because silently degrading to no encryption is the one
failure this design exists to exclude — and because replacing an operator's
trust root is the other. The mint issues only what is missing and only what it
can issue: a box holding an operator's `ca.pem` with no CA key beside it (a
client machine) is left exactly as it was.

**Certificates are minted out of channel, by `yog wire-certs` and by the
engine's own boot** *(bl-b6fa; amended bl-ae05)*. It shells to `openssl` — yog
links no certificate library and mints nothing *in channel*, ever (§1.4) —
writing a private CA and the server, client and **window** leaves into
`<yog-data-root>/wire`, plus one `address` file.

There is **one recipe** (`src/wire/provision.rs`) and two callers: the boot,
which mints what a box lacks aimed at loopback, and the verb, which is the
operator's own act for a server another machine dials by name and for a
rotation. `scripts/wire-certs.sh` was the recipe until bl-ae05 and is retired —
an installed binary has no repository to find a script in, and two spellings of
one act drift within a week; `make wire-certs` runs the verb, and its
`WIRE_DIR`/`WIRE_HOST`/`WIRE_PORT`/`FORCE` interface is unchanged. The server
leaf always carries `IP:127.0.0.1` beside whatever host it is minted for,
because the window is a client of loopback unconditionally and a certificate
that named only an operator's public host would refuse the one seat certain to
be there.
`WIRE_HOST`/`WIRE_PORT` name what the server binds and a seat dials; the SAN
and the address file are derived from the same `WIRE_HOST`, because two
spellings of one host is the drift §8's name resolution removed from the
boundary. The verb refuses to overwrite: a rotation distrusts every
certificate already issued, so it is `FORCE=1` and never a silent re-mint. (The
boot's call cannot rotate at all — it only ever adds what is absent.) Test material is
minted the same way at test runtime (`src/test_support/wire.rs`) — **a
certificate fixture is never committed**, which `make leak-scan` would refuse
anyway and which would be a private key in a public repository whether or not
it guarded anything.

**Nothing implicit binds a fixed port** *(bl-4c50, dissolved into the default
by bl-dc14)*. bl-4c50 found the suite contending: three tests that let the
mint fall back to the then-fixed default failed `bind 127.0.0.1:<port>:
Address already in use` on **every** run while yog was up, and the fix was a
fixture seed (`test_support::wire::ephemeral`) writing `127.0.0.1:0` before a
boot. bl-dc14 found the **app** contending the same way — a second bare `yog`
lost the bind and opened an inert window — and the two were one defect: the
fixture seed was a special case standing in for the right default. The seed is
deleted and `127.0.0.1:0` **is** the fallback `provision::ensure` mints when
nothing names an address, so the bare boot the suite exercises is the bare
boot a real box performs, and neither takes a port any running yog holds. The
default is still proved where nothing binds, by reading it back out of the
file the mint wrote — and the *bound* answer is proved by two listeners on one
world holding distinct ports (`wire::tests`). A world provisioned before the
amendment keeps whatever its `address` file names, the file being the
operator's; the second instance on such a world refuses **visibly** now
(below), and pointing the file at `127.0.0.1:0` is the stated remedy.

**A wire the engine cannot get up is a refusal the window PAINTS** *(bl-dc14)*.
Until then the whole failure family — a bind an operator-stated port lost, a
mint the box cannot perform, a half-provisioned directory, a window seat the
material cannot open — collapsed to one stderr line and an engine with no
listener, and `main.rs` kept the window anyway: a frame with no asker, no
poster and no searcher, accepting text and firing nothing, with the one
diagnostic on a stream a desktop launch has nowhere to show. This section had
already rejected refusing-with-a-terminal-instruction, and the missing half of
that ruling is that the refusal therefore has to be *paintable*:
`wire::listen` now returns the sentence instead of swallowing it, the engine's
boot says it once per face — stderr for `yog serve`, `AppModel::refuse_wire`
for a window (kept at the FIRST reason, so the cause outranks the "no seat"
derived from it) — and `shell::refusal` paints it INSTEAD of the shell: the
engine's own words verbatim, the same headline every wireless act receipt
carries (`wire::post::NO_WIRE`), the remedy naming the port-zero path, and
**no composer, tab or roster beside it**, because a control painted without a
wire only looks actionable. One early return in `shell::render` is the whole
gating; there is no per-control enablement to drift. Recovery is a relaunch —
the listener is a boot-time fact, and a retry loop inside a refused window
would be a second boot path.

**The material sits BESIDE the world, not inside it** — `<yog-data-root>/wire`,
the sibling of `<yog-data-root>/world`. The world subtree is a generated
artifact yog seeds, wipes and reseeds; key material is operator-provisioned and
irreplaceable by anything yog can do. Nesting it under a directory yog rebuilds
would make a reseed a revocation. Entries sit *inside* `wire/`
(`wire/workspaces/`, §8.2) for the same reason: an entry is the same
operator-provisioned, irreplaceable class of fact.

**An address is one fact with one home, and the home is the relationship**
*(amended bl-aaec)*. A server binds its own `wire/address` and a local seat
dials it; a workspace held elsewhere names its host in its own entry's
`address` file (§8.2). Every address still has exactly one file and no flag —
and no server-name knob either, because the name a client verifies is read
off the address it dialled (an IP literal is an IP identity, anything else a
DNS one).

**Paths never cross the wire.** *(Landed, bl-f5f6.)* Boundary types addressed
workspaces and projects by absolute `PathBuf`; across machines those are
meaningless and a disclosure besides. The wire spelling is now the **name**, on
the types themselves — `Action`, `Query`, the nested payloads they carry
(`monitor::Verb`, `fleet::Verb`, `fan::Obligation`, `config::ConfigFile`,
`start::Payload`, `start::Prepared`), the line's `Context`, and the two `Reply`
fields that identify rather than locate. The engine resolves a name to a path at
the dispatch chokepoint. The rulings below belong here rather than in a ball
body, and they have accumulated rather than arrived at once: bl-f5f6's four, the
live-enumeration barrier bl-6c9e added, and the conversation noun bl-49bc added
— so they are listed rather than counted.

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
- **The third noun is the CONVERSATION, and the rule differs again** *(bl-49bc)*.
  A workspace and a project are resolved over an enumerated set of *paths*; a
  conversation's identity is a **pair** — lernie's id (its branch name, and the
  only half that addresses a path) beside the §3.3 name it wears — so it is
  addressed by **an agent id, or the unique stored name a living agent wears**.
  That is not a vocabulary yog invented: it is lernie's own
  (`workspace::agent_name::resolve`, "an exact id match first, else the unique
  living agent wearing that name"), and the two spaces are disjoint by
  construction, every id opening with the compact `YYYYMMDDTHHMMSSZ` stamp and a
  name that reads like one being refused at creation — so the resolution never
  guesses which reading was meant. It had to become a contract because a
  `/prompt` receipt answers with the **minted name** (the root has no id until
  its detached driver writes `agents/<id>`) while the terminal's usage said
  `--agent ID`: the handle composed with `message`, lernie's one name-resolving
  verb, and with nothing else — empty inspector reads, refusals from `stop` and
  `retarget`, and, worse, *successes* from `floor`, `flag` and `seen`, which
  write yog's own id-keyed rows and so left policy that governed nothing. One
  vocabulary rather than a translation table: the receipt keeps publishing the
  name and every agent-addressed `Action` and `Query` accepts it. `Action::agent`
  / `Query::agent` (`src/boundary/address/agent.rs`) are the tables; the ladder is
  id-shape → the published derivation (the same set every read answers from) →
  disk, that last rung being the bullet below's barrier one noun down. Unknown
  and ambiguous refuse, and so does the ladder's legacy display-only rung, which
  is a title no ref answers to and never was an address.
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
- **The set it resolves over is the live enumeration, so a birth is a barrier**
  *(bl-6c9e)*. The intake builds each gesture's environment with the §3.1
  enumeration as disk holds it — three readdirs, folded over the published
  derivation's cached copy by `app::addressable` — and only then narrows it by
  scope. Resolving over the *cached* set meant an act that founds a workspace
  answered before the worker had enumerated it, so the very next call refused the
  name that reply had just made addressable: `/prepare` then `/prompt` could not
  compose two processes deep, and the window's own posted receipt earned `unknown
  workspace` for the wall its previous act had founded. **When an action returns a
  newly addressable resource, its success reply is a barrier for every boundary
  call after it** — kept as a *query*, not a claim: nothing stored, nothing
  republished, no wait. Three consequences, all of them wanted. Scope is
  untouched, because it still narrows by **registration** and the create's
  auto-registration writes that in the same breath — the live set makes a wall
  *enumerable*, never *authorized*, and a certificate nobody seated still gets
  `unknown workspace "x"` for a workspace it can prove exists. The steady state
  allocates nothing (the two sets agree; the published `Arc` is handed straight
  back). And the rule runs backwards for free: a workspace §3.6 deleted stops
  resolving at once rather than at the next sweep.
- **A reply speaks the name where it IDENTIFIES and the path where the path
  IS the answer.** `WsRow` identified a workspace by carrying the whole
  `Workspace` — path, kind and, for a named one, a `name` beside the path. It
  now carries the name and the kind: §3.1 makes the leaf the name, so the field
  that was already the identity became the whole of it and the path went. A
  field whose *subject* is a filesystem location, by contrast, keeps path
  semantics, because answering it by name would answer a different question.
  bl-f5f6 listed four such fields as a residual to be narrowed once a transport
  made the distance real. **§8.1 is what became of that list.**
- **`Prepared::name` is gone, not migrated.** It carried the §3.2
  `--as`/`YOG_NAME` stamp beside a `workspace` path. Once `workspace` became the
  name, §3.1 and §3.2 made the two fields one string twice — so the second went
  rather than being kept in step.

### 8.1 The path-typed reply residuals, closed (bl-ccf7, bl-b4b5)

The list was four items and it was wrong about one of them; three narrowed, and
the fourth is now a ruling rather than a residual. **The question each answers
is "what does this field's subject actually make computable?"** — not "can a
path be shortened", which is why three of the four ended up carrying *less*
rather than carrying a relative spelling.

- **`Reply::Applied { file }` carries nothing at all.** It said the absolute
  path that now held the staged text. But a `ConfigFile` **determines its own
  location** — the named workspace's wall `config.toml`, `models.yaml`,
  `workflows/<name>.yaml`, `cadence.yaml` — so the field was the destination
  the gesture had just named, respelled as the engine's home root. One
  computable fact said twice, and the second saying was the disclosure. The
  receipt is now `Reply::Applied`, the `Nudged`/`Acked`/`TrailCleared` shape:
  *it landed* is the whole of what a write adds to the address it was given.
- **`Reply::Marks { space }` is gone for the same reason, one level deeper.**
  It was answered on the argument that "which branch" and "whose branch" are
  one question. That argument expired at the §16.3 per-agent ruling: a
  workspace's marks space is now *always* its own (`<wall>/marks`), so the read
  is a pure function of the workspace the gesture named and the field could
  never have answered anything else. `Reply::Marks { branch }` — the branch,
  re-read after the write, exactly as before.
- **`WorkDiff`'s `Attempt::project` is the project's §5.1 #1 wire name.** This
  one identifies rather than locates, so it took the *name* this section's
  first bullet already rules for a project: the shortest unique trailing run,
  the word the §11 roster labels it with and the word `--project` takes. The
  patch read that needs a real repository (`workdiff::patch`) now takes the
  snapshot and resolves the name at that one seam — the same round trip
  `Snapshot::project_name` / `project_path` already owns, read in both
  directions rather than a path carried so one caller need not ask. Its one
  visible cost is the Work tab's copy-paste hint, which was `git -C <abs> diff
  …` and is now the range beside the project's name: a `-C` a remote seat could
  not have run anyway.
- **`Files` was never a residual, and the list was wrong to name it.**
  `FilesView` is `FileEntry { rel_path, … }`, `/`-joined from the worktree root
  since it was written, and `Preview` carries no path at all. The absolute
  worktree root exists only engine-side. Nothing was narrowed here because
  nothing crossed.
- **The join row's own two paths were the residual this list missed, and they
  are closed** (bl-b4b5). `JoinRow::project` and `JoinRow::workspace` rode
  `Reply::Balls` — and, copied off it, `BoardRow`'s two — as absolute paths
  under the engine's home, which is neither usable nor unseeable on a thin
  client. They are the §5.1 #1 project name and the §3.1 workspace leaf now, the
  same two words the `bl` family's actions already take, so a seat holding a
  focus can select its own workspace's rows without joining the answer back
  against the engine's table (bl-7407's refused shape). The engine resolves
  either back at the one seam that owns the round trip
  (`Snapshot::project_path` / `ws_path`).
- **`Prepared::binding` stays a path, and that is now the ruling.** It is
  lernie's `--cwd`: minted by the engine at `Prepare`, relayed back verbatim
  inside `Action::Prompt` by a seat that never reads it, and read again by the
  engine to seed the working-directory mark and to discover the §3.7
  instruction pins. Both narrowings §8 imagined are worse than the disclosure.
  An **opaque handle** needs a mint→resolve table, which is durable state for a
  fact that is already computed — the one thing the file religion and the
  single-source rule both refuse — and it would be state whose lifetime is a
  composer draft's. **Re-deriving it at the fire** means a second derivation of
  the work target beside the executor's cross-checked worktree, and §3.3's
  whole point is that there is one. So the binding crosses as a path, and what
  it discloses is bounded and stated: an engine-side directory name, to a seat
  that was already told the workspace it belongs to.

  It is one a **remote seat cannot use and does not read** — which makes it a
  disclosure and not an interop defect. Closing it wants the thing §9.5's
  residual wants: a ruling about what the local window *is*, because the shape
  that dissolves it (`Prepared` becoming opaque to the seat entirely) is only
  affordable once one seat's read path is the only read path.

  **This bullet claimed to be the last one and was wrong** (bl-22ab). It read
  "since bl-b4b5 this is the only path-typed field left on the reply surface",
  which was a statement about the *list above* rather than about the surface,
  and the surface was never swept for it. Four fields were still spelling
  engine-local paths, and the one that mattered is now closed:

  - **`QueueRow::workspace` was a `PathBuf` and `/attention` answered it**, so
    the §6 decision queue — the one read whose whole product is *an address you
    answer* — handed back a pair that `/seen`, `/message` and `/stop` all refuse
    with `unknown workspace`. It is the §3.1 name now, the token the gestures
    take, and `app::tests::attention` posts the answered pair straight back as
    a `/seen` to prove the round trip rather than the spelling.
  - **Three remain, all of them disclosure rather than broken addressing**, and
    each needs a ruling of the kind this bullet's first half is, not a rename:
    `search::Address`'s three path fields (`Reply::Search`), `fleet::Facts`'s
    `workspace` and `project` (`Reply::Board`), and `OpRow::cwd`
    (`Reply::Ops`), whose *subject* is where a command ran — the one case §8
    already says keeps path semantics, since answering it by name answers a
    different question.

  The general lesson is the one bl-f5f6 already paid for once: a type migration
  is finished when the **encoders** are swept, not when the types the ball named
  are. A field whose key matches a gesture's and whose value does not is worse
  than one that plainly does not match, because it reads as an address.

### 8.2 The client-side workspace (bl-aaec)

**The ruling** (operator ruling 2026-08-24): a client can be a client of many
servers, and the client-side workspace is what names one — its own mTLS
material, its own address, its own conversations, separate everything. This
section is that ruling made structural; §1.5, §1.8, §2, §6 and §7 carry its
other amendments.

**An entry is a directory, and its shape already existed.**

```text
<yog-data-root>/wire/workspaces/<leaf>/
    ca.pem        the HOST engine's anchors — that operator's trust root
    client.pem    this box's leaf for this workspace (one certificate = one
    client.key    client identity, §2 — so separation is §2's own rule)
    address       the host engine, host:port — "the server", entire
    workspace     OPTIONAL: the name this workspace bears on its host, when
                  it differs from <leaf>; absent, the leaf is the name
```

Four of the five files are exactly the material a pure-client box already
holds flat (`material::read_dir` with `Role::Client` reads an entry
unchanged): an entry is that directory, one level down, named. `<leaf>` is
the *client's* name for the workspace — the name the window's roster and
every gesture resolve — and `workspace` exists because a host's namespace is
the host's fact (§9.6: global per server) and two hosts may both call
something `home`: the remedy for a collision is a local rename, which is
`mv`, never a server-side rewrite. The mapping between the two names is spent
at exactly one place, the channel boundary, in both directions — a gesture
crossing the wire carries the host's name, a reply landing is labelled with
the leaf — the same two-directions-one-mapping discipline as §8's
`Snapshot::ws_name`.

**Separation is not a mechanism; it is the absence of one.** Entries share
nothing: not anchors (two servers are two operators' trust roots), not leaves
(one certificate = one client identity, and the provisioned norm is one leaf
registered in exactly its one workspace — then §1.5's invisibility is the
wire's own fact rather than a rendering choice), not addresses, not
conversations. An operator who registers one leaf in several workspaces on
one host collapses that channel-level separation to presentation — lawful,
and theirs.

**The window's channel set.** The window is a client of the engine in its own
process — over loopback, on the window leaf, at the address only that
listener knows (§8) — plus one channel per entry, each on the entry's own
material. The roster is the union: a workspace is a workspace, and which
engine hosts it is a fact painted on it, never a mode the window is in. Names
resolve over the union — local leaves and entry leaves in one namespace — and
a collision refuses naming the token, §8's two-roots-one-leaf rule with the
same remedy shape (rename the entry). Per channel, everything §3 ruled holds
unchanged: the asker's pass at human cadence, the poster's exactly-once act
routed by the workspace it names, the follow lane dialled at whichever
channel hosts the focused conversation, the searcher fanned out and unioned.
A channel that cannot be dialled is *that channel's* workspaces painted
unreachable — the bl-dc14 refusal discipline applied per entry, never the
whole shell, which stays reserved for the one wire the window cannot exist
without: its own.

**The reframe, tested and taken in half.** The proposal was that "local" stop
being a special case — the loopback engine one entry among N, the window
picking one. Half holds: loopback stops being the *only* channel, and
`Engine::window_seat`'s forced `loopback()` stops defining the window's shape
— it becomes one channel's address resolution among N. Half is rejected, for
cause. The loopback channel cannot BE an entry, because an entry's address is
a file and the loopback address is a `:0` answered in RAM by the listener
that bound it (§8, bl-dc14) — writing what it became to disk each boot would
store a runtime fact, the drift the one-home rule exists to refuse. And the
window does not "pick one": the ruling is a client of MANY servers at once,
so the window attaches to every channel it holds; picking would make the
window modal, and the mode would be the new special case.

**What a seat verb does with entries.** `yog seat` and `yog tool-host`
resolve the gesture's workspace name over the entries first; a name no entry
holds — and a gesture naming no workspace — goes where it always went, the
flat directory's client material. The flat directory therefore remains what
it has always been, the box's own root: its engine's material, its window
leaf, and the one client relationship the box holds without naming it.
Everything beyond the box's own engine is an entry.

**How material reaches an entry** — §1.4 verbatim, forever. On the host, the
operator mints a leaf for the visiting box (`yog wire-certs` issues an extra
client leaf under a stated common name — the one recipe, one more artifact it
can be asked for) and writes the registration (§4.1's `mkdir` and `touch`).
The anchors, leaf and key are carried to the client box by hand — the same
out-of-channel act the certificates always rode — and written into
`wire/workspaces/<leaf>/` beside an `address` the operator states. No
enrollment, no pairing, no first-connect ceremony. A workspace that does not
yet exist on the host is founded by the entry's own first `Prepare` (§4.1's
raise), which auto-registers its creator — in-channel work, on material that
moved out of channel.

**Migration: none.** A box with a flat `wire/` and no `workspaces/`
directory is the general path with zero entries and behaves as before, byte
for byte — the self-provisioned loopback root keeps its meaning, the window
keeps its one channel, `yog seat` keeps its flat read. The one deployed shape
that *should* move is a box whose flat directory was a copied client set
aimed at another machine: one `mkdir` and one `mv` turns it into the entry it
always was. Nothing forces the move; the flat spelling stays lawful as the
box's own root.

**What this does not solve, on purpose.**

- **Offline.** A client holds material and RAM (§6); a workspace whose host
  is unreachable paints unreachable and holds no cache to read. §11's
  rejection of syncing the world at clients is load-bearing here: offline
  reading IS a synced world.
- **Cross-engine aggregation.** Search fans out and unions; attention ranks
  across channels over each host's own facts. No global ordering, dedupe or
  clock is promised across engines — each host's timestamps are its own.
- **Revocation propagation.** Deleting an entry un-participates this box; it
  revokes nothing. Revocation stays the host operator's act (§4), and the two
  ends can disagree: a revoked entry is a dead channel painted unreachable, a
  deleted entry with a live registration is a host waiting for a client that
  never dials. Both are legible; neither is reconciled.
- **A lying host.** A client trusts each of its servers about that server's
  workspaces, so a compromised host can paint lies onto its own tabs. What
  separation buys is the boundary: it can say nothing about another entry's
  workspaces, and it ever held only what its channel's replies carried —
  which, for a client of RAM, is nothing durable.
- **Moving a workspace between hosts.** No federation, no export: a
  workspace's world lives on its host (§1.1) and this design gives it no
  second home.
- **Scale in entries.** The window's cost is linear — one channel, one asker
  pass per cadence period, per entry. Entries are operator-provisioned by
  hand, so N is small by construction; nothing here is sized for a box
  holding hundreds, and no mechanism was added against a problem that
  provisioning friction already bounds.

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
6. **Registration and scoping.** *(Landed, bl-8bbc — see §4.1, §7 and §9.6.)*
   The per-workspace registry, the one scope filter that makes an unregistered
   workspace absent, auto-registration on create, and the per-seat `ui.json`
   split.
7. **Tool hosts:** advertisement, rendering, routing, the lernie seam —
   after its own design pass (§5). **Landed, in three parts.**
   - bl-4e08 (§5.1): the `advertise` gesture in all three serializations, the
     per-client `tools.json`, connection-scoped presence, and the
     `Query::Clients` roster the workspace surface renders.
   - bl-c907 (§5.2): the lernie seam filled (`Fx::tool_injection`, lernie
     0.0.9), the `clients` tool with its three ops, the durable per-agent
     loaded set, and the frozen definitions the prefix declares. The
     model-facing half is complete: an agent can see this workspace's machines,
     read what they advertise, and make named tools callable.

   - bl-024b (§5.3): the routing leg and the client-side executor — the four
     boundary verbs (`invocations`/`complete` for the tool host,
     `invoke`/`capture` for the asking side), the engine-side mailbox beside
     the presence refcount, and `yog tool-host`, the client mode that reads the
     §5.2 config, advertises the projection of it, and runs what it is handed.
     An agent's call on a loaded remote name now crosses to the machine that
     advertised it and the capture comes back, verbatim.

   The day the leg landed the same call succeeded with nothing above it
   changing, which is what the interim refusal was shaped to make true: an
   in-band non-zero result is what a vanished endpoint had to produce anyway
   (lernie's §3.3), so the seam was complete and honest before the transport
   existed.
8. **The window becomes a client.** *(Landed. Opened bl-ae05 — see §9.7; the act
   path §9.8, designed and half-migrated in bl-4841 and **completed in
   bl-1747**, which deleted `AppModel::dispatch` and with it the window's last
   in-process gesture. Its read residual is §9.7's, and is scope rather than
   architecture.)* The operator ruling that settled §1.2 against §4.1, the
   boot's own mint, the window's leaf and its seating, the off-frame asker, and
   the first surface painted from a decoded reply. Its residual was the
   migration of the remaining reads and of the acts: **the acts are done**
   (§9.8 — every one of them crosses, and the second execution path §1.2 exists
   to refuse no longer exists), and the reads are bl-f297's list (§9.7).

   **Two rulings came out of it and belong here rather than in a ball body.**
   *Presence is not the routing predicate* (§5, amended): a tool host holds a
   connection only while it is waiting, so a busy one is absent and a presence
   test would refuse exactly the calls that would have succeeded — the queue is
   the predicate, and the caller's deadline is what makes a vanished machine
   visible. And *the asking side needs two verbs, not one* (§3): the engine's
   intake is one thread for the world, so no gesture may wait on a tool.

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

**The residual, as it stood: the window was not a client.** §1.2 rules that
the UI operates entirely via RPC and that even the local case does the hard
split. What bl-b6fa landed was the channel, the server and a *terminal* seat;
the window still held the engine it serves and read it in process. **bl-ae05
opened the read path** (§9.7) — the window now dials its own listener with its
own leaf — and what remains of this bullet is scope rather than architecture:

- A frame still paints the published snapshot (§7.2) for every surface but one.
  §9.4 finished the *taxonomy* — paint may name only what a `Reply` can say —
  and bl-ae05 built the delivery, but only the clients section (§5) is fed by
  it so far. The rest is a migration, surface by surface, with the transport
  already under it.
- ~~Per-seat UI state (§7) is undivided~~ — split by bl-8bbc (§7, §9.6).
- ~~The path-typed reply fields §8 lists as residual are still absolute
  paths~~ — closed by bl-ccf7 (§8.1): three carried a fact the address already
  made computable and now carry nothing, and the fourth is a ruling.
- ~~One connection per gesture~~ — kept, and ruled rather than deferred
  (§10, bl-ccf7). No follow-class `Query` yet; the framing carries a chunked
  stream already (§3), so the live tail needs a query, not a wire change.

Registration, scoping and reply filtering were deliberately **not** here: a
connection was trusted at certificate grade this step. §9.6 (bl-8bbc) closed
that, and the per-seat `ui.json` bullet above with it. **One residual in this
list is left, it is the read path, and §9.7 is why it is still here.**

### 9.6 Registration and scoping, and what the trust domain still is not

Landed (bl-8bbc): `src/registry/*` — the file-per-registration store, the
reserved `local` identity, and the certificate → common-name walk;
`Snapshot::scoped`, the one filter; `ConsumerCtx::answer_as`, the scoped
chokepoint; the raise in `dispatch`; and the two-document `UiState`. The
rulings live in §4.1 and §7 rather than in a ball body: the registry layout,
the identity spelling, the one-filter scoping, auto-registration without
create-detection, the raise-cannot-join rule and the existence-is-observable-at-
creation ruling, plus §7's key-by-key split and the `local` window's spelling.

**What the workspace-grade trust domain still is not.** §1.5 divides
workspaces and nothing else, and the landing divides exactly the
workspace-keyed facts. The **project** set, the balls projection, the §3.5 join
and the `ops.jsonl` trail are world-wide and every registered client sees all of
them. That is §11's standing rejection of per-tool and per-verb ACLs in force,
not an oversight — a finer policy layer is speculative until a second human
exists — but it is worth stating plainly, because "the workspace is the trust
domain" reads like more isolation than it buys: a client registered in one
workspace still reads the trail of every workspace.

**Two residuals, named rather than papered over.**

- ~~The window is not scoped, because the window is not a client yet~~ —
  **closed by bl-ae05** (§9.7). It carries `yog-window`'s leaf, it is scoped by
  its registrations like any client, and the engine seats it in every workspace
  it enumerates (§4.1). Its pane document moved with the identity.
- **A workspace name is a global namespace.** Creation refuses on a collision,
  including with a workspace the creator cannot see, so two clients cannot both
  hold a workspace called `home`. Per-client name spaces would dissolve it and
  would also break §3.2's `--as` identity, which is the same leaf; the collision
  refusal is the cheaper answer and §4.1 records what it discloses.

### 9.7 The read path, and the ruling that opened it (bl-ccf7, bl-ae05, bl-adcb, bl-f297, bl-44e9, bl-13f9, bl-7407, bl-48ae, bl-296f, bl-b4b5)

Landed (bl-ccf7): §8.1's narrowing of the path-typed reply fields, and §10's two
transport questions settled. It could not move the read path, because §1.2 and
§4.1 could not both be executed and the tree had already executed §4.1 — a
window that dialled the wire would present a leaf, be identified by *that* name,
be scoped like any remote client and read a different pane document, and on an
unprovisioned box would have no read path at all.

**The ruling, 2026-08-14: neither of §9.7's cheap options — the window is a wire
client of localhost.** One boundary, everything through the front door: the real
socket, the real handshake, the real certificate. The in-process transport
bl-ccf7 recommended was declined, so §11's rejection stands unlifted; §4.1 was
narrowed rather than kept, so the collision is dissolved rather than deferred;
and the unprovisioned box is answered by the engine's boot performing the same
out-of-channel act the operator's own verb performs, on the operator's own box,
before anything is dialled (§8). Every consequence is recorded where it belongs
— §1.2, §4.1, §7, §8 — rather than here.

**What bl-ae05 landed.**

- `src/wire/provision.rs` — the one `openssl` recipe, and `scripts/wire-certs.sh`
  retired into it. The engine's boot mints what a box lacks; `yog wire-certs` is
  the operator's act over the same function.
- `Role::Window` and `registry::WINDOW` — the window's leaf and the identity on
  it, one const spent by the mint and by the seating.
- `src/wire/asker.rs` + `src/wire/link.rs` — the off-frame asker and the frame's
  half of it. The frame declares standing questions and paints what has landed;
  the asker seats the window, dials loopback once per ask at human cadence,
  decodes with `reply::decode` and publishes. Two channels, no lock, and no
  frame-side wait: a dead engine costs a surface its content and the window
  nothing.
- `Engine::asker` — a window takes it, `yog serve` never does. One asker per
  engine, because the link end is taken rather than shared.
- The **clients section** (§5) is the first surface painted from a wire reply:
  the roster it renders crossed loopback mTLS, was scoped against the window's
  own registrations, and was decoded like any seat's. Its per-derivation memo
  and the model's in-process `clients()` derivation both went — with the model's
  copy of the presence map, which nothing reads any more.

**What bl-adcb landed, and what it found.**

- Two more surfaces read over the wire. The **ops trail** (§4.2, the §11
  activity accessory's expanded pane) is a `Reply::Ops` bounded by
  `opslog::OPS_TAIL`, which is now the log's one number rather than the
  derivation's private one; the **V4 board** (the §11 balls fold) is a
  `Reply::Board`. `AppModel::ops_rows` and `AppModel::board` went with them —
  neither had a memo, an answer already being the cached fold.
- `src/shell/wire.rs` is the shell's **one spelling** of a wire read. bl-ae05's
  four arms — the answer, the refusal, the honest not-yet, and the wrong-kind
  reply that is a codec defect rather than a state — were written by hand for
  the clients section; restating them per pane would restate the third one
  wrong. A surface now declares its query and picks its payload out of one
  reply variant.
- `src/shell/acceptance/wire.rs` is the **acceptance world's own answerer**: the
  frame's standing questions taken off the `LinkEnd` exactly as the asker takes
  them, decoded by the one codec and answered by `AppModel::answer` — the
  transport stood in for, never a second dispatch, because a fixture mints no
  certificate and binds no port. Without it a migrated surface paints nothing in
  every test, which is a surface with no witness. It pays the **settle-then-render
  shape** out in full: paint (ask), settle, answer, paint (re-declare), settle
  (land), paint. Three passes, and it cannot be fewer — a settle whose frame
  declared nothing drops the answers as no longer standing, which is the same
  rule that makes a collapsed pane free.
- **A migrated read paints the derivation, never the frame's fold** — and that
  is §7.2's own partition (*paint reads the fold, gestures read the derivation*)
  landing on the other side of the line, because a wire read is answered from
  the engine's published snapshot like any seat's. The cost is stated rather
  than discovered: the §3.4 pending **echo** and the §7.2 live tail are folds
  the window makes for itself, so a just-fired conversation no longer shows as a
  drone row on the board until the derivation carries it. Optimism is a seat's,
  and a seat that reads over a wire has none — which is a fact bl-4841 inherits
  whole, since an act's receipt is what the echo exists to stand in for.
- **What crosses is the read, not the affordance.** The board's rows crossed;
  `startable`/`resumable`, which build the composer's fire-time inputs the
  click-glue consumes synchronously, did not. That line is the acts ball's, and
  a surface that mixes the two migrates its read half only.

**The residual, as bl-adcb left it: three surfaces read over the wire, the rest
do not, and the acts do not.** This was still scope rather than architecture,
but the remaining reads were **not** more of the same three moves — each was
blocked on one of three named things, and bl-adcb's audit is what found that.
**bl-f297** took the list; what follows is what it landed and what it ruled, and
the three classes are stated in its terms rather than in the audit's, because
two of them dissolved.

**What bl-f297 landed.** Three more surfaces: one of bl-adcb's two unblocked
candidates, one the acts ruling had just unblocked, and one out of a class the
audit had called blocked and which turned out to need a sentence rather than a
mechanism.

- The **§11 Work tab** (§5.1 #32) is a standing `Query::WorkDiff`. Its *two*
  per-snapshot memos went with it — every arm of that read forks `git` against
  a project repo, which is why the in-process version was memoized, and an
  answer is that cached fold refreshed at the asker's cadence instead. The
  listing and the picked file's patch were two reads with two memo keys and are
  now **one question**: the query carries the file, so re-picking asks a
  different question and scrolling asks nothing. An inactive tab declares
  nothing at all, which is the collapsed-pane rule.
- The **marks pane's `Read current`** is the click-time read §9.8 handed back:
  **a standing question with a latch**. The click throws the latch, the pane
  declares `Query::Marks` while it paints, and the branch lands one ask-period
  later. The latch names the *workspace* it was thrown for, never a bare
  `bool` — the focus can move under it, and a latch that could not say which
  wall it meant would paint one wall's branch under another's name. It does
  **not** fall on the first answer: a read that switched itself off would be a
  one-shot with a socket behind it, showing the same bytes whether the branch
  moved a second later or not. The `Cli` pair evaporated from the pane and from
  `config_edit::center` above it, which is §9.8's observation reaching the read
  side.
- The **§6 decision queue** behind the desktop escalation is `Query::Attention`,
  and `AppModel::decision_queue` went with it — one accessor fewer, and the
  strip's count, the desktop's ask and a headless `/attention` are now one
  answer rather than one derivation run twice.

**Class 3 dissolved, and the rule it needed is one sentence.** The audit's third
class — *a read with nowhere to paint a refusal, or a consumer that reads it
synchronously* — is empty now, and neither half needed a mechanism.

- The **alert baseline** was the hard half: `alert::announce` folded the queue
  on every frame, and a frame the wire has not answered holds no queue, so
  folding one would read as everything having departed and then, on the next
  answer, as everything arriving at once — the first-boot flood re-armed twice a
  second. The rule: **an unanswered frame is not a reading of the queue, and
  neither is a refusal.** The baseline moves only on a frame that was told
  something, which is the *same* rule that already makes a freshly-opened window
  silent — no observation, no arrival — so the special case dissolved into the
  general path rather than earning a branch. The refusal half answers itself
  with it: this seat has no surface to paint a refusal on, because a
  notification is output and not a pane, so the window stays quiet rather than
  announcing on a guess. Recorded in DESIGN §6, where the baseline's ruling
  lives. What it costs is stated: the fold runs at the asker's cadence rather
  than the frame's, and the focus gate is read on the frame the answer lands.
  A window buried and re-focused inside one ask period folds once instead of
  thirty times, which is a difference no difference detector can feel.
  Its witness is **both directions** (`shell/acceptance/alerts.rs`), because the
  positive one alone would have passed against a seat that never asked: each
  test seeds the baseline with an empty fold first, since with no prior
  observation `announce` returns nothing whatever the frames did.
- The **§9 config family** was smaller than the audit read it as, and that is
  worth recording rather than quietly acting on. Of the five queries the audit
  named (`ReadConfig`, `Lineages`, `Models`, `Providers`, `Marks`) exactly
  **one** was ever a frame-side read: the marks pane's, migrated above. The
  §9.1/§9.2/§9.3 editors do not reach those queries at all — they load their
  bytes directly and their *writes* are the boundary's — so "the config family
  is answered at click time" was true of one surface and a projection onto four.
  Pointing the editors at `ReadConfig`/`Lineages`/`Models`/`Providers` is a real
  and separate migration: it would replace `Editor::load` with an answer, which
  is a change to what a config editor *is* (§9.5's "controls over facts"), not a
  read moved onto a wire.

**Class 1 stands, and the ruling is that it stands.** The viewport folds —
`Query::Conversations`' expanded set and the §11 inspector family's rail **pin**
— stay in process, and §8.5 is **not** amended. Three shapes were weighed:

- **The fold as a query parameter.** It preserves the letter of the ruling — the
  boundary would still *store* no viewport state, the seat spelling its own fold
  in its ask — and it is the shape `visible_conversations(now, expanded)`
  already has. It is refused on §8.5's own sentence: *"Views gain no boundary
  representation, by design."* The parameter **is** the representation; the
  question of whether the engine keeps it is a different question, and answering
  the second does not answer the first. The precedent that looks like a
  counter-example is not one: `Query::Files { path }`, `Query::Step { seq }` and
  `Query::WorkDiff { file }` carry a *selection*, and a selection names **which
  thing you are asking about** — it is the question. A fold names **which
  answers you have open**, which is the rendering of an answer already given.
- **The fold becomes a durable pane fact.** Closed already, by DESIGN §5.3's own
  entry on the expanded set: it is keyed one-per-conversation and would accrete
  a stale key for every conversation that ever existed, and mirroring it would
  drag a second instance's list open under the operator's hands. Not re-argued
  here.
- **The answer moves altitude.** The one shape that could work: the boundary
  answers the whole descent **forest** with its per-row rollups, all-collapsed
  being the root subset of it, and each seat renders the rows its own fold makes
  visible. The fold then never crosses, the seat derives nothing (it *selects*),
  and one derivation still serves both seats. This is not smuggled in with a
  migration, because it is a change to `Reply::Conversations`' payload that
  every seat and the codec read — a headless `/conversations` included — and the
  same move on `Reply::Rail` for the pin, plus the pinned `Files` listing that
  no query spells at all. It is a payload ball, not a read-path one, and it is
  where this class goes next.

**Class 1's payload landed, and the class is closed (bl-44e9).** The third shape
was built, and §8.5 is still not amended — the fold stayed seat-side, so *views
gain no boundary representation* is true of the letter and of the spirit.

- **`Reply::Conversations` is the whole descent forest.** `answer::conversations`
  answers every member with its own per-subtree rollups, in paint order, and
  `nav::convs::visible(rows, expanded)` is the seat's pure selection out of it —
  a one-pass depth cut over rows, no snapshot, no recursion. `visible_rows` split
  into `forest_rows` (the derivation) and `visible` (the fold), which is the
  altitude ruling made structural. A seat holding no fold selects the **root
  subset**, which is byte for byte the list this query used to answer, so a
  machine reader that wants the old shape filters `depth == 0` and every existing
  consumer keeps working.
- **The §11 list reads it over the wire**, and `AppModel::conversations`,
  `visible_conversations` and `conversation_groups` are gone with it. One seat
  read (`shell::convs`) serves the paint, the ↑/↓ walk and the ← that pages to a
  parent, so the rows on the glass and the rows the keyboard steps cannot be two
  answers. The grouped-by-ball view is `nav::convs::group::group_by_ball` over
  the same folded rows — a partition of a selection, not a second derivation.
- **`Reply::Rail`'s notches carry the budget as a rollup**, not their own spend
  (`Notch::budget`, wire key `budget`), so `rail::pin` reads one field off the
  notch the operator picked instead of summing the prefix. Every field a pin
  shows is now on the answer: the commit and the cut were already, the transcript
  is a prefix of the chat the seat was answered, and the budget is this. The pin
  is a *view* that derives nothing.
- **The pinned `Files` listing is spelled**: `Query::Files` gained
  `at: Option<String>`, and it is a **selection, not a fold** — it names which
  *tree* you are asking about, exactly as `Step { seq }` names which step. Of
  VISION V1.2's four pinnable tabs this is the one whose subject is a different
  tree; the other three read something the seat already holds. Headless it is
  `/files [<path>] [--at <commit>]`, and the window's own pinned tab is the memo
  around that one answer rather than a second spelling of the same two arms.
  **Config-frozen-at is the same shape and has no query at all yet** — when it
  gets one it takes `at` for this reason, and that is a residual below rather
  than a decision left open.

**And the §3.4 echo needed an altitude of its own, which is the finding.**
bl-adcb's *"optimism is a seat's, and a seat that reads over a wire has none"* is
true of the board and would have been a silent regression here: the §11 list is
the surface §3.4 exists for, and migrating it read as deleting `bl-915e` — the
operator's typed goal with no representation anywhere in yog until the driver
wrote a branch. The sentence is narrower than it looked, and the ruling is:
**a seat's optimism reaches whatever that seat actually reads.** So the echo has
a second projection beside `echo::compose`'s — `echo::rows::with_echo`, over an
answered list — and both live in the one module, which is `compose`'s own
single-source argument kept. A start with no row yet leads the list, faded, in
the operator's own words; a target already in the answer is *freshened* to the
echo's own age and nothing else moves (the reorder `compose` earns by bumping
`last_action_unix` is deliberately not made — the answer arrives sorted, and a
seat that re-sorted it would be deriving). A `Target::Agent` the answer no longer
carries adds nothing.

**A driven frame settles its own reads, too** (the harness half). §9.8 ruled that
`Screen::run` pays an act's round trip once so the transport does not leak into
every call site; bl-44e9 extended it to the read half for the same reason — a
migrated surface paints an answer that landed a round trip later, so a drive that
only settled acts saw every such surface blank and a beat asserting a row is
*there* would have been red for the transport's reason rather than the window's.
The loop settles to a fixed point on `AppModel::awaiting` — *is any standing
question still unanswered* — because a `Link` may never be settled twice without
a frame between (the second declares nothing and drops every answer), so a landed
read costs the two extra passes `wire::wired` spells out and cannot cost fewer.
A frame that asked nothing new pays none of it.

**Class 2 stands unchanged, and bl-44e9 found what it is really blocked on.**
`Reply::Workspaces` carries no path (§8.1's ruling), and the tab bar's every
entry is a **focus target** — `nav::tabs::Tab` holds the `PathBuf` a click, a pin
toggle and the §3.6 delete seat all spend. It could be made to *paint* off a
reply tomorrow by resolving each name through `Snapshot::ws_name`/`ws_path` at
the seat's door, and that is exactly the shape to refuse: the seat would be
joining a wire answer back against the engine's own in-process table, which is
two sources for one fact and worse than not having migrated. The honest change is
the wide one — focus held by **name**, resolved at the doors that need a path.

It was attacked and put down, and the reason is worth more than the refactor
would have been. **A name resolves against the enumeration, and the §3.4 raise
focuses a workspace the enumeration does not carry yet.** `Snapshot::ws_path`
answers over the published `workspaces` set; a `lernie new` that has just
returned is a wall the derivation has not read. Today the frame holds the path
and the composer's bare rung fires into the wall that was just raised
(bl-9acf's whole fix); name-keyed, that focus would be unresolvable for as long
as one derivation takes, and Enter in that window would fire into the *previous*
workspace. Re-deriving the enumeration synchronously at the raise, or holding the
raised wall as a claim until the world catches up (the §3.4 start claim's own
shape, one noun up), are the two answers — and picking between them is design,
not a refactor, so it is its own ball. Note that it is **not** dissolved by
migrating the tab bar: the engine resolves the same name against the same
snapshot and refuses for the same reason.

**Class 2's prerequisite landed, and the raise was answered by the claim
(bl-7407).** `Focus::ws` is the §3.1 name — the wire spelling — `nav::tabs::Tab`
carries no path at all, and `selected` compares names. The synchronous re-derive
was refused: it puts a disk walk on the receipt path, and it is a special case
where a general shape already exists.

- **The claim is the §3.4 start claim's shape one noun up, and it is one
  mechanism rather than two** (`src/app/raise.rs`). Same holder — per-instance
  RAM on the model (§13.1) — same retirement predicate, *the derivation shows
  it*, and the **same seam**: `echo::compose` is *"the one place snapshot and the
  non-derived facts meet… a third such fact is a third argument here rather than
  a third mechanism"*, and the raised wall is that third argument. The two stay
  two *values* because a raise carries no message and a send raises no wall.
  Every landed `Prepare` claims — the raise, ▶ Continue's own claimant wall, the
  bootstrap — and a start into an enumerated workspace is simply the claim
  retired the instant it is made, which is one rule instead of a raise/re-focus
  branch. bl-9acf's invariant is held by
  `app::balls::tests::prepare::a_raise_focuses_the_raised_workspace_and_retargets_the_bare_rung`
  (the bare rung resolves into the wall just raised) and, end to end through the
  real window, by `shell::acceptance::raise`.
- **Folding beats resolving per door, and that is what keeps it one source.**
  The claim is folded into the snapshot the frame paints, so `ws_path`, the tab
  bar, the centre pane and the composer read the *same* enumerated set and none
  of them knows a claim exists. It is retired ahead of the fold, so the painted
  set can never carry one workspace twice — which matters, `by_leaf` refusing an
  ambiguous name.
- **The doors that still need a path are named and few**:
  `AppModel::workspace_path` (the one resolver, `by_leaf` over the painted
  enumeration — the engine's own rule, so the two directions cannot disagree),
  `focused_workspace` above it, the pin toggle (`ui.json` keys pins by path,
  durable state whose re-keying is its own migration, so `toggle_pin` takes a
  name and resolves at the click), and the §3.6 delete dialog. `focus_agent`
  keeps taking a path because the acknowledgement it writes is keyed by one and
  the caller holds it already.
- **Two things dissolved rather than moved.** `focused_ws_name` was a leaf
  derivation off a held path and is now the focus verbatim; `startup_focus`
  answers a name while still *ranking* by path, so §4.1's derived-path order is
  unchanged.
- **What this does not do**: the tab bar still paints from the in-process
  enumeration. The refused shape — painting off a `Reply::Workspaces` while
  joining each name back through the engine's own table — is now the only thing
  standing between it and the wire, and the join is gone from the seat, so the
  migration is scope.

`Prepared::binding` stays named for the condition it always had (*once one seat's
read path is the only read path*), which bl-44e9 does not reach — the composer's
own reads are not migrated.

**Search was re-assessed and is no longer an unblocked candidate.** bl-adcb
named it one, and that reading predates §9.8. The asker's pass is **serial over
the standing set**, and §9.8 put acts on a thread of their own precisely because
*"an act runs as long as the verb behind it, so posting it on the asker's thread
would stall every standing read for the duration"*. `Query::Search` walks every
transcript in the world; a standing question is re-asked every `ASK_PERIOD`, so
riding the asker as written turns a once-per-ask walk into a 2 Hz one **and**
puts it in front of every other surface's answer. That is the poster's own
ruling pointed at a read. Two shapes remain and neither is "retire a mechanism":
give the asker a second lane for long reads (which is the poster, renamed), or
point the existing `Searcher` thread at the wire — it stays a thread, justified
by exactly the argument above, and what changes is that its read crosses the
boundary instead of running in process. The second is the smaller and is the one
to take; the win is `§1.2` compliance, not one fewer thread.

**Landed (bl-44e9), as ruled.** `Searcher` holds a seat rather than a snapshot
cell and asks `Query::Search` over the wire; it is minted by `Engine::searcher`
beside the asker and the poster, on the same window leaf and the same failure
condition, and rides `WindowWire` as its third half. Two consequences are stated
rather than discovered. **A refusal is an unreadable source** — `Found` already
carries *"each unreadable source, named with why"*, which is exactly what the
engine's sentence is, so a refused search paints the reason and never reads as
*no matches*; no new state and no second pane. And **the abandon predicate went
with the walk**: in process a superseded search stopped mid-walk, and over the
wire the engine finishes what it was asked while the answer for a question nobody
is asking any more is dropped on publish. The cost is one wasted walk on the
engine; the alternative is a cancel the boundary would have to carry, which is a
mechanism for an optimisation.

**The residual, as bl-44e9 leaves it.** The payload work is done and two more
surfaces crossed; what is left is scope with one exception, and the exception is
named rather than open.

- ~~The §11 inspector family is still read in process~~ — **landed, bl-13f9**,
  below.
- ~~Config-frozen-at has no query~~ — **spelled, bl-13f9**, below.
- ~~The Files preview reads its own two arms~~ — **dissolved, bl-13f9**, below.
- **The workspace tab bar** (class 2) — ~~blocked on the raise's unenumerated
  focus~~ **unblocked, bl-7407**: focus is a name and the raise holds a claim, so
  what is left is the migration itself.
- **The §9 config editors' own loads**, which were never one of the three classes
  (bl-f297's ruling: pointing them at `ReadConfig`/`Lineages`/`Models`/
  `Providers` changes what a config editor *is*, and is a separate design).
- ~~The seat's own `Query::Agent`~~ — **closed, bl-48ae**, below.

Nothing in that list is blocked on a decision that has not now been taken; each
is blocked on work whose shape is named.

**And the list is a list of NAMED SURFACES, which is worth saying out loud**
(bl-48ae). Every one bl-adcb's audit enumerated has now crossed or been ruled
on, and it would be easy to read that as *the window reads over the wire*. It is
not what any of these balls established. What is still derived in process is the
§11 **accessory** tail nobody put on the list: the header's live mark
(`mark_seats`) and the bottom in-flight strip (`flight_strip`), the §3.2/§3.5
ball and spend family (`conversation_ball`, `ws_balls`, `ball_spend`,
`conversation_spend`, `conversation_context`, `focused_join`, `bound_ball`,
`roster_ball_rows`), the composer's pending listing (`focused_pending`) and its
§3.3 title table (`agent_titles`), the §2.2 lineage tip (`config_tip`), the §3.6
and §8.4 fire-time gates (`delete_confirmation`, `agent_delete_confirmation`,
`move_targets`, `conversation_names`), and the altitude-0 chrome
(`tab_bar`, `strip_total`, `activity`, `staleness`, `has_alarms`). None of them
is blocked and none of them is a ruling — each is a `Reply` that does not exist
yet — but a residual that says *three surfaces plus scope* while thirty
accessors fold the snapshot on the paint thread is a doc that has stopped being
true. (**Seven of that list crossed in bl-296f and needed no new query at all;
that block, and what is left of this list, are below.**) The **affordance**
half is deliberately not on that list: `startable` /
`resumable` and the composer's fire-time inputs are the acts side of bl-adcb's
own line (*what crosses is the read, not the affordance*).

**The §11 inspector family crossed, and config-frozen-at got its query
(bl-13f9).** Six tab reads — transcript, steps, one step's records, the worktree
listing, the spine, the mail — plus the seventh the pin needed and nobody had
spelled. The window's Altitude-2 pane derives nothing at all now; what is left in
`shell/inspector` is which tab is open and the pin.

- **The four memos went with the accessors** (`tx_memo`, `steps_memo`,
  `rail_memo`, `files_memo`), which is bl-adcb's rule paid in full: an answer
  *is* the cached fold, refreshed at the asker's cadence rather than the
  derivation's, so a memo in front of one is a second cache over one fact. Two
  seats outside the inspector read the same two questions — the centre's
  auth/wound banners want the steps view, the composer's ↑ recall wants the
  transcript — and pay nothing for them: a standing question is keyed by its own
  encoded envelope, so two callers are one ask. What is left in
  `shell/ram/inspector.rs` is the V2 fork composer's choices, whose subject is a
  draft's input rather than a §11 read.
- **The Files preview's seat-side branch dissolved rather than moving.**
  `Query::Files` carries the path *and* the tree, so one ask answers the listing
  and the picked file's bytes at whichever commit the pin names. Its cost was a
  **selection change**: `Ephemera::files_sel` is the entry's path now, not its
  row index, because the number would index a listing that landed a round trip
  ago. That is `work_sel`'s own shape one tab over, and it is the general rule
  for a selection that is a query's parameter.
- **Config-frozen-at is `Query::Governing { workspace, agent, at }`**, all three
  serializations plus a help page, answered by `answer::inspector::governing`.
  `at` is the `Files` shape and for the same reason — it names *which commit*,
  which is the question and not the view — and absent it is the agent's own tip,
  resolved engine-side off the published snapshot so no seat has to know one
  before it may ask. It is the family's one read that **refuses** where its
  siblings answer absent: the derivation is a walk of the workspace's git and
  fails as `Lineages` fails, and "this conversation has no policy" is never a
  reading. Headless it is `/governing [--at <commit>]`.
- **The pin stayed seat-side and §8.5 is still unamended.** Every field it shows
  is on an answer (bl-44e9's altitude work), so `shell/inspector/rail.rs` is two
  selections and nothing else: pick a notch out of the landed spine, cut the
  landed transcript in front of it. Of VISION V1.2's four pinnable tabs, the two
  whose subject really is a *different tree* now name that tree as a query
  parameter; the other two fold over answers the seat already holds.
- **A pane declares several questions, so refusals are collected distinct**
  (`shell::wire::Said`). The family shares one address, so an unresolvable
  workspace refuses every one of them in the same sentence — five copies of which
  is a report about the transport, where one is content.

**The live tail went half a second old at the seat, the follow-stream
graduation was declined — and the operator reversed that decline.** Both halves
are kept, because the second is only readable against the first.

*What bl-13f9 ruled.* bl-ccf7 named the §7.2 live tail the one follow-class
candidate, and this migration is where it would have paid. It did not, and the
reasoning was the ruling rather than the scope. The fold itself is unmoved —
`inspector::live_tail` is still the boundary's (bl-6233), so the two seats cannot
describe one moment differently — but the frame used to re-fold it off the
rendered snapshot every paint and now reads it at `ASK_PERIOD`, so streamed text
arrives as one row per half second rather than as characters. That is legible:
500 ms is the cadence the whole window already reads at, and a model's output is
prose a person reads, not a machine polling a machine — which is REMOTE §10's own
criterion for minting a held connection, and it is not met. Taking the
graduation would cost a **second lane on the asker**, because the asker's pass is
serial over the standing set and a held read would stall every other surface for
its duration — which is the poster, renamed (§9.7 already refused exactly that
for `Query::Search`). A mechanism for a legibility optimisation is the wrong
trade; if the tail ever needs to be smoother, the lane is the ball, not the
migration.

*The reversal (operator ruling 2026-08-22, bl-73e7).* **Streamed model output
must reach the glass at write cadence.** The decline turned on a judgement about
legibility — that half a second of prose arriving as one row reads the same as
prose arriving as characters — and the operator, who is the party the judgement
was about, ruled it wrong. §10's criterion is *"when a surface needs a rate an
operator could not read at"*, and by that ruling the streaming tail is such a
surface: watching a model think is watching it write, and a row that appears
whole says nothing about whether anything is happening. So the lane was minted
exactly as §10 priced it and exactly as the paragraph above says it would have
to be — **the ball, not the migration**, which is why nothing above is retracted.
Everything the decline said about the *cost* was correct and was paid:

- **`Query::Follow` is the second follow-class read**, and the first whose
  subject is the world rather than a queue. The engine answers it with a frame
  per growth of the open `response.json` and no terminator until the stream
  closes at the step boundary. No wire change, no framing change, no new
  dependency: §3's *"a follow-class read is the general path with more frames"*,
  spent for the first time — until this ball, `Query::Invocations` was
  follow-class and still answered exactly one frame.
- **The lane is a second connection and a second thread** (`src/wire/lane.rs`),
  beside the asker and never inside it, because the asker's pass is serial and
  the paragraph above is right that a held read in it would stall every other
  surface. Re-ask is the whole reconnect ladder.
- **The pull path stays, and stays load-bearing.** `Query::Transcript` still
  folds the tail at `ASK_PERIOD`. A lane that is down, one whose stream just
  ended, and a window with no wire at all are one state at the seat: the chat is
  exactly what this migration left. The lane may fail without the chat failing,
  which is the only reason it is worth having.
- **The fold is untouched.** `inspector::live_tail` still decides whether there
  *is* a tail; what the lane adds is where the bytes are read from — the suffix
  on the writer's schedule rather than the whole file on the worker's — and the
  two agree by `Stream::absorb`'s contract, pinned by a test that folds the same
  bytes both ways.
- **What it cost that the decline did not price**: `Answerer::answer` hands back
  an iterator rather than a `Vec`. A materialized list has to be finished before
  its first frame can be written, so a read that answers *as the world changes*
  could not be one. That is the whole of the wire change, and it is a signature.

*And what the reversal did NOT cost, stated because §10 priced these too*:
per-request identity is unchanged (a held read is **one** request, and its scope
is spent at connect exactly as a one-frame answer's is), there is no
connection-scoped identity, no liveness protocol and no reconnect ladder beyond
re-asking. The pull model stays the rule for every human-cadence read; this is
one surface, named.

**A drive settles to a fixed point in one place now** (`World::drain`). bl-44e9
put that loop in `Screen::run`; the whole-window paint driver (`acceptance::mod`'s
`painted`) still counted three passes, and three stopped being enough the moment a
read *chained* — the step drill-in's sequence name is picked out of the step list
that landed, so its own question is not even declared until the pass that answers
the list. Two facts came out of writing it and both are stated rather than
rediscovered: a `Link` may never be settled twice without a frame between (the
second declares nothing and drops every answer), and a **fresh** `egui::Context`
measures before it paints — it sizes a content-sized panel and culls a scroll area
against the previous frame's rect, so the §11 chat read off the frame its answer
arrived on is read off an unmeasured layout. `painted` therefore pre-rolls two
frames, drains, and reads the frame after; `Screen` needs no pre-roll, its context
being persistent across the drive.

**What bl-13f9 did NOT take, and why it is a ruling and not scope** — *closed
by bl-48ae, below.* The seat's own `Query::Agent` —
`AppModel::focused_conversation` — stayed in process. It is
not a §11 tab read: it is the *selection's* own view, and its consumers are
frame-synchronous in a way the tab reads are not. The composer's target line names
the conversation off it (a name that blinked between "start a conversation" and
the real one for a round trip would be a regression, not a cost), the §11 focus
walk unfolds a selected member's ancestors off it (an unfold that lags the
selection by an ask period is the visible-selection invariant broken), and two act
gates read it at click time (`x` stops the selection iff `seat.stoppable`). Three
of its seven callers hold `&AppModel` rather than `&mut`, so migrating it is a
signature cascade *and* a rendering ruling about what a seat shows between the
click and the answer. That is bl-f297's dissolved class 3 — *a consumer that reads
it synchronously* — reappearing at a different noun, and it wanted the ruling
first. It was the residual that ball left, and it was filed as **bl-48ae**.

**The seat's own read is gone, and it did not become a question (bl-48ae).**
The ruling is a **split by who needs it when**, and the two shapes bl-48ae
weighed against it are recorded as declined: a *latch* (bl-f297's marks-pane
shape) answers the gates and answers neither the name nor the unfold, both of
which are wanted on the frame the selection changes; and a seat *holding its
last answer* under the new selection's name — a surface that lies for half a
second — is worse than one that is blank. What made a third shape available is
that the payload work had already been done: since bl-44e9 `Query::Conversations`
lands the **whole descent forest** with per-row rollups, and a `ConvRow` has
carried the §8.2 gates since bl-1eb0. So the frame-synchronous half was never a
wire read at all — it is a *selection out of an answer the §11 list is already
holding*.

- **`nav::convs::selection` is `visible`'s sibling** and the same kind of thing:
  a pure fold over an answered forest, `visible` keyed by which rows are open and
  this one by which row is picked. The composer's `→ message <name>` line, §11's
  ancestor unfold (`shell::focus::ancestors`), `x`'s `stoppable` and the §3.6
  danger row's root all come off it, in the frame the selection changed, with no
  new ask and nothing latched. Depth *is* the parentage — the answer is
  pre-order, so a row's ancestors are exactly the shallower rows above it, which
  is `parent_of`'s own rule iterated to the root — and `answer::agent`'s
  *absence is a value, not a refusal* ruling is kept verbatim at this altitude.
- **Nothing was added to `ConvRow`, and nothing was taken off `AgentView`.** The
  ball's fallback was a payload move; it was not needed for the four consumers
  that forced the ruling, and it would have been wrong for the residue. What the
  forest does not answer is `tip`, `marks`, `held` and `nudgeable` — facts about
  **one agent** rather than about the list, one of them a git oid and one a
  parked invocation blob, and putting either on every row of a workspace's forest
  to serve the one row that is selected is the altitude mistake `ConvRow`'s own
  definition (*the projection of one subtree to what the list paints*) exists to
  prevent. They ride a standing `Query::Agent` at the seat — the first window
  consumer that query has ever had.
- **The parity is pinned, because two projections of one derivation are two
  facts waiting to disagree.**
  `boundary::answer::agent::tests::the_forest_answers_every_fact_the_seat_reads_off_the_selection`
  asserts all nine shared fields equal between `answer::agent` and
  `nav::convs::selection` over a root, a member and an id nothing carries. That
  is what licenses the split: the two answers are one derivation, projected
  twice, and a change to either that moved them apart reddens.
- **The rendering ruling, stated once so no later reader re-derives it: a fact
  that gates a gesture is read off the forest; a fact that only paints may land
  an ask period later.** Everything on the `Query::Agent` side paints an
  affordance rather than judging one — an unpainted button cannot refuse a
  click — so a freshly-selected conversation shows its §6 marks, its §9.4 model
  row and its `Nudge` button half a second after its name. That is the same beat
  the transcript beside them has kept since bl-13f9, and it is the general rule
  at one more surface rather than a case of its own.
- **`AppModel::focused_conversation` is deleted, and the subtraction is the
  proof** (bl-1747's precedent). Its seven callers took the split; three of them
  (`shell::{focus,delete_agent,settings}`) took `&mut AppModel` with it, which is
  what a read that crosses the boundary costs a signature and is the cascade the
  ball named. `src/shell/seat.rs` is the one place either half is spelled.
- **Four bespoke acceptance drivers learned the harness ruling** (§9.8's, as
  bl-44e9 extended it to reads): `settings`, `drift`, `inbox_composer` and
  `one_rendering` each ran their own counted frames and settled no wire, so a
  seat whose subject is now an answer painted nothing in them. They pre-roll and
  then `World::drain` to a fixed point like `acceptance::painted` does. A driver
  that counts frames is fine; one that counts frames *and* reads a migrated
  surface is not.
- **One ast-grep rule gained a carve-out, keyed to the enum path rather than to
  a file.** `no-engine-tree-in-paint` forbids paint code naming `GitTree`/`Agent`/
  `CommitNode`; `Query::Agent` and `Reply::Agent` are the words a seat writes to
  declare *a payload the codec spells in both directions*, which is exactly what
  the rule exists to permit. The exception matches only an identifier whose
  scoped parent's path is `Query` or `Reply`, so `crate::git_tree::Agent` is
  still flagged everywhere it appears; the fixtures still bite and `src` is
  clean, which is the two-direction check.

**The accessory tail, part one: the folds that needed no question (bl-296f).**
Seven of the tail's accessors are gone and the boundary gained **no new
query**, which is the finding rather than the scope. The tail was filed as
*"each a `Reply` that does not exist yet"*, and for these seven that reading was
wrong: the facts were already on answers this window had landed, and what was
missing was a seat willing to read them there. Where a payload grew it grew by
**one field on an existing answer**, never by a question of its own.

- **The §11 tab bar and the attention strip are one `Query::Workspaces`, and
  class 2 is closed.** `WsRow` gained the §4.1 **pin rank** — `pinned:
  Option<usize>`, its place in the durable pin list, absent-not-null so rank 0
  is not read as unpinned — and `nav::tabs::build` now folds the answered rows
  rather than a snapshot: `Item` is deleted, the bar has no path in it anywhere,
  and `strip_total` beside it is the same rows summed. The **rank** rather than
  a flag is the whole of why no join is left at the seat: hoisting in pin order
  off a boolean would mean re-reading `ui.json` to sort, which is the seat
  joining an answer back against the engine's own document — bl-7407's refused
  shape reappearing at the field it was refused for. A pin is lawful on the wire
  where the §11 expanded set is not, and the test is DESIGN §5.3's own: a pin is
  **durable operator state**, living in the document whose §6 acknowledgements
  this row's `attention` already folds, where a fold is a viewport's.
  A **stale pin key dissolves** rather than being skipped — a key naming no
  enumerated workspace ranks no row, so the seat has nothing to drop.
- **The §3.4 raise claim needed a second projection, exactly as the echo did.**
  Migrating the bar re-opened bl-9acf on the first run: a wall `lernie new` has
  just founded is in no answer for as long as one derivation takes, so it wore
  no tab and its name resolved to nothing, and the composer's bare Enter would
  fire into the previous wall. bl-44e9's ruling covers it verbatim — *a seat's
  optimism reaches whatever that seat actually reads* — so `AppModel::raised_rows`
  is `echo::rows::with_echo`'s shape one noun up, folding the claim into an
  answered listing. It cannot double-count: the claim is retired against the
  same derivation the answer is made from.
- **The activity chip and its Dismiss fold the trail they summarize.** The chip's
  counts were an in-process fold over the window's own snapshot while the rows
  under them came off the wire — two readings of one tail, and the one number an
  operator could catch lying. `activity`/`has_alarms` are gone; the seat folds
  `Query::Ops` with `opslog::activity`, and *alarming* is now a method on the
  summary so the ichor and the button cannot come apart. The question therefore
  stands whenever the chip paints, which is every frame; expanding the trail
  asks nothing new, a standing question being keyed by its own envelope.
- **The live mark and the in-flight strip are fields on `Query::Agent`.**
  `AgentView` gained `seats` (§5.1 #28b, one entry per agent in the conversation
  with what it is doing) and `strip` (§5.1 #28's characteristics), so the §11
  header's marks, its mark and the bottom strip are **one ask**. They ride the
  per-agent answer rather than `ConvRow` for bl-48ae's own reason: a per-agent
  activity list on every row of a workspace's forest, to serve the one row that
  is selected, is the altitude mistake `ConvRow`'s definition exists to prevent.
  The **cost is stated**: the strip's elapsed segment is stamped when the answer
  is derived rather than re-rendered per frame, so it advances at `ASK_PERIOD`
  — bl-13f9's live-tail ruling at a coarser unit, a figure that ticks in seconds
  read at half-second cadence.
- **The header's conversation ball was already on the row.** `ConvRow::ball` has
  carried the §3.3 stamp resolved through the §3.5 join since the list was
  written, so `conversation_ball` and the `resolve_conv_ball` behind it were a
  second read of an answered fact. It is `Selection::ball` now — the one field
  of that fold with no `AgentView` twin, deliberately, since a second copy on the
  per-agent answer is precisely the disagreement the parity test exists to catch.
  It lands in the frame the selection changes, which is where the ball line
  belongs: it sits under the name, and arriving late would read as belonging to
  the previous conversation.
- **One more bespoke driver learned the harness ruling** (`acceptance::bands`),
  for bl-48ae's reason at the fifth site: it counted frames and censused which
  bands each window size seats, and the strip's subject is an answer now, so the
  band it exists to place was missing for the transport's reason. It drains to a
  fixed point like the rest.

**The residual, as bl-296f leaves it, and it is the accessory tail minus the
seven above** — filed whole as **bl-b4b5**, and closed by it below.

- **The §3.2/§3.5 ball-and-spend family** — `ws_balls`, `roster_ball_rows`,
  `bound_ball`, `focused_join`, `ball_spend`, `conversation_spend`,
  `conversation_context`. The altitude is the open question and it is worth
  taking as one: `Query::Balls` already answers the join rows, but it answers
  them **path-typed** (`JoinRow` carries `project` and `workspace` as
  `PathBuf`s — §8.1's list did not reach them), so a seat holding a §3.1 *name*
  cannot select its workspace's balls out of that answer without the join
  bl-7407 refused. So this group is one workspace-addressed question answering
  the bound balls with their figures, not eight small ones, and the §8.1
  narrowing of `JoinRow` rides with it. The two conversation-scoped figures
  (`conversation_spend`, `conversation_context`) are `AgentView`'s shape — facts
  about one conversation's subtree, like `flight` and now `strip`.
- **The §3.6/§8.4 fire-time gates** — `delete_confirmation`,
  `agent_delete_confirmation`, `move_targets`, `conversation_names`. The
  rendering split is already exact and must stay so: the chokepoint re-derives
  every confirmation **fail-closed** at fire (§9.8, bl-1747), so the seat's copy
  is a painted affordance and may land an ask period late — which is what makes
  these ordinary reads rather than a class of their own.
- **The misc singles** — `focused_pending` (the composer's §5.1 #11 queue),
  `agent_titles` (the §3.3 table a seat resolves a *third party* against), and
  `config_tip` (the §2.2 lineage tip the §9.4 row's drift clause reads).
- **`staleness`, and the one thing in the tail that is not scope.** The §7.2
  staleness line is the age of `Snapshot::derived_at`, an `Instant`; the
  chokepoint takes `now_unix`, an `i64` minted at the process boundary
  precisely so every derivation is deterministic under test. So this read cannot
  be answered without the snapshot carrying its completion as a **wall-clock
  stamp**, which is a payload-and-clock change rather than a read migration, and
  `growth_note` beside it is painted in the same two-line loop and moves with it.

**The accessory tail, part two: the tail is empty (bl-b4b5).** Every accessor
above is gone, the boundary gained **one** query, and the finding is the ratio:
of four groups, three were folds over answers this window was already holding
and only one was a question nobody had asked. Where a payload grew it grew by a
field, which is bl-296f's own rule applied a second time.

- **The ball-and-spend family is one workspace-addressed question**
  (`Query::WorkspaceBalls { workspace }`), and the eight accessors were never
  eight questions. The answer is the §11 balls section's whole content — every
  ball the workspace holds, with its badge, its project, its claimant and **its
  §3.5 figure** — and `roster_ball_rows` / `bound_ball` are pure selections out
  of it (`nav::balls`), exactly as `visible` and `selection` are selections out
  of the forest. The figure rides the row rather than earning a read of its own
  because it is a filter over the very `Snapshot::bills` walk the listing is made
  from: asking separately would be the two-readings-of-one-derivation defect
  bl-296f closed at the activity chip. The two conversation-scoped figures are
  fields on `AgentView`, for `seats`/`strip`'s reason, and they land an ask
  period after the name like everything else on that answer.
- **`JoinRow` says names now, and the last path-typed payload field but one is
  gone** (§8.1). `project` is the §5.1 #1 wire name and `workspace` the §3.1
  leaf — the *same two words* `Action::Close`/`Assign`/`Move` already take — so
  the seat's ball row hands its verbs a name it was answered rather than a path
  it resolved, and `Reply::Balls` is finally readable by a client that holds no
  world. `BoardRow` narrowed with it (it copies both fields off the join), and
  `delete::Claim` did too, which is what let the §3.6 confirmation become
  buildable by a seat at all. The engine resolves either back through
  `Snapshot::project_path` / `ws_path`, the one seam that owns the round trip.
- **The fire-time gates needed no query, and that is the finding.** Every one
  was already answered: `move_targets` is the enumeration minus the holder
  (`nav::tabs::move_targets`), the §3.6 scope is `WsRow`'s own `kind`, the
  workspace confirmation is the enumeration + the descent forest + the balls
  listing folded into the *same* `Confirmation` type the chokepoint gates on
  (`delete::confirmation_of_rows`), the conversation one is the forest's own
  subtree run (`delete::agent::confirmation_of_rows`), and the §3.3 occupied set
  is every answered row's stored name. Two constructors, one type, one
  `armed`/`refused` — a second type would have been two representations of one
  gate. **The chokepoint's re-derivation is untouched and stays authoritative**;
  what moved is the painted affordance, and it may be an ask period behind
  because an unpainted button cannot refuse a click.
- **The misc singles split the same way.** `agent_titles` is `Titles::of_rows`
  over the landed forest — a row's `display` *is* the ladder's answer for its own
  agent, so the answered list already carried the table. `focused_pending` is
  `Query::Inbox`' answer, which is the §11 Inbox tab's own standing question, so
  the composer and the tab are one ask; it re-reads the deposit directory where
  the accessor folded the tree's gathered copy, which is the same rows one
  derivation fresher. And `config_tip` — bl-1eb0's named residual — became a
  **field on `WsRow`**: the §2.2 lineage tip is a fact about a *workspace*
  exactly as `running` is, and `Query::Workspaces` is the question that answers
  those. Its altitude objection does not apply, since this is not a per-agent
  fact pushed onto every row of a list to serve one.
- **The §3.4 echo needed a third projection, for bl-44e9's reason a third
  time.** The composer's queue reads an answer now, and `echo::compose`'s fold
  lands on the *snapshot* — so a typed message would have vanished for an ask
  period between Enter and the deposit file, which is the §11 faded-send ruling
  deleted at the one surface it exists for. `AppModel::echoed_pending` folds the
  echo's deposit onto the answered listing; all three projections stay in the one
  module, which is `compose`'s own single-source argument kept.
- **The staleness stamp is the payload-and-clock change the ball named, and it
  is a subtraction.** `Snapshot::derived_at` was an `Instant`; it is
  `derived_at_unix` now — one stamp, in the unit both ends of the wire speak
  (DESIGN §7.2). The two §7.2 lines then ride `Query::Workspaces`' answer beside
  the rows, because that is the one read every window makes every frame and the
  currency of an answer costs it nothing to say; a question of its own would have
  bought a round trip per ask period for two lines that are usually absent. They
  cross as the **rendered** lines for `FlightStrip::facts`' reason: the wording
  is one derivation's, over a bound the operator tunes in `cadence.yaml`, and a
  wire spelling of the parts would be a second place that decides when a
  derivation is late.
- **A test double had to grow a wall clock, and the reason is the ruling.**
  `FakeClock::stamp` is the opaque `"TS"` sentinel every ops assertion spells, so
  the default `Clock::unix` would have pinned every derivation's completion at
  epoch zero and no §7.2 staleness line could ever be observed under a clock the
  test moves. The fake answers `unix` from its own advanceable instant instead —
  epoch-relative, because every reader of it computes a *difference*.
- **One more driver pair learned the harness ruling, and a second kind of
  settle appeared beside it.** `acceptance::painted` and `inbox_composer::Frames`
  both drain the wire already; what they did not settle is the composer's fold
  line, which is last frame's *painted* content height eased over `i.time`
  (bl-929d) and therefore changes on the frame the inbox answer lands — after
  the drain's fixed point. Both now run trailing **clock** frames. `Frames`
  settles the wire between them because its beats deposit between settles;
  `painted` deliberately does not, because a settle on a frame the steps answer
  has not reached reads as *not wounded* and closes the §7.3 grace window
  (`app::grace`).

**What is left of §9.7, in full.** Two residuals, and neither is scope:

- **The §9 config editors' own loads**, which were never one of the three
  classes (bl-f297's ruling: pointing them at `ReadConfig`/`Lineages`/`Models`/
  `Providers` changes what a config editor *is*, and is a separate design).
- **`Prepared::binding` stays a path** (§8.1's ruling, unchanged), affordable to
  dissolve only once one seat's read path is the *only* read path — and the
  composer's own fire-time inputs (`startable`/`resumable`, the §8.5 line
  context) are the acts side of bl-adcb's line, which no read ball reaches.

Everything else the read path ever named has crossed, been folded, or been
ruled on.

- ~~Gestures are still dispatched in process~~ — **the design and the first
  group landed, §9.8** (bl-4841). An act is a declaration whose receipt lands
  later, under a ticket the frame mints; thirteen gestures cross the wire and
  four are left, each named there with the frame-side fact its receipt gates.
  `AppModel::dispatch` stayed until those four moved; **bl-1747 moved them and
  deleted it** (§9.8), so the window's dialogue is wire-only in full.
- **`Prepared::binding` stays a path** (§8.1's ruling, unchanged, re-checked
  bl-44e9). The shape that dissolves it — `Prepared` becoming opaque to the seat
  — is affordable only once one seat's read path is the *only* read path, and the
  composer is not migrated yet. It is a disclosure of an engine-side directory
  name to a seat already told the workspace it belongs to, and it stays bounded
  and stated.
- ~~Search rides a snapshot of its own~~ — **landed, bl-44e9**: the `Searcher`
  asks the engine.

### 9.8 The acts, and the receipt that lands later (bl-4841, closed bl-1747)

§9.7 moved the window's *reads* onto the wire and left its *acts* where they
were: every gesture still went through `AppModel::dispatch`, the in-process
`boundary::dispatch` over this instance's own `ui.json` — the second execution
path §1.2 exists to refuse, and the larger half of the residual by volume. It
could not simply be re-pointed, because the click-glue read each act's `Reply`
**synchronously** and branched on it in the same frame, and a frame may never
wait on a socket. Four questions were answered before anything was built.

**1. The receipt's identity is a minted ticket, and could not be anything else.**
The read path keys a standing question by its own encoded envelope, because
asking twice is asking once. An act cannot borrow that rule: a gesture is not
idempotent — two clicks of Nudge are two nudges — and a resend is never free, so
nothing about an act's own bytes can be its handle. Something has to *mint* one.
It is a `Ticket`: a number from a counter the frame owns, minted at the send,
spent at the read, and mintable by nothing else. What makes it survive the
repaints between the two is that the surface holds it in **its own RAM**, beside
the status line it already held across frames — the frame it was clicked in is
long gone by then.

It is deliberately **not** bl-024b's invocation id. That handle names a slot in
the engine's mailbox: a fact about the world for the seconds a tool takes,
readable by the client that posted it, swept after an hour. A receipt is a fact
about *this window between two frames* — it never crosses the wire, and nothing
but the frame that minted it can name it.

**Every ticket earns exactly one receipt, so "never answered" does not exist.**
The poster is the only thing that can answer and it answers on every path it
has: the engine's reply, the engine's refusal, an answer the codec could not
read, a socket that would not open. A send that cannot even reach the poster —
a window with nothing behind its end of the channel — is answered *in the send*.
So there is no timeout, no clock and no expiry sweep in the act path: the
special case dissolved rather than being answered, and the one bound that
remains is `Seat`'s own read timeout, which turns an engine that has stopped
answering into a sentence.

**2. In flight is the sentence the click already wrote, with a mark on it.**
The surfaces that read a receipt at all are the four that paint one — the marks
pane, the picker's two writes, the lineage config editor — and each already held
a status line across frames. So the fire writes **what a clean landing means**,
which is known at the click, and the line carries an ellipsis until the receipt
lands: a clean receipt drops the mark and moves nothing else, and anything else
appends the reason. No second phrasing to learn, and no state the reader has to
be taught. Every other act paints nothing new, because none of them ever did:
their durable record is the `ops.jsonl` line the §7.3 banner reads back (INV-2).

**The §3.4 pending echo is neither replaced nor generalized, and that is the
ruling.** bl-adcb found that a migrated read paints the derivation, so the
window's optimism — the echo, the §7.2 live tail — is a fold a seat makes for
itself, and asked whether an act's receipt is what the echo was standing in for.
It is not: they are two facts at two rates. An **echo** stands in for a
*derivation* that has not caught up, and is retired by the world showing the
conversation. A **receipt** stands in for an *engine* that has not answered, and
is retired by the answer. They compose — a fire may hold both — and collapsing
them would make a conversation appear on the strength of a gesture having been
*sent*. What does change is the echo's trigger: one held today on a synchronous
`Ok` must be held on the receipt instead, which is the move the fire-and-hold
flows make when they migrate.

**3. `ui.json`: the engine writes and the window adopts, which is the ordering
two faces already have.** In process, dispatch took `&mut self.ui` — the
window's own document. Over the wire the engine opens the file fresh per
gesture, writes, and the §7.2 worker carries the bytes back as an external
change; `adopt_ui` takes them wholesale unless they hash to what this window
last wrote, which an engine's write never does. So an act's own write reads back
as an external change and the window **adopts** it rather than fighting it —
whole-file last-writer-wins is I5, and the window's own writes are write-through
at each set, so the file the engine opens already holds them.

What remains is the window's own direct writes (the per-frame §6 ack, panel
sizes) landing between the engine's write and the window's adopt, where a
re-save carries the older copy back. That race is **not new** — it is exactly
the one the `gestures/` inbox consumer already has with the frame — and it is
bounded by the derivation cadence. It was untouched by bl-4841's landing — no
act that crossed there wrote `ui.json` at all — so the ordering was stated
rather than discovered, and **bl-1747 paid the check**. The writers turned out
to be three, not four: `MarkSeen` and the two §3.6 deletes. `Prompt` reads the
§3.5 ceiling out of the document and writes nothing, so the file it opens is
one it never dirties. All three cross now, and the residual's own entry records
what the check found: the prune lands whole on the next pass and nothing
fights.

**4. Fire-and-hold is the precedent in structure, and not the mechanism.** The
start claim and the pending echo already hold state across frames awaiting a
derivation, and that is the shape everything converges on — a click that cannot
finish is a fact held until something retires it. But they are not the
mechanism, because what retires them is the world catching up, not an answer
arriving. They survive unchanged; the general handle is the ticket.

**What landed.**

- `src/wire/post.rs` and `src/wire/poster.rs` — the act path's two halves, the
  read path's shape pointed the other way.
- `src/app/acts.rs` — the model's act half: post a gesture, hold what its
  landing re-derives, hand the receipt to whoever kept the ticket.
- `src/shell/act.rs` — the shell's one spelling of an act, `src/shell/wire.rs`'s
  twin: `fire` for the act whose receipt is nothing, `Held` for the act whose
  receipt is a sentence.
- `src/engine/window.rs` — the engine's hand-overs to a window, both ends off
  the one loopback seat, split from `engine.rs` at DESIGN §12's budget.

**The poster is a thread of its own, and that is a ruling.** An act runs as long
as the verb behind it — a `bl close` runs a project's gate — so posting it on
the asker's thread would stall every standing read for the duration, which is
the frame going blind because the operator clicked something. Acts are
serialized among *themselves*, one connection at a time in the order they were
clicked, which is strictly less blocking than the in-process dispatch it
replaces: there, the **frame** waited. Connection-per-act, like the asker, so
§10's ruling is untouched.

**The `Cli` pair evaporates as a surface migrates**, and it is worth recording
because it is the split becoming visible in the code. A dispatched act needed
`boundary_deps` — the verb binaries *this box* resolved; a posted one carries
the gesture and nothing else, because the engine owns the binaries and a seat
never did. Ten click-glue files lost the parameter in this landing and none
gained one. A remote seat could fire every act listed below.

**What crossed in bl-4841**, by the line that decided it then — *an act whose
receipt is nothing, or a sentence, crossed; an act whose receipt gates a
frame-side state change did not*. That line is retired: bl-1747 crossed the
other side of it too, so **everything crosses** and the list below is history
rather than a boundary.

- the §8.2 short verbs — `Stop`, `Scan`, `Nudge`, `AnswerHold`;
- the ball verbs — `Close`, `Release`, `Move`, `Assign`;
- the V2 fork cohort — one `Fork` per candidate, in the order composed;
- the four that paint a sentence — `SetMarks`, `PickModel`, `Retarget`,
  `ApplyConfig`.

**The residual, closed (bl-1747): none.** Four were left, and none of them was
more of the same move — each is recorded here with what its receipt turned out
to owe, because the answers are the ruling and the code is only where they live.

- **`Action::Message`.** Its reply gated two frame-side facts in one breath: the
  draft clearing (a refused send leaves the words on screen to be fixed) and the
  §3.4 pending echo. The rendering question — *does a draft stay up for one
  round trip?* — is answered **yes**: it stays until the receipt, and a refusal
  keeps it for good. That is §5.3's own sentence read literally (*a draft is RAM
  until sent*) with "sent" meaning what the engine says rather than what the
  click hoped, and it is the only answer that cannot lose typed text. The echo
  moves with it, by ruling 3: same fact, same rate, new trigger.
- **`Prepare` / `Prompt`.** The frame-only aftermath — the §3.4 workspace
  adoption, the held start claim, the §3.3 mint seed a landed fire spends —
  rides the receipt now, and the typed doors the window used to enter through
  are gone from the seat side (the chokepoint still has them; the `Prepare` and
  `Prompt` arms are their one caller). **The seed forced a reframe and got the
  right one.** It was a `Deps` field, which is the engine's to fill — but the
  window's greyed §3.3 preview is drawn off it, and a seat that predicts a name
  must be able to fire *that* name. It rides `Action::Prompt` as
  `seed: Option<u64>`: a parameter of the gesture, not of the environment, with
  `None` for a caller that predicted nothing and one stamp-derived default at
  the one door that mints. `Deps::mint_seed` is deleted, and with it the three
  intakes that each spelled the same default. **The composer's Enter is two
  acts**, the `Prompt` posted when the `Prepared` lands; that chain is held in
  one place so the draft it composed empties when the *second* one lands, and
  the gesture is judged once exactly as the synchronous pair was.
- **`DeleteWorkspace` / `DeleteAgent`.** The modal holds a ticket beside the
  sentence it already held, disarms while one is outstanding, closes on a clean
  receipt and keeps the reason otherwise. The fire-time re-derivation stays
  fail-closed and unmoved — it is inside the chokepoint, so it now runs where
  every other seat's does. **Answer 3's ordering was checked here and holds.**
  These are the only acts left that write `ui.json` (`Prompt` reads the ceiling
  and writes nothing), so this is where the check was owed: the engine opens the
  file fresh and prunes, the window keeps its own copy inside the frame, and the
  §7.2 worker carries the pruned bytes back as an ordinary external change that
  `adopt_ui` takes wholesale — the window's own last write being a different
  document, so the echo test never suppresses it. Nothing fights, and the
  regression naming that is
  `app::tests::deletes::the_engines_prune_reaches_the_window_as_an_ordinary_external_change`.
- **The slash line's act arm.** Same shape as `Message` and answered the same
  way: the typed line clears on a clean receipt and is kept, with the reason
  under it, otherwise. Its **query** arm is untouched — a read is answered in
  place or latched (§9.7), and only the act arm had a receipt to wait for.

**`AppModel::dispatch` is deleted, and the subtraction is the proof.** It was
exactly the size of that list. With it went `AppModel::fire_prompt`,
`prepare_start`, `delete_workspace` and `delete_agent` — the frame's four
private entrances into the chokepoint — and `Deps::mint_seed`. The window's
dialogue is **wire-only in full**, and §1.2 is closed to the letter: the engine's
own intakes (the deposit consumer, the listener) are the only callers of
`boundary::dispatch::dispatch` that remain, which is what having one execution
path means. The `Cli` pair went on evaporating as it was always going to
(§9.8's own observation): eleven more click-glue files lost it, and what still
carries it does so for the §8.5 line's *query* arm and nothing else.

**What it unblocks for bl-f297.** That ball's third blocked class — *a read with
nowhere to paint a refusal, or a consumer that reads it synchronously* — put the
§9 config family here, on the grounds that it is answered at click time. It
turns out to need no receipt at all: a click-time read is **a standing question
with a latch**, the click turning it on and the surface declaring it while it
paints, which is the same rule that makes a collapsed pane free. And where such
a read is really a *write's* own re-read — the marks pane's branch — the receipt
already carries it: `Reply::Marks` is the branch re-read after the write, which
is §5.3's receipt discipline (*a receipt is a re-read, never an echo*). One
fewer blocked class.

**The acceptance world answers acts too**, and since bl-1747 it does so for
*every* world rather than the wired ones: the two channel ends are taken at the
fixture's own boot (`shell::acceptance::fixture::world`), because a fixture with
nothing behind its end of the channel is a window whose every gesture is refused.
Two chokepoints — the standing questions through `AppModel::answer`, the posted
acts through `boundary::dispatch::dispatch` over a `ui.json` opened fresh per
gesture — with the transport stood in for and no second dispatch added. A read
that reaches the act queue, or an act that reaches the standing set, is refused
with the sentence saying so rather than quietly answered by the other half.

**A driven frame settles its own acts**, and that is a harness ruling worth
stating. A gesture's aftermath lands on a *later* frame reading the receipt, and
the start family is two acts deep. A window pays that in ask periods; a drive
would have to pay it in counted frames at every call site, which is a fact about
the transport leaking into every test that has nothing to do with it. So
`Screen::run` pays it once — answer what the frame posted, repaint, repeat while
anything was answered — and the loop terminates because only a receipt can post
an act and nothing in the window posts one unprompted. The engine's substrate
binaries live on the **world**, not on the driver, for the same reason a posted
act carries no `Cli`: they are the engine's, and a seat never had them.

## 10. Open questions (living)

- ~~The follow/streaming frame shape~~ — settled by bl-b6fa (§3): every answer
  is a frame stream terminated by a zero-length frame, so a follow-class read
  is the general path with more frames.

  **And it has its consumer now (bl-73e7).** `Query::Follow` is the first read
  that answers more than one frame — `Query::Invocations` is follow-class and
  still answers exactly one — so the shape this row settled ahead of any payer
  was finally spent, and it cost nothing it had not already promised: no
  version, no flag, no second reader. The one thing it did cost was on the
  *engine* side and was not priced here: `Answerer::answer` had to become lazy
  (an iterator, not a `Vec`), because a materialized answer must be finished
  before its first frame can be written and a read that answers as the world
  changes never finishes.
- ~~Whether a seat holds one connection across gestures~~ — **settled by
  bl-ccf7: it dials per ask, and the server already loops.** Two facts decide
  it. The listener's connection thread is `while let Ok(Some(request)) =
  read_value(…)` — request, reply stream, terminator, again — so a held
  connection is not a wire change, a framing change or a server change: it is
  `Seat::ask` keeping the `StreamOwned` it currently drops, and it can be taken
  the day it pays. And it does not pay yet: nothing polls. `yog seat` is
  one-shot by construction, and the seat that *would* poll is the window, which
  §9.7 says is not a seat until a ruling lands. Holding a connection ahead of a
  poller would buy a reconnect ladder, a liveness question and a
  connection-scoped identity — REMOTE §4 derives the client per request
  precisely so none of that exists — in exchange for a handshake nobody is
  paying yet. The criterion for revisiting is stated rather than felt: **when a
  seat's ask rate exceeds human cadence.**

  **Re-asked and kept by bl-ae05, now that a poller exists.** The window is that
  seat, and its rate is one pass per 500 ms over its standing set — which is
  human cadence by construction rather than by luck, and the criterion is
  therefore still unmet. Nothing was added to hold a connection: a handshake per
  ask on loopback is microseconds, and the reconnect ladder, the liveness
  question and the connection-scoped identity a held connection buys are all
  still costs with no payer. Revisit when a surface needs a rate an operator
  could not read at — which is what a follow-class read is for, and it holds its
  own connection by asking (§3).

  **Revisited, and one lane now stands (operator ruling 2026-08-22, bl-73e7).**
  The streaming transcript tail is the surface that needs a rate an operator
  could not read at, and it holds its connection by asking — exactly as this row
  said it would. Note carefully **what that did not change**, because the row's
  whole argument was about the costs a held connection buys and none of them was
  bought:

  - **Per-request identity stands, everywhere, this read included** (§4). A held
    read is *one request*: the certificate is read at its first frame and the
    scope is spent at connect, the same moment and on the same terms as a
    one-frame answer's. There is no connection-scoped identity and nothing
    remembers a peer between requests.
  - **The pull model stands for every human-cadence read.** The standing set is
    still connection-per-ask at 500 ms, and it is still the *fallback* for the
    tail itself — a seat that loses the lane keeps the chat.
  - **No reconnect ladder and no liveness protocol.** The lane re-asks; a stream
    that ended, a subject that moved and a dial that failed are one case. The
    engine's own bound is a quiet hold, and a frame written is what discovers a
    peer that went away, so nothing pings anything.

  The criterion is unchanged and still the thing to measure a future candidate
  against. One surface met it. The row stays open because the next one will have
  to argue the same case.
- ~~When polling graduates to a follow-class query~~ — **settled and built by
  bl-024b**: `Query::Invocations` is the first follow-class read with a
  consumer, and it needed no wire change. ~~What stays open is only whether the
  *transcript* tail follows it.~~ **It did: `Query::Follow`, by operator ruling
  2026-08-22 (bl-73e7, §9.7).** So bl-ccf7's one candidate and bl-c907's second
  are both built, and this question is closed in both halves. The reasoning,
  kept: bl-ccf7 said there was one candidate —
  the live model-call tail (§9 step 1's folded `Stream`), the one read whose
  subject changes faster than an operator looks, every other read being a
  projection of a snapshot the derivation worker republishes on its own
  schedule. **bl-c907 found the second, and it is the one with a consumer: a
  tool host waiting for its next invocation** (§3's routing-frame ruling). It
  needs no wire change either, and unlike the tail it has a caller that cannot
  be written any other way — a poll would be a machine asking a machine, which
  is the ask rate the bullet above set as the criterion.
- **Whether a tool host is a seat**, and whether it should run more than one
  invocation at a time. New with bl-c907, sharpened by bl-024b. A client may be
  both (§2 says the work laptop usually is), and the two are one identity
  holding connections at different moments, which the presence refcount already
  handles. What bl-024b found is that the question is not really about a
  *dedicated* connection: `yog tool-host` executes **serially**, so it holds no
  connection at all while a tool runs, and a second invocation simply waits in
  the mailbox. Concurrency there would want a worker pool on the client and
  nothing on the engine — the mailbox already hands out every queued
  invocation at once. Deferred until a host has two tools worth overlapping.
- ~~The tool-advertisement schema (the exact shape of name/description/input
  schema)~~ — settled by bl-4e08 (§5.1): three fields, `name` a single path
  component, `description` one string, `input_schema` the JSON Schema verbatim,
  in a `tools.json` document per client. How availability is spelled was already
  settled (§5, bl-bc7c): definitions frozen in the prefix, presence answered at
  invocation.
- Whether pins are world facts or pane facts (§7 defaults them to world, and
  bl-8bbc landed that default — moving a pin is one accessor, not a migration).
- Certificate hygiene: lifetime, rotation cadence, whether the CA distrusts
  or registrations carry the whole revocation load.
- **Whether a seat ever reads a foreign host's per-engine policy keys**
  (bl-aaec). §7 routes a workspace's world facts to its host and the pane to
  the box's own engine; the keys with no per-workspace subject (`ceiling`,
  `prices`, `identity_last_used`) stay the window's own engine's. Reading — or
  editing — another host's waits for the first surface that needs it.

## 11. Rejections

Recorded so they are not relitigated:

- **A second in-process face for the local window** — the split is the point;
  one method, one channel (§1.2). Challenged by bl-ccf7 (§9.7) and **upheld by
  operator ruling 2026-08-14 (bl-ae05)**: the challenge was that an in-process
  *transport* to the one `Answerer` is not a face, and the answer was to take
  the front door instead. The window is a wire client of localhost, nothing
  in-process was added, and this entry stands as written.
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
- **A "server" noun, or a per-server config object, on the client** (bl-aaec)
  — the client-side unit is the workspace by operator ruling; a per-server
  object would make the noun mean two things and reintroduce discovery
  through the channel. A server dissolves into the address inside each entry.
- **A client-side unit whose workspaces are discovered through the channel**
  (bl-aaec) — what a box participates in is what its entries state; what a
  host permits is what its registrations state; participation needs both,
  the same two-sided shape as certificate possession against issuer trust. A
  reads-what-it-could-join discovery surface is the convenience flow §1.4's
  posture exists to exclude, and it would derive the client's roster from a
  set the server owns — drift with an authority on the other end of a wire.
