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

   **The `enroll` act does not lift this, and the distinction is exact**
   (operator ruling 2026-08-30, bl-f4e3; §8.4). What this clause forbids is a
   **device enrolling itself** — a machine holding no material opening a
   connection and being handed some. That stays impossible rather than merely
   forbidden: with no certificate there is no handshake, so the new device
   performs no channel act of any kind. `enroll` is performed by an
   **operator-grade seat**, over a channel already authenticated by material
   that already moved out of channel, and what it produces is bytes the
   operator then carries — a QR on a screen is a screen-shaped `scp`, and the
   operator is standing in front of both machines. That is the same class as
   the boot-time mint §8 already blessed: the operator's own tooling, reached
   through the boundary because §3 forbids a capability that exists on the wire
   and nowhere else. Nothing is *learned* in channel that the operator's own CA
   did not mint one hop away, and no unauthenticated peer is ever answered.
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

litany bans "session" (DESIGN §1), and the transport/connection collision is
exactly the reason — the word appears nowhere here. The split adds three nouns
and reuses one:

| Noun | Definition |
|---|---|
| **client** | A machine holding an operator-issued certificate. One certificate = one client identity (its leaf name). A client is a fact about a machine, not a person — v1 has one human. **A certificate carries one of two grades** (§4.2, bl-1dd3): **operator**, which is the whole boundary within its registrations, and **foot**, which is the tool-host gestures and nothing else. This narrows the original sentence here — *"every certificate is operator-grade within its registrations"* — for the foot class only; an operator-grade leaf is unchanged in every respect. |
| **seat** | A client connection acting as an operator face: it asks queries, paints replies, dispatches gestures. (The word already names GUI/headless/line faces in DESIGN §8.5 — a remote seat is the fourth face of the same surface.) A seat is operator-grade by definition: a face that could not ask is not a face. |
| **tool host** | A client advertising tools into the workspaces it is registered in. One client may be seat and tool host at once (the work laptop usually is) — a machine wearing both roles wears them on **one operator-grade certificate**, because the roles are what a connection does, and the grade is what its leaf may do. |
| **thrall** | The **component** that is only a tool host (§5.4, §12; bl-1dd3): a separately installed foot whose entire wire surface is advertise, wait, complete. It is the noun for the installable; *tool host* stays the noun for the role, and the two are not synonyms — an operator-grade laptop hosting tools is a tool host and not a thrall. |
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
is a transport for that surface, not a vocabulary. It states exactly one fact
of its own — **the protocol version each end speaks** (bl-a670, below), which
is a fact about the transport's two ends and about nothing the boundary
says — and adds no verb, no field and no envelope:

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
  follow-class `Query` lands. (*"No version"* is about the **framing**: a frame
  never says what shape it is. The connection's version preface below is a
  different fact — who is speaking, stated once per connection — and it changes
  nothing about how a frame is read.) That promise was tested and held by bl-73e7's
  `Query::Follow`, the first read whose N is greater than 1: no flag, no
  version, no second reader were added. What the engine *did* have to change is
  that `Answerer::answer` hands back a lazy iterator rather than a `Vec` — a
  materialized answer must be finished before its first frame can be written,
  and a read that answers as the world changes never finishes. That is a
  signature, not a protocol.
- **Every connection opens with a version preface** *(decided and landed,
  bl-a670 — `src/wire/hello.rs`)*: **each end writes one frame,
  `{"protocol": <integer>}`, before it reads the peer's.** Both write before
  either reads, so neither waits on the other and there is no ordering rule to
  remember. **The version itself is not restated here.** It lives in
  `src/wire/hello.rs`'s `PROTOCOL`, with the changelog of what moved it beside
  it, and the subsections below record each bump as it was taken; this
  paragraph carried a literal `1` until bl-2410 and was five versions stale by
  then, which is what a second home for one fact does.

  **Why now, and not before.** Until the four-component split (§12) one crate
  shipped both ends of every connection, so the wire could not skew and a
  version would have been a field nobody could ever disagree about. Four
  separately installed components can skew, and the day yog, lernie, litany and
  thrall are upgraded on their own schedules is the day an unversioned wire
  starts answering an old question with a new meaning.

  **A mismatch is fail-closed, and the refusal names both versions.** No
  version list, no capability probe, no compat shim, no downgrade: the engine
  refuses in band on the connection the peer opened — *before* any gesture is
  decoded, so a request of a version this build does not speak is never
  adjudicated — and a seat refuses to its caller as the one `Err(String)` every
  other transport failure already arrives as (§9.7). Both say the same
  sentence, and it names **this end's version, the peer's, and the remedy**.
  **The refusal is the upgrade prompt**, which is the whole reason it must name
  a number an operator can act on rather than a code. Negotiation is the
  mechanism that makes every later version carry every earlier one's shape
  forever, and nobody here is paying for that: an operator who installs both
  ends can upgrade the older one.

  **A peer that states no version is refused exactly as a peer of the wrong
  one.** An unversioned build (a gesture envelope where a preface belongs), a
  frame that is not an object, and a peer that hung up mid-preface are one
  case, because none of them can be served and three sentences for one outcome
  is three sentences. The pre-version era is diagnosed rather than
  special-cased.

  **It adds nothing to the boundary, and that is not a technicality.** The
  preface rides *beside* the gesture envelope, never inside it, so the frame
  the wire carries is still byte for byte the frame the `gestures/` inbox
  carries and the codec gains no field — the version is a fact about the
  *transport's* two ends, and the disk inbox has only one. It costs no round
  trip either: a seat writes its preface and its request in the same breath and
  confirms the engine's on the way to the answer.

  **ALPN was the alternative and it cannot say this.** rustls will refuse a
  handshake whose application protocol does not match, at no cost and with no
  frame — but that refusal is a TLS alert, so neither end learns the other's
  version and an operator reads a transport error where a sentence belongs.
  Naming both versions is the requirement, so the preface is in band.

  **What bumps it.** A new `Query`, a new `Action` or a new reply kind is
  **not** a bump: the strict decode already refuses an unknown one in band,
  naming it, which is the boundary correcting itself rather than two protocols
  meeting. The integer moves when the *existing* shape changes meaning — the
  framing, the envelope, or what a spelling already in use is taken to say.
- **This document is the protocol authority** (bl-a670, with §12's split). All
  four components implement against REMOTE; **each repo's DESIGN governs only
  its own component.** Where a component doc describes the wire it **cites this
  section rather than restating it** — one fact, one home, because four
  restatements of a protocol drift into four protocols. What a component's own
  DESIGN owns is everything on its side of the socket: yog's world and its
  chokepoints, the seat's window, the engine's loop, the thrall's local
  execution corpus.
- **The vocabulary ships as a conformance corpus, and a change to a
  wire-visible shape bumps the protocol version** *(decided and landed, bl-32cb
  — `corpus/`, generated by `src/boundary/corpus.rs`)*. Four components
  implement one vocabulary, in more than one language, so the failure mode here
  is not a refusal — it is a quiet miss: a field one end drops and the other
  never notices, on a wire whose strict decode only ever sees what was
  actually written. A shared types crate was weighed and declined: it protects
  only same-language consumers (the android client cannot link it) and couples
  four release cadences for the one consumer it does protect. A corpus protects
  every consumer, because a fixture is data.

  **It is generated from the codec, never authored.** The values are the same
  round-trip surfaces the codec's own tests walk — one populated value per
  spelling, one entry per enum arm, the empty collection beside the populated
  one, the absent option beside the present one — so a fixture a client is
  judged against and a fixture yog proves itself against are one thing. There
  is no second list to keep true, and a spelling added tomorrow with no entry
  leaves its own encode arm uncovered, which yog's coverage floor refuses.

  **The layout**: `corpus/request/<op>.json` and `corpus/reply/<kind>.json`,
  one file per shape, each holding that shape's frames verbatim — byte for
  byte what a frame carries, with no wrapper — and stamped with the protocol
  version at which *that shape's* fields last moved. `corpus/shapes.json` is
  the standing record: per shape, its field signature and that version, plus
  the version the corpus as a whole is for. Keys sorted, one trailing newline,
  nothing derived from a clock or an address: regenerating on an unchanged
  boundary is byte-identical.

  **The rule, and where it is mechanical.** Any change to a wire-visible shape
  — a field renamed, retyped, gained, lost, or a spelling in use withdrawn —
  bumps `PROTOCOL`. That is the same rule the preface bullet above states; what
  the corpus adds is enforcement. A test regenerates and diffs, so a boundary
  change that alters an emitted byte fails until the corpus is regenerated; and
  because regenerating would otherwise erase the evidence, the standing record
  remembers each shape's field signature, and `make corpus` **refuses** to
  rewrite a shape whose signature moved while its own version stood still. Both
  failures name both halves of the remedy. A *new* shape is exempt, as the
  preface bullet already says a new verb is: strict decode refuses an unknown
  one in band, naming it.

  **What a client owes it**: decode every frame in both directories into its
  own types, and round-trip what it emits — decode then re-encode must return
  the frame exactly. A client that only sends requests still decodes the
  request fixtures; that is what catches a field it drops on the way out. A
  shape a client does not implement is still one it must not misread, so
  skipping a fixture is a decision recorded in the client, never a silent pass.

  **Where it comes from**: the yog repository, vendored or read from a
  checkout. There is no published artifact and no endpoint that serves it — the
  corpus travels with the component that generates it, which is the only copy
  that cannot drift from the codec. The consumers are the seat and the android
  app; the foot's surface is small enough that it may consume only the subset
  of shapes it speaks, under the sentence above. `corpus/README.md` carries the
  same contract where a client author will find it.
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
- **A lost reply leaves an act IN DOUBT, and the recovery is a read — never a
  resend** *(bl-d1f1)*. A connection that dies between the engine completing
  an act and the reply frame landing tells the client nothing about whether
  the effect ran, and nothing on the wire can be added to say it: an act is
  not idempotent (§9.8 — two clicks of Nudge are two nudges), an engine-side
  receipt journal could only assert *dispatched*, never *committed* — the
  effect belongs to a subprocess (`bl`, litany), so exactly-once is the effect
  owner's to provide and never the wire's to promise — and a stored reply
  replayed later would violate the receipt discipline (§5.3: a receipt is a
  re-read, never an echo) when the honest answer by then is a fresh read
  anyway. So no idempotency token rides the act envelope and no redelivery
  slot exists for acts: a client whose act earned a transport error instead of
  a reply paints the failure and consults the world, which is the durable
  record (§9.8: the `ops.jsonl` trail, the transcript, the roster — the reads
  are the recovery). **Asks are the opposite case and re-ask freely**: a read
  is answered in place, and asking twice is asking once (§9.7). The one
  deliberate exception is §5.3's invocation leg, at-least-once *by design*
  with the invocation id offered as the dedupe key — and thrall declines even
  that memory and re-runs (its DESIGN §3.8), which is this same ruling read
  from the other side. The disk inbox's cousin of this window is answered
  mechanically — a claim is lock-held, and a dead claimant's gesture earns an
  in-doubt refusal on its durable reply slot at the next engine boot (yog
  DESIGN §8.5) — because there a reply slot exists to answer into; the wire
  has none, so here the contract is this bullet, and it moves no wire-visible
  shape: `PROTOCOL` stands.
- **The disk inbox survives for in-world callers.** Agents drive yog through
  the `yog` PATH shim and the `gestures/` deposit inbox — same machine, same
  world, disk is the bus. The wire is for cross-trust-domain callers; the
  inbox is for the world's own residents. One boundary, two intakes, each the
  religion of its domain. (Certificates for every spawned agent would be an
  enrollment surface §1.4 forbids.) *(Landed, bl-b6fa: the two intakes are
  `yog gesture` and `yog seat`, and they share one argv reader and one
  answering function — `ConsumerCtx::answer` — so the second door opens onto
  the same room and can add no verb.)*

### 3.1 Stopping a turn: the envelope is the gesture, a deposited line is text (bl-1cf4)

Worked once, in full, because a client author was otherwise reverse-engineering
it from the codec: **a client that wants a stop control on an in-flight turn
sends `{"op":"stop","workspace":…,"agent":…,"children":<bool>}`, and nothing
else stops a turn.**

**A slash line never crosses the wire.** `/stop` is the boundary's third
serialization (`src/boundary/line.rs`) and it is read **at the seat**: `yog
gesture '/stop' --ws … --agent …` parses the line against the seat's own
selection and deposits the envelope it encodes to
(`sugar::argv::envelope`). Both engine intakes — a `gestures/` file and a wire
frame — meet at `consume::run_value` → `codec::decode`, which reads envelopes
only; a line handed to either is refused (`deposit is not JSON`). A client that
lets a human type `/stop` parses it on its own side, exactly as the argv seat
does, and `boundary::line::{is_command, unescape}` are `pub` for that: a
leading `/` is a command, a leading `//` is the escape that sends one slash.

**A `/stop` sent as `content` is a message, and this is the trap.**
`{"op":"message",…,"content":"/stop"}` decodes to `Action::Message` and spawns
`litany message <ws> <agent> /stop`: the text reaches the model verbatim (§3.3's
rule — a deposit's content is never read for flags) and wakes the very driver
the operator meant to kill. Nothing at either intake inspects `content`.

**Which of the three to send.** All three answer `Reply::Outcome` — the spawned
verb's `exit`/`stdout`/`stderr`, `ok` iff it exited 0.

- `stop` → `litany stop <ws> <agent>`, plus `--stop-children` when `children`
  is true, which cascades over the §2.3 hyphenated descent. `children` is
  optional and decodes `false` when absent; `workspace` and `agent` are
  required. One §4.2 ops row. Work the conversation already committed is kept.
- `interrupt` → `litany stop`, then `litany message`, carrying the same three
  fields `message` does and no `children` flag. The deposit is what restarts
  the conversation (ARCH §2.9: there is no resume verb), so this **replaces**
  the turn rather than ending it. **Two ops rows**, because the halves fail
  independently, and the reply is the *deposit's* outcome. A stop declined
  because nothing was running still deposits, so `interrupt` degrades to a
  plain send — that is the same gesture at zero work, not a case to branch on.
- `message` → `litany message`. No stop at all.

A stopped conversation is not a dead one: it comes to rest `stopped` and the
next deposit starts a driver again. A stop that lands inside a tool window is
settled in band by the pinned litany — one `is_error` `tool_result` per
unanswered `tool_use` — so the branch is left replayable and the deposit
revives it (`src/boundary/interrupt.rs` carries that ruling).

**When to offer the control.** `stoppable` and `stop_children` ride the
conversation reads — every `Reply::Conversations` row and `Reply::Agent`'s
`AgentView` — so a seat never derives the gate (§9.4: a gate that is not
derivable from a row goes on the row). `stoppable` is true iff that
conversation is `live` or `in-flight`, the two states where a driver holds the
executor lock; `stop_children` is true iff some other agent's id extends
`<agent>-` (`src/actions/enabled.rs`). A row's `state` is the badge aggregated
over the subtree and is **not** the gate: a quiet root with a working child
reads `live` and has no driver to kill. Firing `stop` anyway is not an error —
it is an `Outcome` with `ok: false` and litany's own words.

**The corpus carries all three request shapes** — `corpus/request/stop.json`,
`interrupt.json`, `message.json` — and `stop`'s fixture is the `children: true`
spelling.

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
it and the idempotent ensure's own `litany new` decides — finishing the dead
birth, or refusing in litany's own words with a logged ops row.

That refusal is the one place existence is observable to a scoped client, and
the ruling is that this is acceptable and bounded: **a namespace with creation
*by name* cannot also make a name's availability unknowable**, and §4 chose
creation. What a collision reveals is that a name is taken. It reveals no
workspace's contents, conversations, clients or registrations — everything §4
makes absent stays absent.

### 4.2 The two grades, and the foot (bl-1dd3)

**A certificate carries a grade, and there are exactly two.**

- **Operator grade** — the whole boundary, within the registrations §4 already
  scopes. This is every certificate issued before the split and every seat
  after it; nothing about it changes.
- **Foot grade** — the tool-host gestures and **nothing else**: `advertise`
  (§5.1), `invocations` and `complete` (§5.3). No other `Query`, no other
  `Action`.

**"No ask, no act" is the sentence, and the three verbs above are not the
exception to it — they are what it is measured against.** A foot cannot ask
*about the world*: not the workspaces, not the board, not the trail, not a
transcript. It cannot act *on the world*: no message, no start, no stop, no
ball, no config. What it may do is answer for the machine it is: state what
this box can run, wait for work addressed to it, and hand back what happened.
Note which of §5.3's four verbs is absent — `invoke`, the asking side's. A foot
is invoked; it never invokes.

**The grade is on the leaf, not in a registration and not in a config.** It is
issued out of channel with the certificate, by the operator's own CA, on the
same act §1.4 already requires — so making a foot into an operator is minting
a new certificate, which is exactly the friction that ruling wants. A
registration field would be a second authority for one fact and would let a
gesture over the wire widen the sender; a config file would put it on the box
being trusted.

**The spelling is the subject's organizational unit, read by the walk that
already reads the common name** (`registry::leaf`): `CN=<client>, OU=foot` is a
foot, and a subject with no `OU=foot` is operator grade. Two reasons, and the
second is the one that decides it. A custom X.509 extension or an EKU OID would
be the textbook home for a capability bit, but it needs an `openssl` config
stanza in the §8 mint recipe *and* an extension parser yog does not have, where
the OU is one more attribute in a subject the DER walk opens anyway. And
**default-operator, not default-foot**: a certificate minted before this
existed, or by a recipe that has not learned the flag, must keep working
exactly as it did — a silently demoted seat would be an outage with no sentence
attached, while a silently promoted foot cannot happen, because promotion
requires the operator's CA to have written the word.

**Enforcement is one raise at the chokepoint, and it is landed** (bl-7ff3),
where the client identity is
already spent for scoping (§9.6): the same `answer_as` that filters what a
caller sees refuses what a foot may say, **in band and naming the grade**. Not
absent-shaped, and that is deliberate — §4's absence rule exists so a scoped
caller cannot map what it is not registered in, and a foot asking for the board
learns nothing about the world from being told it is a foot. It made a category
error and the sentence is worth more than the silence, exactly as §5.1 rules
for an intake carrying no client identity.

**Three parts, as built.** The DER walk that reads the common name
(`registry::leaf`) reads the organizational units of the same subject and
answers a `Grade` rather than an `Option` — a leaf that says nothing is
operator grade, which is default-operator made total rather than defaulted.
What a connection carries into the boundary is therefore a `Peer` — the
identity and the grade, read off one certificate per request — and **not** a
field on the client identity itself, which keys the presence map, the mailbox
and every registration on disk: folding a second fact in would make a peer that
connected under one grade a different key from the same client read off the
`clients/` listing. The mint writes the word (`src/wire/provision.rs`, the one
`openssl` recipe): `WIRE_FOOT=1` beside `WIRE_LEAF=<common-name>` on `yog
wire-certs` puts `OU=foot` in the subject, and it is presence-shaped like
`FORCE` precisely so there is no word to mistype into a demotion. And the raise
sits in `answer_as` ahead of the dispatch and ahead of the create's
auto-registration, so a refused gesture founds nothing and seats nothing; the
follow lane needs no second refusal path, because a follow-class read is not in
the foot set, so the lane answers no stream and the fall-through `answer_as`
already there words the sentence.

**This is not the per-verb ACL §11 rejects, and the difference is not a
quibble.** The grade is **binary, closed, and in the code**: two values, one
fixed set of gestures each, no table, no configuration, nothing an operator
writes per verb or per tool and nothing that grows a row when a verb is added.
A new `Action` is operator-only by construction — the foot set is enumerated,
not subtracted from. §11's rejection is of a *policy layer*, and a policy layer
is the thing with a place to put a rule; this has none. §11's entry stands
unamended.

**The enrollment act is the first proof of that construction, and it cost no
row** (bl-f4e3, §8.4). `enroll` mints a certificate and seats a registration —
the most privileged thing on the surface — and nothing about the grade was
touched to keep a foot out of it. An act is not among the foot's enumerated
three, so `answer_as` refuses it in band, naming the grade, ahead of the
dispatch and ahead of the create's auto-registration; the executor writes no
check of its own, because a second authority for one fact is the one that
drifts. The inverse is worth stating too: an act a foot *may* say would have to
be added to that enumeration by hand, which is what makes the set closed rather
than default-open.

**What it closes.** §9.6 states a residual plainly: *"a client registered in
one workspace still reads the trail of every workspace"*. For a foot that is no
longer true, because a foot reads nothing at all — and a foot is the class most
likely to run on a box the operator trusts least (a build machine, a phone, a
box in someone else's house). The residual stands for operator-grade clients,
where it is the same ruling it always was.

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
  append-only was left to the litany seam; the invariant
  is that nothing but an explicit load ever changes the tool surface.
  **Settled by bl-c907 (§5.2): re-declared.** The set is a durable document the
  injection reads at every assembly, so the rebuild happens once, at the step
  after the load, and the agent chose it.
- **Locality rides in the name, and there is no second place to say it**
  (bl-71d0). A loaded tool is a **host-bound instance**: the name the model
  chooses already states which machine will run it. Two consequences follow and
  both are the point. Schemas stay **per host** and provider-validated at
  emission — there is no shared `host` argument for the model to fill in and get
  wrong, and no union of every host's schema under one name, which is what a
  name that did not carry its host would force. And no call has an *implicit*
  location: a model that can name a tool can say where it runs, before anything
  is routed. The spelling is §5.2's `<client>_<tool>`, unconditionally; the
  joiner is an underscore rather than a colon because the providers' tool-name
  grammars refuse a colon, and it is the qualification, not the punctuation,
  that is the rule. This is a naming discipline over the load mechanism §5.2
  landed (`list`/`get`/`load` on the one client-management tool), not a new
  surface — nothing here adds a verb, a field or a document.
- **A tool executes where its subject lives** — the subject-locality invariant
  (bl-71d0). A workspace-relative tool follows the worktree it reads; a tool
  about a box follows that box. **"Local" is not a privileged default**: the
  machine the engine happens to run on is simply the host that holds the
  subject in the ordinary case, and when it does not hold the subject it is not
  the executor. This is what makes the qualification above honest rather than
  decorative — the name states the host because the *subject* already chose the
  host, and a routing decision that disagreed with the subject would be
  executing somewhere the answer does not exist.
- **The environment is the executing end's own** (bl-71d0). An invocation
  carries its subject's location; everything else — `PATH`, the substrate's
  home, the state root, the credentials — comes from the machine that runs it.
  The server's composed world fold (DESIGN §16.2: `LITANY_HOME`,
  `XDG_STATE_HOME` and the rest of the nested world) is **never shipped across
  the wire.** It names paths that exist on the server and nowhere else, so an
  env that crossed would at best be inert and at worst point a remote process at
  a directory of the same name on the wrong box; and a server that folded env
  into another operator's machine would be administering a box it does not own.
  Substrate access therefore reaches an agent as a **tool the executing end
  advertises** (§12's thrall) or as an engine act, never as ambient inherited
  env. As the tree stands the executing end already supplies its own
  environment, and the subject location is the host config's own
  `cwd` (§5.2) for every cwd-less invocation. **Carrying the location on the
  *invocation* — the half this bullet said the thrall move owed — landed with
  bl-77be's worktree lane** (§5.4): an `invoke` may carry the conversation's
  resolved working directory, and the far end honours it only for an entry its
  own operator marked `subject_cwd` (§5.2's document; thrall's DESIGN §3.4).
- **No broadcast** (bl-71d0). The invocation shape names one addressee and takes
  no hosts list, in the wire verb (§5.3's `invoke`) and in the loaded name
  alike. One adjudication decision must stand for exactly one execution on one
  machine, and a list would make it stand for N on N — with one refusal, one
  deadline and one capture to describe all of them. If fan-out ever becomes hot
  it is a distinct tool that makes sub-calls, with its own name, its own schema
  and its own trip through the chokepoint, never an overload of `invoke`.
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
- **Invocation path — one pipeline, and there is no second one** (amended by
  bl-fe61 under §12's front-door invariant; it used to route *designated* tools
  this way and execute the rest in the driver). **Every** tool call the agent
  makes takes the same road: it hits the engine's injection seam →
  server-side adjudication (`yog tool-control`, unchanged, fails closed) →
  the driver queues the invocation in that client's engine-side mailbox → the
  thrall, waiting on its follow-class read, is handed it → it executes on its
  own box → it posts the capture back as an ordinary act →
  the driver's poll collects it. A routed invocation carries a deadline; a
  vanished client is a visible refusal, not a hang. **Nothing in that path is
  the engine speaking first** (§3).

  **The engine's driver keeps no local executor.** Not a fallback, not a
  fast path for the co-located case, not a residual for tools nobody routed —
  adjudicate → mailbox → execute → capture is the whole of it. A driver that
  could still run a binary itself would be a second pipeline with its own
  adjudication story, its own capture shape and its own containment claim, and
  the one an operator hit would depend on which tools happened to be
  designated. The engine does not care where execution happens: it emits the
  invocation and waits on the capture.

  **So a server with zero enrolled thralls refuses every tool call, in band.**
  That is §12's ship-inert posture working, not an error state and not a
  degraded mode: with nowhere to route, use-is-attempt gives the model an error
  tool result it reacts to, exactly as it does for a tool a client no longer
  carries. Enrolling a thrall — the local one included — is the explicit
  operator act that makes execution possible at all.

  **Six names reach that router and are not tool calls on a machine at all**
  (bl-dfce widened by bl-77be): the compactor's procedure pair and the four
  conversation-subject worker grants (`dispatch`, `message`, `load_skill`,
  `cd`), which yog answers itself as **engine acts** rather than routing
  anywhere. The road above is the road every call *on a machine* takes, and
  that is the whole of what "the same road" ever claimed; §5.4 records the
  ruling, the name-by-name subject audit and how the acts are performed. A
  server with no thrall therefore still compacts, dispatches subagents,
  messages, loads skills and moves its agents' working directories — and the
  worktree names (`bash`, `read_file`, `apply_patch`) take §5.4's worktree
  lane to the workspace's one consenting thrall.
- **Honesty about containment:** execution happens on a machine the
  adjudicator cannot inspect. Adjudication judges the invocation exactly as
  today; any containment beyond that is whatever the client enforces locally,
  and the design must not claim otherwise.
- **The driver-side seam was a litany ask, and it landed.** litany's driver
  executes tools; routing a designated tool to a remote executor needed an
  upstream seam. It is lernie 0.0.9's `Fx::tool_injection` (its
  `docs/DESIGN_TOOL_INJECTION.md`): **one** object carrying both halves — the
  definitions prompt assembly and the grant gate read, and the router the
  executor consults ahead of binary resolution — so a tool declared and not
  permitted, or permitted and not declared, is unrepresentable. An injected
  name outranks an elected one, the per-invocation disk record stays the
  executor's, and adjudication is untouched. yog owns the registry, the
  advertisement, the routing and the adjudication chokepoint; §5.2 is what it
  filled the seam with. **The seam's scope is what bl-fe61 inverts**, and only
  its scope: the same object, the same two halves, the same rule that an
  injected name outranks an elected one — but the router is consulted for every
  call rather than ahead of a binary resolution that no longer stands behind
  it. Deleting the executor is the engine's half of that ball and landed in the
  engine's own repo; this document is what it implemented against. **The pin
  that makes it load-bearing here is bl-dfce's**, which is also where the
  question the inversion raised — where the compactor's procedure pair belongs —
  is answered (§5.4).

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
- **`subject_cwd` is the one optional fourth fact** (bl-77be, PROTOCOL 2):
  `true` states that the advertising box consents to run this tool at a
  working directory the invocation names — §5.4's worktree lane routes on it.
  Absent reads false, rides only when true, and a mistyped value refuses at
  the read. It is advertised rather than kept local because it is the fact the
  ENGINE routes on; it stays checkable because the box that stated it is the
  box that enforces it (thrall refuses a carried cwd against an unconsenting
  entry, in band, naming the key).

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

**A set is not replaced under a machine that is serving, and one identity has
one reader** (bl-1462, twin of thrall bl-2d78). The store is keyed on the
identity and was last-writer-wins, so any connection bearing the certificate
could present the empty array and be answered `ok`; from then on every invoke
for that client was refused engine-side for a tool that plainly existed, while
the host sat there healthy and holding a full set. The host cannot notice on
its own — the bullet above is why, and its traffic reasoning is right: the set
is presented once per channel, and re-presenting before every read would double
the traffic to say a thing that had not changed. What that reasoning assumes is
that nothing else writes the set, and nothing in the protocol made it true.

The seam is drawn at the one moment the engine can tell a reconnect from a
usurper, and it needs no version, no generation and no receipt — three facts
yog would store and could not check:

- **A second concurrent `invocations` read is refused in band**, naming the
  client. Two parked readers under one identity is two processes claiming one
  machine's name; the newcomer would take work the first is waiting for and
  neither end would learn it. The claim is RAII inside the mailbox, released
  however the read leaves — presence's own shape, and there is no leave verb.

  **Its life is the hold and not the connection's, which is what makes the
  refusal retryable** (bl-0a74). `Mailbox::take` drops the claim on the way out,
  *before* the caller writes the answer, so a peer that vanished without a FIN
  frees the slot within one hold's width — thirty seconds — rather than when
  some later socket act finally notices. A machine redialling after a blip
  therefore meets this sentence naming *itself*, from a predecessor that is
  already dying, and the right answer is to wait and ask again. It is stated
  here because the alternative reading is catastrophic in exactly the case the
  guard was built for: a redial that took the refusal as final would make the
  first network blip permanent, which is the failure §5.3's reversal exists to
  end. It is a *contract*, not an implementation note — a refactor handing the
  claim back to the caller would break a foot on the far side of the wire with
  nothing here to catch it, so `slots.rs` carries the test that names it.
- **An advertisement that would CHANGE the set in force is refused while that
  client holds a parked read**, naming the client and both ways out. A host
  re-presenting an unchanged set writes nothing and never reaches the guard, so
  the ordinary reconnect pays neither a refusal nor a file read; a genuinely
  reconfigured box restarting is refused for at most the hold's width and lands
  on its next dial, loudly, rather than silently disarming whatever is serving.

**The receipt says whether the engine WROTE** (`{"kind": "advertised", "ok":
true, "wrote": true|false}` — bl-66d4, PROTOCOL 8). The answer used to be the
same `ok` whether the document changed or was found identical and compared, so
a box re-presenting an unchanged set and a box restoring a set another
connection had blanked read alike, and the second event — two processes
claiming one machine's name — reached no log on either side.

It is **false on the ordinary re-presentation**: every reconnect makes one, and
so does every §5.3 hand-off, since the foot re-asserts at the end of each
(thrall bl-2d78). A box that presents once per channel sees `true` on its first
presentation and never again. **A `true` on any later re-assertion is the
machine learning it was disarmed while it was absent**, and the sentence it
prints is the whole remedy — the window is the one the two guards above cannot
reach, because a foot executing a tool holds no parked read and `serving()` is
false for that tool's whole runtime.

It is not an echo and §8.1's test still holds: the stored set after the write
*is* the set the gesture carried, so answering the set would be one fact said
twice. `wrote` is a fact about the **document**, and it is the one fact in this
exchange the advertising box cannot compute for itself. **Required rather than
optional-absent-reads-false**, because absent would read as *"nothing was
restored"* — the reassuring answer — on exactly the build too old to tell; a
field that exists to make one event audible must not be silently false. The
engine already computed it (`registry::tools::store` answers "did it write" and
the dispatch discarded it), so what landed is one field on one reply and no new
verb.

Refusal rather than a trail note because both parties can act on a refusal and
only one of them reads a trail — and because §5.3's redelivery predicate reads
"the client asked for work again" as "it did not finish what it holds", which
is exact only while one connection at a time may ask.

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

**One tool, named `clients`, four ops.** Its subject is the roster, which is
why it is one tool rather than one per op — and why loaded remote tools still
surface as individually named definitions of their own. litany's
`docs/DESIGN_MCP_BRIDGE.md` §6 ruling binds a host too: a generic
`call {client, tool, arguments}` would collapse the role grant, the grant gate,
the tool control and every future policy into one bit.

**The same question was re-put by bl-71d0 as a *wrapper meta-tool* — one
declared tool carrying an untyped payload — and refused again, with two reasons
of its own.** It loses **emission-time validation**: the provider validates the
wrapper's schema, and the wrapper's schema says nothing about the tool actually
being called, so a malformed call becomes a runtime refusal on a far machine
instead of a model correcting itself before it emits. And it forces the
**adjudication chokepoint to unwrap a payload** before it can see what is being
asked — judging a string it must first parse, rather than a call. Its one
virtue is discovery without a server-side change, and that is exactly what
`load` already delivers on a surface the model reads and the chokepoint can
see. §5's host-qualified naming is the alternative, and it costs no new verb.

The declared schema:

```json
{"name": "clients",
 "input_schema": {
   "type": "object",
   "properties": {
     "op": {"type": "string", "enum": ["list", "get", "load", "unload"]},
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
- **`unload {client, tools?}`** — they stop being declared from the next
  assembly on. `tools` is optional and absent means *that client's whole loaded
  set*, which is the ordinary case: an agent that has finished with a machine
  has finished with all of it, and making it spell out what it loaded turns
  done-here into a recall exercise. An empty array is not that spelling — it is
  still an act with no effect and still declines, because a model that meant
  *all of them* must be told rather than answered with a no-op it will read as
  success.

Every answer opens with the instant it was observed at, because presence is true
only then and a line that did not say when it was read would be a claim about
now. Every refusal is in band and non-zero: an unknown op, an identity this
workspace has not registered (**absent**, §4 — the same sentence a name nobody
seated earns), a tool the client does not advertise, an engine that did not
answer. Nothing here ever mutates the prefix.

**A load resolves whole or not at all, and so does an unload.** Every named
tool must be advertised right now; one miss refuses the whole act, because a
partial load leaves the model believing it holds a tool it does not. The mirror
is the reason for the second: every named tool must be one the document
actually holds, and one miss refuses the whole act, because a partial unload
leaves the model believing it dropped a tool it still declares. The belief
desyncs in one direction or the other and neither is acceptable.

**The two acts resolve against different authorities, which is the whole
asymmetry between them.** A load's authority is the roster, so it needs the
engine and carries the observation's date. An unload's is the durable document
on the box the driver runs on, so it asks no engine and deposits nothing — a
finished host can be dropped while the engine, or the machine, is down. That is
the same reason this section already gives for declaring: *"declaring touches
nothing but disk, so a slow or absent engine can never change the prefix"*.

**The client is half of every name.** An unload names a client and, optionally,
tools *that client* contributed; two machines both advertising `Bash` are two
loaded tools, and dropping by bare tool name would silently change which
machine a name reaches. A client this conversation has loaded nothing from
refuses rather than answering an empty success — the wholesale form's version
of the same rule, and the only way a model learns its recollection was wrong.

**The presented name is `<client>_<tool>`, always.** §5.1 leaves the
cross-client collision — two laptops both advertising `Bash` — to the act that
loads one. Prefixing *conditionally* would make a tool's own name depend on what
some other machine advertises, so a name the model already learned could change
under it; one rule and no case. A composed name a provider's tool block would
refuse declines at the load, naming it.

**That unconditional prefix IS §5's host-qualification** (bl-71d0), and reading
it as a collision fix undersells it: the name is the locality, so there is no
tool a model can call without having said which machine answers, and no case
where two hosts' schemas have to be reconciled under one name. The underscore
is the joiner because it survives the providers' tool-name grammars —
`loaded::callable` refuses a composed name that would not (alphanumerics, `_`
and `-`, at most 64 characters; checked at the load act, not at the call, so a
model never learns a name a provider will refuse) — and the punctuation is the
only part of this that is negotiable.

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

**Subtraction is an explicit act, and there is no inheritance** (bl-3455).
`unload` removes entries from the same document; the next assembly re-declares
without them, one paid prefix rebuild at a moment the agent chose, exactly as
load's own settlement reads. It is still true that nothing but an explicit act
of the agent's own ever changes the tool surface — `unload` is a second such
act, not an exception to the rule. The set belongs to the agent that loaded it;
a fresh conversation, and a freshly dispatched subagent, starts clean and loads
what it needs through the `clients` tool every agent always has.

**Emptying the set is not a special case.** The last unload leaves an empty
array, and a document that is absent, unreadable or empty already reads as the
same nothing every agent reads before its first load — so no reader carries a
second case, and a load after an unload is an ordinary load.

**Why this is here and not in the model's own hands.** The operator's standing
principle is that context management happens in yog: what an agent declares is
a property of the conversation the server holds, so the act that changes it is
a tool this side offers rather than a discipline a model is asked to keep. That
also fixes what this op is NOT. It lands the edit immediately, and immediately
costs the prompt-cache rebuild that is the whole reason the loaded document
only ever grew. **Scheduling such an edit against a cache miss that was going
to be paid anyway is a different mechanism and a different ball** — bl-b6f9,
where a maintenance act queues against the agent's context and merges into the
operating branch at a moment the miss is already inevitable. Nothing here
defers, and nothing here should: a deferred unload whose queue does not yet
exist would be an unload that silently did not happen.

**The loaded set is declared for the driven agent, per assembly** (bl-fd24;
litany bl-ddaa). litany's seam asks `tools()` *for* an agent — the same
`workspace`/`agent` discriminants every routed call carries — so the injection
reads that agent's loaded document whatever verb fired the driver. Before the
amendment the binding read the pair off its own argv, `prompt` (which mints
its agent) answered none, and a conversation's first driver declared only the
`clients` tool for its whole drive: it could load — the load answered
"callable from the next step on" — and could never call, because the grant
gate enumerates exactly what `tools()` declared. The control that proved it:
the same conversation's identical load worked first try when a deposit resumed
it under a verb that named the agent.

**Where the injection runs, and what it may touch.** It is installed by the
`yog litany` arm (DESIGN §16.7 W11) at `Fx::tool_injection`, which puts it
inside the **driver** — a child process, not the engine. Two consequences, both
load-bearing. Declaring touches nothing but disk, so a slow or absent engine can
never change the prefix. Answering a `clients` op needs presence, which is
engine RAM by ruling, so it asks through the door §3 already reserves for the
world's own residents: `Query::Clients` deposited into the `gestures/` inbox and
its reply read back with the one reply codec. **No verb and no transport were
added** — the roster read is the one bl-4e08 landed. The child folds the same
state root the engine writes, because the world hands `XDG_STATE_HOME` down to
every process it spawns (DESIGN §16.2). Every wait carries a bound and ends
early on litany's stop flag, which is the router obligation litany states and
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

The first three keys — and an optional `"subject_cwd": true` (bl-77be) —
**are** §5.1's advertised element, verbatim; `command` and
the optional `cwd` are the local half. `subject_cwd` is the worktree lane's
per-tool consent (§5.4): this box will execute that entry at the working
directory an invocation carries, overriding the entry's own `cwd`. It is the
operator's statement about the machine, in the machine's own file, and
deleting the key deletes the capability — the severability the whole document
exists for. The client presents the advertisement by
reading this file and dropping the local half — one document, two readings — so
what a host offers and what it can actually run cannot drift, which is the whole
of why the config is not a second list beside the advertisement.

`command` is an **argv, spawned directly**. There is no shell and no
interpolation of the invocation's input into it: a shell would make the declared
schema advisory and turn an operator's config into a command-injection surface
for anything the model can type. The invocation reaches the command exactly as
litany's own tool contract already delivers one (its ARCH §3.3): the
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
the tool, and `capture`, which polls for what came back. Since bl-77be an
`invoke` — and the `invocations` row the far machine is handed — may carry one
optional field, `cwd`: the subject's location (§5's worktree lane), set only
by the lane and honoured only under §5.2's `subject_cwd` consent. That field
and §5.1's consent flag are the two shape moves PROTOCOL 2 versions; the
corpus ledger is what forced the bump. All four are ordinary
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

**Hand-off is not delivery, so an unanswered invocation is redelivered**
(bl-e658, twin of thrall bl-9261). The read parks for the mailbox's whole hold
and a thread asleep in that loop has not learned that its peer's socket is
gone, so marking a slot spent where it is handed to the answering code loses
every invocation posted into a dead parked read — silently, and for the hour
until the sweep, because *absent* is documented above as *still running*. The
mark is therefore a **lease**, and the thing that ends it is the client's own
next follow-class read: every slot that client was handed and this engine has
no capture for goes back on the queue at the moment it asks for work again.
The connection cannot be the lease's scope — a host dials per ask and drops the
connection the instant the answer is read, so a lease released on connection
end would re-queue an invocation at the moment it was correctly delivered.

**Which makes the leg at-least-once, and the invocation id is the idempotency
key.** The trade is deliberate and it is not symmetric: what the latch bought
was silence — an invocation nobody ran and nobody was told about — and what the
lease costs is a second run in the one case where a host executed something and
its `complete` never landed. A redelivery carries the **id it was first handed
under**, so a far end that must not run a thing twice has a stable name to
dedupe on and needs no field of its own to get it. Nothing on the wire moved:
`invocation` is the id the follow-class read has always carried.

**One reader per client identity is what makes the predicate exact.** "The
client asked for work again" only means "it did not finish what it holds" while
one connection at a time may ask. Two parked readers under one identity is two
processes claiming one machine's name, and §5.1 refuses the second in band
(bl-1462) — that guard is this predicate's precondition, not a decoration on
it.

**A capture is text, and the transcode happens once.** A capture ends as a
model's tool result and a model's message is text, so the *executor* transcodes
its child's bytes at the one place bytes stop being bytes, and nothing
downstream carries an encoding case. A tool whose output is not UTF-8 loses
exactly the bytes no string can name, which is the trade every other §11 file
read already makes.

**The executor is the thrall (§5.4), and it is not in this crate.** It reads the
§5.2 config, advertises the projection of it, and then loops: `invocations` →
run → `complete`. It runs **serially** — one invocation at a time, which is what
makes a busy host absent. *This paragraph named `yog tool-host`, a client mode
beside `yog seat`, until the severance (bl-7942) took every client out of yog:
the crate now holds a listener and no dialler at all, and the sentence stood
long enough to send a reader after `src/wire/host/` that had not existed for
days.*

***The no-reconnect ruling is reversed at the channel and kept at the process***
*(bl-0a74).* It read: *"it **does not reconnect** — a channel that fails is an
exit naming the failure, because restart policy belongs to the supervision the
operator's machine already has, and inventing one here would be yog deciding how
a box it does not administer runs a program."* The premise is sound and the
conclusion does not follow, because **supervision restarts a process and the
roaming case does not kill one**. §1's canonical box sleeps, changes networks
and crosses a relay switch: TCP drops, the channel's thread ends with its
sentence, and the foot process stays healthy — serving its other engines, if it
has any. A supervisor sees an exit code that never comes. From that moment the
engine believes this box is gone (presence is connection RAM, correctly) and the
box believes it is serving, and nothing on either side says otherwise until
somebody restarts a process that never failed.

So a foot **redials its own channels**, with a backoff that settles rather than
spins — a box in airplane mode must reach a slow cadence, not burn a core — and
still **exits when it cannot be a foot at all**, which is the part supervision
was always the right owner of. It is not a session resume and there is nothing
to resume: presence re-forms as it does for any fresh connection, the
advertisement rides the connection already, registration is durable engine-side,
and an invocation in flight when the wire died is the mailbox's lease (above),
not the redial's. **A redial is also a coarser instance of the §5.1
re-assertion** — it presents the set again on the new connection — which is why
the two are one subject and not two.

**The one refusal a redial must expect is its own predecessor.** A read parked
when its connection died does not leave until this engine tries to answer it, so
a redial inside that window meets §5.1's one-reader refusal naming this very
client. It is **retryable** and it is **bounded**: the claim's life is the
mailbox's hold and not the connection's — `Mailbox::take` releases it on the way
out, before its answer is written — so the stale predecessor is gone within one
hold's width (thirty seconds) whether or not the socket has been noticed. A
redial loop that treated that sentence as final would turn the first blip into
the permanent stop this reversal exists to end.

**A tool runs under two deadlines, and they measure different things.** The
host's own bound terminates the child (SIGTERM then SIGKILL, the cascade the
crate already owns) and answers the shell's `timeout` verdict with a sentence;
the driver's longer patience stands behind it for the case where the whole host
process went away. Neither is a knob: an engine that has not answered is down,
and a tool that has not answered is working.

### 5.4 The thrall, and the local execution corpus (bl-1dd3)

**A thrall is §5.3's executor, severed into a separately installed component**
(§12) and carrying a foot-grade leaf (§4.2). Its entire wire surface is the
three gestures a foot may send: advertise once, wait on `invocations`, post
each capture with `complete`. **The transport is unchanged in every
particular** — it dials per ask and holds a connection only while it is
*waiting*, so a busy thrall is absent (§5's presence amendment), and **nothing
in the path is the engine speaking first** (§3's routing ruling). Founding the
repo is bl-1dd3's other half and lands there; this section is what it
implements against. **It has shipped, and it is the only executor** — the
interim reading, *"until it ships `yog tool-host` is the same executor reached
by a verb"*, expired with the severance (bl-7942, §5.3).

**The thrall owns the local execution corpus, and yog owns none of it.** This
is the bl-37fd ruling's sharpest consequence and it inverts what the tree does
today:

- **Workspace substrate access is a thrall tool.** Balls operations, repo
  tools, the shell of a checkout — anything that reads or writes a working
  tree — reaches an agent as a tool the thrall on that box advertises, and
  never as an integration inside the engine.
- **yog exposes engine acts only** — agent lifecycle: start a conversation,
  deposit, interrupt, stop, compact, fork. Those are acts on the *world*, which
  is the thing yog holds. Everything that happens on a *machine* is a thrall's.
  **This is the destination, not a description of the tree**: today's boundary
  carries the balls family, the config writes, the files and git reads and the
  rest, because today one process is server, seat and foot at once. Each moves
  under §12's migration order, and the sentence above is what each move is
  measured against — nothing NEW goes into the engine that a thrall could
  advertise.
- **A thrall beside the server is the normal install, not a special case.** The
  single-box operator runs both and enrolls the local one, and that enrollment
  is an explicit act (§12's "ship inert"). There is no in-process shortcut for
  the co-located case and no unix socket beside the wire: one transport, one
  code path, no place for a bug to hide (§12's "front door only"). The
  co-located thrall pays a loopback handshake per invocation, which is
  microseconds and buys the property that the local case and the remote case
  are the same case.

**The compactor's procedure pair is an engine act, and yog answers it**
(bl-dfce, operator ruling 2026-08-29). litany injects tool definitions from two
sources, not one: the host's injection (§5.2) and the calling role's own
**procedure**, which today is exactly `write_summary` / `mark_for_deletion`.
Since the seam inverted (§5's pipeline ruling) the engine's router is total, so
that pair arrives at yog's router like every other name and yog has to say
where it belongs. **It belongs here, and the subject-locality invariant decides
it on its own**: *"a tool executes where its subject lives"* — `write_summary`
writes the conversation's own summary onto the compactor branch and
`mark_for_deletion` nominates that same conversation's files, so the pair's
subject is the conversation, the conversation lives on the server, and no
machine and no thrall is involved. §12's *front door only* is not narrowed by
this; it governs execution **on a machine**, which this is not. Shipping the
pair to a thrall would send a box that does not hold the world a request about
it — the adjacent move §5 already forbids for the world fold. The principle the
ruling states, and which this is the first instance of: **context management
happens in yog.**

**Performed at the engine's own front door, never reimplemented.** litany
defines what the pair *does* — the summary numbering, the refusal to nominate
the dispatch entry, the staged removal — and yog restates none of it. The
router re-enters `<driver_target> tool <name>`, the third hop litany's own
resolution addressed before the inversion, with the caller identity on the
child's environment and the `tool_use` input on its stdin; the child is yog
under the `litany` namespace, standing in the same world. One definition of the
acts and it is upstream's. **The name set is closed and enumerated in exactly
one place in code** (`src/tool_host/engine_act.rs`): two rows, because the
procedure is the only second source of injected definitions litany has. A third
would arrive there as a row, never as a prefix test or a name shape.

**The tool control still sees the pair, and no carve-out was needed.** litany
adjudicates in the tool window, *before* the executor is entered, for every
name — injected ones included. So `yog tool-control` judges these two exactly
as it judges every other invocation, and yog's router, which lives inside
execution, could not exempt them if it wanted to. Nothing about the chokepoint
moves.

**A name nobody offers is now a refusal yog renders.** With the router total
there is nothing to hand an unowned name back to, so the injection answers it
itself: non-zero, the reason on stderr, indistinguishable from the "no such
tool" an absent binary produced behind the front door. That is §12's ship-inert
posture reaching its natural edge — a server with zero enrolled thralls refuses
every ordinary call in band **and still compacts**, because compaction was
never a machine's work. Since bl-5710 it also still **reads and writes its own
worktrees**: the three names the engine implements take the lane's last rung
below rather than a refusal.

**The conversation-subject worker grants are engine acts too** (bl-77be,
which widened bl-dfce's set from two to six). The engine's shipped worker
grant carries eight names, and under the total router seven refused —
subagents, inter-agent messaging, the skills corpus and every act on the
conversation's own worktree all dead in band. The remedy is not one mechanism
but the subject-locality audit, name by name:

- **`dispatch`** mints and launches a child conversation — branch, goal
  deposit, driver launch, all on the workspace the server holds. Engine act.
- **`message`** deposits into another conversation's inbox. Engine act.
- **`load_skill`** copies a server-disk skill (`<data-root>/skills/<name>`)
  into the agent's server-disk worktree. Engine act.
- **`cd`** writes the agent's working-directory mark, a ref on the workspace,
  and validates the path against the filesystem that holds the worktree.
  Engine act — and its validation being server-side is consistent with the
  lane below, whose consenting box holds that same filesystem.
- **`bash`, `read_file`, `apply_patch`** are execution and filesystem acts at
  the conversation's working directory: they take the worktree lane below,
  reaching a consenting machine where the operator enrolled one and the
  engine's own front door where none consents (the lane's last rung, bl-5710).
  Never an engine act, because an engine act never consults the roster.
- **`multi_tool`** never reaches the router at all (the engine's own step
  loop fans it out); each inner name is judged on its own subject.

The engine-act name set stays closed and enumerated in exactly one place
(`src/tool_host/engine_act.rs`), now six rows, and the mechanism is bl-dfce's
unchanged: re-entry at the engine's own front door with the caller identity on
the child's environment — and, since the seam hands it over (litany bl-ddaa),
at the caller's **resolved working directory**, which is the engine's own
contract for in-process built-ins (a relative `cd` resolves against where the
agent stands). A seventh row is a deliberate act with this audit's question
asked again.

**The worktree lane** (bl-77be). A granted, unqualified name that is neither
the `clients` tool, an engine act, nor a loaded host-qualified instance is a
**workspace-subject attempt**: its subject is the conversation's working tree,
the worktree lives on the server's box, so the subject already chose the
executing box — which is why a bare name is not a call with an implicit
location, and why §5's locality-rides-in-the-name rule for *loaded* instances
is not contradicted. The router resolves it against the workspace roster: the
ONE registered client that both advertises the name and consents to
workspace-cwd execution (`subject_cwd`, §5.1/§5.2) executes it, and the
invocation carries the conversation's resolved working directory — the
mark-or-worktree resolution the engine already performs, crossing the seam as
`RoutedCall::cwd` (litany bl-ddaa). Zero consenting advertisers is an in-band
refusal naming **the one way out that is one** — the operator marks an entry
`subject_cwd` on the box that holds this server's worktrees — and saying
outright that loading a host-bound instance is *not* that (bl-68e1, below).
More than one is a config ambiguity,
refused naming every claimant: one adjudication decision must stand for
exactly one execution on one machine (§5, no broadcast). The consenting box is
normally the co-located thrall — the normal install above — because it
actually holds the worktrees; a box that consents without holding them earns
honest in-band failures from its own end (thrall refuses a directory it does
not hold, naming it). Front-door-only is untouched: a routed call is the same
adjudicate → mailbox → execute → capture pipeline every call takes, and the
server executes nothing in its own process. **Zero consenting advertisers is
no longer the end of the lane** — see the last rung below (bl-5710) — but for
every name the engine does not implement it is still the refusal, unchanged.

One shape this deliberately is not. **Not a narrowed grant**: cutting the
shipped grant to `[multi_tool]` would stop the decoys and give nothing back —
the four capabilities the grant names are real and the audit above houses
each.

**The lane's last rung: the engine performs its own built-ins** (bl-5710,
operator ruling 2026-08-31: *ship some basic tools — a default install must be
able to write a file*). bl-77be closed with a second rejection beside the one
above — *not engine-side execution*, on the argument that answering `bash` at
the engine's front door would be the server executing machine work in its own
process. **That rejection is reversed here**, and what it cost is the reason:
the shipped worker grant offers `bash`, `read_file` and `apply_patch` to every
model, a bare engine is the default install, and with no foot enrolled every
one of those calls refused. A conversation on such an install opened with
`apply_patch`, spent a step per refusal, and ended having written nothing —
five refused calls and seven round trips, measured. A server that cannot act
on its own worktrees is not inert; it is dead, and ship-inert was never meant
to reach that far.

So the lane is a **ladder**, and its rungs are ordered by how explicit the
operator's intent is:

1. **Exactly one consenting machine** — an enrollment plus a `subject_cwd`
   key, both operator acts. It wins, unchanged: bl-77be's landing stands, and
   an operator who wants the work on a particular box still gets it there.
2. **More than one** — the config ambiguity above, refused naming every
   claimant. An ambiguity the operator authored is a defect to tell them
   about, never a reason to quietly execute somewhere third.
3. **No consenting machine, and the engine implements the name** — performed
   at the engine's own front door, `<driver_target> tool <name>`, the
   compactor pair's mechanism unchanged: the caller identity on the child's
   environment, the `tool_use` input on its stdin, and the conversation's
   resolved working directory as the child's cwd.
4. **Anything else** — the refusals above. A pool name an operator granted
   has no engine implementation behind it, so it keeps the sentence that
   names the enrollment.

**A refusal names ways out, and a way out that lands the work somewhere else
is not one** (bl-68e1). Rung 4's two sentences used to offer the loaded lane
*first*, as the remedy the model could take unaided — *"use the clients tool
to see this workspace's machines and load what one advertises"*, *"load it
with the clients tool to run it in that machine's own directory"*. That offer
is wrong on the lane's own premise: a workspace-subject name's subject is the
conversation's working tree, so the subject already chose the box, and running
the same argv on a different one is not the act the model was refused. It is
worse than wrong in practice, because **a loaded invocation carries no
directory at all** — §5's locality-rides-in-the-name definition — so the far
foot runs it in whatever directory its own process inherited. That is exactly
the directory thrall's DESIGN §3.4 refuses to resolve a relative `cwd`
against: *"a place nobody wrote down, which changes when the unit file does,
and which nothing in the running system reports"* — the refused case reached
through a different door.

A drive took the offer, and nothing at the boundary could tell. Every write,
every test run and every `ls` the model made to check itself happened in the
foot's inherited directory, so the check could not fail; the conversation
reported success with output quoted, and the bound directory was empty.
`/files` and `/work-diff` read the conversation's own tree, so a run that
built in the right place and one that built in the foot's scratch answer
alike. **So the refusal names only the operator's config edit** — the one act
that puts the work where its subject is — and names the load as what it is
not, in one clause carrying where a loaded instance would run and that nothing
this conversation can read would show what it wrote there
(`src/tool_host/subject/refusal.rs`, `NOT_A_REMEDY`, the one home of that
sentence). The lane still *has* a loaded rung and the `clients` tool still
loads; what ended is a refusal recommending it for a subject it cannot serve.

**What that rung does not concede.** §12's front-door invariant says the
server executes nothing **in its own process**, and it still does not: the act
crosses the engine's own front door as a child process, which is the same door
the compactor pair takes and the same one litany's own resolution addressed
before the seam inverted. yog restates none of what the three names *do* —
`litany tool <name>` is the one definition, upstream's. And subject locality
is not bent either: the conversation's working tree lives on the server's box,
so the server's box is the box subject locality already named. A co-located
thrall is the *explicit* way to reach that box and still wins; the front door
is the default, because a default install has no foot.

**The rung's set is closed, and since bl-e654 it is *derived* in exactly one
place in code rather than listed** (`src/tool_host/subject.rs`, `performs`):
`litany::cmd::BUILTIN_TOOLS` — the engine's own constant, exported since litany
0.0.5, upstream bl-4cbb — **minus** `engine_act::NAMES`. The two admitting facts
are unchanged and are now *stated by the partition itself*: a row's subject is
the conversation's working tree (it is not an engine act), and the engine
already ships an implementation of it (it is a builtin). What the partition
leaves today is `apply_patch`, `bash` and `read_file` — three rows — and those
literals live in the test that audits the partition, not in yog's prose or its
source. yog restates no name it does not own, so an upstream builtin added or
renamed moves the rung by moving the engine's constant, and reddens that one
test rather than silently disagreeing with a stale copy. A fourth row is still a
deliberate act with both questions asked again, upstream, and never a prefix test
or a name shape.

**Local config gates what a thrall enables**, and it is §5.2's document
unchanged: `<yog-data-root>/tools.json` on the thrall's own box, operator
authored, with the advertisement derived from it by dropping the local half. A
tool absent from that file does not exist as far as the wire is concerned —
which is the severability the whole arrangement rests on: what a box will run
is decided on that box, by the person who administers it, in a file, and
removing a capability is deleting a config entry rather than editing anything.

**Server-side adjudication is unchanged and still fails closed.** The
invocation crosses `yog tool-control` exactly as it does today, before it
reaches a mailbox. Foot grade narrows what a thrall may *say*; it widens
nothing about what it may be *asked*, and the two questions do not meet.

**The containment honesty carries over verbatim** (§5): *"execution happens on
a machine the adjudicator cannot inspect. Adjudication judges the invocation
exactly as today; any containment beyond that is whatever the client enforces
locally, and the design must not claim otherwise."* Severing the executor into
its own installable does not change this by one word — if anything it makes the
sentence more literal, since the box now runs a different program, by a
different name, that the server never shipped.

**MCP enters only as a thrall-local bridge, and is deferred.** The shape, when
it is paid for: a thrall is an MCP *client* on its own box, and re-advertises
the tools its local MCP servers offer up the wire as ordinary §5.1 elements. It
buys the existing ecosystem without any of it reaching the engine. **yog never
learns MCP** — not a verb, not a schema, not a transport — because a protocol
in the engine is a second vocabulary on a surface §3 keeps down to one, and the
bridge belongs where the servers are. Nothing about v1 is blocked on this and
nothing in v1 anticipates it.

### 5.5 The follow lane's frame is an append (bl-3655)

**A `Query::Follow` frame carries what landed since that read's previous frame,
not the whole answer.** This is the ruling a seat implements against, and it is
one rule with no flag and no case:

> Absorb every frame of a read, in order, onto an empty fold. What you hold
> after the last frame you have received is what you paint.

Three consequences fall out of it and none needs a field:

- **A read starts holding nothing**, and the engine's reader is minted per held
  connection and opens the response file at byte zero — so the **first** frame
  of any read is the whole tail so far. A seat that dropped a connection
  mid-answer re-asks and is whole on its first frame, with nothing to
  reconcile. That is the property the old whole-text frame existed for, and it
  is kept.
- **The one-shot answer is unchanged.** An intake that cannot hold a connection
  (§3: the streaming form is not a second form) is answered `Reply::Follow` with
  the accumulated tail — which is a stream of one frame absorbed onto nothing,
  i.e. exactly the rule above. Two reads by the same seat are two reads: the
  second starts holding nothing, so it replaces rather than appending.
- **The fold is the crate's own and the seat may copy it.** `Stream::absorb`
  accumulates text in stream order and lets the newer delta kind win, and its
  contract is `fold(a).absorb(fold(b)) == fold(a ++ b)` on any line boundary.
  The engine gathers its frames with that same operation, so an engine frame and
  a seat's accumulation agree by contract rather than by coincidence.

**The wire spelling did not move, and that is the hazard worth stating.** The
frame body is still `{"delta": "text"|"thinking", "text": "…", "thinking": "…"}`
— `delta` names the kind of the last content event, as it always did, and the
two text fields are now the *appended* part rather than the accumulated one.
The corpus ledger records field paths and types, so it cannot see a change of
meaning under an unchanged signature: nothing forced a version bump and none was
taken. PROTOCOL 2 is unreleased, so the move is free; a seat consuming this lane
(the mobile seat's protocol-2 follow-up) must consume this section with it.

**Why the trade flipped.** The whole-text frame was idempotent per frame and
order-independent, which is a real property and was the right first answer on
loopback. Its cost is **quadratic in the answer's length**: one measured reply
was 32 frames, 416 bytes of answer, 8,310 bytes on the wire — 20x, and that is
the *floor*, measured on two sentences. The same frame density over an
eight-hundred-word answer is half a megabyte for five kilobytes of text. This
lane exists for a phone on a mobile link watching a long answer write itself,
and §6 already says a client holds "key material and RAM" — RAM is precisely
what makes the accumulation cheap to keep locally, and a seat is already holding
the accumulated text in order to paint it.

**Frame count is not addressed and is deliberately left alone.** Coalescing —
a minimum interval or a minimum new-byte count — was the cheaper half-fix
before the frame carried an append; with the append it buys only per-frame
envelope overhead, which is linear rather than quadratic and is the tick's
question (§7.2's write cadence), not the frame's. Adding a second mechanism to
save bytes the first one already stopped multiplying is mechanism for its own
sake.

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

**Bare `yog` is the server, and there is no other face in this crate**
*(bl-7942)*. The binary's whole interface is: the engine (a bare `yog`, which
boots `Engine::boot` and parks until a §8.5 stop); `yog gesture`, the
deposit-and-wait sugar over the world's own inbox; the three namespace arms
(`bl`, `litany`, `bz`) and balls' two plugin seams; the two world hatches (`yog
env`, `yog exec`); `yog tool-control`, the capability adjudicator litany's seam
spawns; and `yog wire-certs`, the operator's mint (below).

**There is no `serve` verb.** It named the windowless face while a windowed one
stood beside it; with one face the word selects nothing, and two spellings of
one face are two facts — which is the same reasoning that renamed `headless` to
`serve` in the first place (below). An operator's systemd unit runs the bare
binary — since the bl-b973 cutover, inside the OCI image
(`scripts/deploy/yog.service` is a `docker run` unit).

> **Until bl-0716 (landed):** one crate, one multi-call binary. `yog serve`
> runs the engine (the former `headless` boot plus the wire listener); bare
> `yog` runs the window; `yog seat` and `yog tool-host` are the two **client**
> modes, the second landed with §5.3. A TUI or web seat later is another
> consumer of the same wire and needs nothing new from the engine.

**Since bl-0716: the seat lives in its own crate and repository.** `lernie`
0.1.0 is the seat — the window and its wire client — extracted at the version
fence §12 fixes, with its own gate, its own store and its own DESIGN. It links
no substrate crate and nothing server-side; that dependency dividend is what
the severance was for.

**And the window's deprecated release was never cut** *(bl-7942)*. The
migration order in §12 gave yog's window one release of overlap so an operator
on the published binary lost nothing while the seat crate was stood up. The
operator pulled the cutover trigger with every component's publish batched into
ONE coordinated moment, so that release would have been a deprecation nobody
consumed — and the only deployments are the operator's own dev box and one
server, neither of which needs the overlap. The window was therefore **dropped
outright** rather than deprecated, and `yog seat` and `yog tool-host` went with
it: both were wire *clients*, and a client is a seat's, not a server's.

**Read a `lernie` in this document against the fence** (§12): a bare one names
the seat crate or the ruling, and one bound to a `0.0.x` version names the
agent-loop engine at that release — the program that continues as `litany`.

**What the extraction cost, and what it did not.** It moved code, not
architecture: §1.2's ruling had already made the window a pure wire client of
localhost, with `AppModel::dispatch` deleted and every read and act through the
front door, so there was no in-process face to unpick. What the seat
**reimplements** is this document — the framing, the preface, the mTLS dial,
the §8.2 entries and the workspace-name mapping — exactly as the android client
does. **No shared protocol crate was created and none should be**: §12 makes
this document the versioned authority all four components implement against,
and a shared crate would make it the authority for three of them and a
dependency for the fourth.

The seat's **first** landing is the wire client and the one-shot verb over it;
the window itself is filed in that repository rather than moved, because a
window paints *typed* replies and the reply vocabulary is the part of the
boundary the seat had not yet had to own. Its DESIGN §6 is the ledger.

**And every `yog seat` / `yog tool-host` below names the PROTOCOL, not a yog
verb** *(bl-7942)*. §12 makes this document the versioned authority all four
components implement against, so the two client modes are described here in
full and are implemented by the `lernie` seat and by `thrall`. Read them as
*"what a seat does"* and *"what a foot does"*; nothing in this crate answers to
either word.

The same reading applies to every `src/shell/**`, `src/wire/{asker, poster,
lane, link, seat, host, channels, dial, entries, client, post}` and frame-side
`AppModel::` path this document names: §9's build-sequence ledgers record what
each ball landed **at its time**, and the client halves they name are the seat
crate's code now. The *protocol* each describes is unchanged and is what this
document is the authority for; the file paths are history.

**Two roles in one process, one boundary between them** *(bl-ae05; the two roles
became two processes at bl-7942)*. Bare `yog` boots the engine in its own
process exactly as it always has — the listener rides `Engine::boot`, one world
one engine, and that ruling is untouched. What changed is that the second role
left: a local window used to share this process and be handed its address in
RAM, because only the listener knows what a `:0` became. A seat is a separate
program now and reads the address the way every other seat does — the operator
states it, or a local one is told the bound port by the operator. The two
alternatives §8 already rejected stay rejected and neither is needed: nothing
spawns a second engine, and nothing refuses a desktop launch with a terminal
instruction, because a desktop launch is the seat's own program.

**`headless` is now `serve`, and the rename is the point** *(bl-b6fa)*. The
face did not change — it is still the one `Engine::boot` with no window — but
the engine now carries the listener, so what it is to anything off the box is a
server. Two spellings of one face is the drift the const exists to prevent, so
there is one word and §8's is it.

**The listener rides the ENGINE, not one face** *(decided, bl-b6fa)*. §8.5
already runs the deposit consumer on both faces, for I0's reason: *a deposit
converges whichever face is up*. A seat wants the identical guarantee, so the
listener boots beside the consumer in `Engine::boot`, which is the whole binary
since bl-7942. That decides the local boot question §9.5
was asked to settle, and it decides it by **dissolving it**: nothing spawns
anything, nothing refuses with a remedy, and there is no ladder — because there
is never a second engine to arrange. One world, one engine, whichever face
started it — a ruling about the box's OWN world, which bl-aaec leaves
untouched: bare `yog` still boots the engine, a box used purely as a seat on
other boxes is just a window whose local workspace set is empty, and no
window-only mode exists. Every other seat of *this* world is a client of this
one engine; the window is additionally a client of every entry it holds
(§8.2). The alternatives were both
worse and both were considered: one face *spawning* another gives one
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
`WIRE_DIR`/`WIRE_HOST`/`WIRE_PORT`/`FORCE` interface is unchanged. **`WIRE_LEAF`
is the fifth reading** *(bl-64a7)* and it selects the recipe's *other* act
rather than modifying the mint: issue ONE extra client leaf under the common
name it states, over the CA already here — no CA founded, no address written,
no other leaf touched. That is §8.2's host half, one more artifact the one
recipe can be asked for. It refuses three ways, each naming its remedy: a
common name the registry would refuse (§4.1's rule, spent once — a plain path
component, never the reserved `local`), a directory with no `ca.key` (a client
box cannot issue, and the mint never replaces an operator's trust root), and a
pair already under that name (re-issuing distrusts nothing, so it would be two
live certificates under one identity; a fresh name is the remedy and rotation
stays `FORCE=1` over the whole directory). The server
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

**A wire the engine cannot get up is a refusal SAID, not swallowed**
*(bl-dc14; narrowed by bl-7942)*. The whole failure family — a bind an
operator-stated port lost, a mint the box cannot perform, a half-provisioned
directory — used to collapse to one stderr line and an engine with no listener,
and `main.rs` kept a window anyway: a frame with no asker, no poster and no
searcher, accepting text and firing nothing, with the one diagnostic on a
stream a desktop launch has nowhere to show. bl-dc14's ruling was that a
refusal has to be *paintable*, and it made `wire::listen` RETURN the sentence
instead of swallowing it, said once per face.

There is one face, and it has a terminal: the engine says the sentence on
stderr, its unit's journal keeps it, and the engine **runs on without a wire**
— every deposit still converges through the inbox, so only a seat is shut out.
A seat that cannot connect learns it from its own dial and paints its own
sentence, which is the seat crate's half of this ruling and is why the
paintable half left with the window. What is not negotiable in either half is
the first line of the original ruling: the engine states the CAUSE (a bind, a
mint, a missing file), never the consequence derived from it.

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
  invent, and **a foreign workspace needs no special case**: litany's
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
  conversation's identity is a **pair** — litany's id (its branch name, and the
  only half that addresses a path) beside the §3.3 name it wears — so it is
  addressed by **an agent id, or the unique stored name a living agent wears**.
  That is not a vocabulary yog invented: it is litany's own
  (`workspace::agent_name::resolve`, "an exact id match first, else the unique
  living agent wearing that name"), and the two spaces are disjoint by
  construction, every id opening with the compact `YYYYMMDDTHHMMSSZ` stamp and a
  name that reads like one being refused at creation — so the resolution never
  guesses which reading was meant. It had to become a contract because a
  `/prompt` receipt answers with the **minted name** (the root has no id until
  its detached driver writes `agents/<id>`) while the terminal's usage said
  `--agent ID`: the handle composed with `message`, litany's one name-resolving
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
- **Both nouns, and the refusal names what it could not match** *(bl-3377)*. The
  bullet above folded workspaces and left the **project** noun on the cached
  copy, so a `yog bl prime` inside the world was refused by every ball gesture
  for a whole sweep — `unknown project "proj"`, byte-identical to a typo, and
  the same line one sweep later succeeded. A project enumerates the same way
  (§5.1 #1, one readdir of the balls clones dir) and now folds beside the
  workspace set, inheriting every consequence above. And because `--project`
  takes a *derived* name that **no gesture answered** — `/balls` and `/board`
  carry a project only for balls that exist, so a primed project with no balls
  is a word the operator cannot learn — the refusal carries the set it could not
  match, bounded and then counted. That is a listing verb dissolved rather than
  added: the refusal is where the question is already being asked, and `naming`
  is already the one home of both names. Scope runs first, so a scoped client is
  told what *it* may address and never that something else exists.
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
  litany's `--cwd`: minted by the engine at `Prepare`, relayed back verbatim
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
  - ~~**Three remain**~~ — **two remain, and `search::Address` is closed**
    (bl-764a). Its three fields were not mere disclosure: the ball arrived as
    *"no conversation search: a client can only scroll"*, and the survey found
    the query was never missing — `Query::Search` already spans conversation
    names, goals and transcript text and answers one hit per selectable
    address — but its answer was unusable off-box, because a conversation
    hit's `workspace` key carried the engine's absolute path where every
    gesture and every read (`/transcript`, `/agent`) takes the §3.1 name.
    Exactly the QueueRow defect above, on the read whose whole product is *an
    address you ask next*. So the ruling is the JoinRow one, not a new query
    family and not a parameter: **a hit is an address, and every address
    field crosses as the wire name** — the §3.1 workspace leaf on
    `Workspace`/`Conversation`, the §5.1 #1 project name on `Ball`, resolved
    back through `Snapshot::ws_path` / `project_path` at the one seam that
    owns the round trip. `Found::unreadable`'s prose names its sources the
    same way (`<workspace>/<agent>: …`), because the gap rides the same reply.
    No protocol bump, on the bl-22ab reading: the field always *meant* the
    address, and the path was the regression. A seat wanting a
    title-only filter over the roster it already holds still filters
    client-side — the answer to "search conversations" a seat can give alone —
    and asks the engine only when content must be read, which is what the
    engine-side corpus (goal + transcript bytes, re-read at ask time) exists
    for.
  - **Two remain, both disclosure rather than broken addressing**, and each
    needs a ruling of the kind this bullet's first half is, not a rename:
    `fleet::Facts`'s `workspace` and `project` (`Reply::Board`), and
    `OpRow::cwd` (`Reply::Ops`), whose *subject* is where a command ran — the
    one case §8 already says keeps path semantics, since answering it by name
    answers a different question.

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

**As landed (bl-4e31).** `yog seat` resolves the typed gesture's workspace
name over the entries first (`wire::seat::channel`), dials that entry's
channel on that entry's material, and re-encodes the gesture carrying the
host's name **only** where the entry renames it — the one write site the
mapping is spent at, with the operator's own envelope crossing byte for byte
everywhere else. An entry that exists is the answer to its name even when it
cannot be dialled: a half-provisioned entry refuses with its own sentence
rather than falling through to the flat root, which would send a gesture to
the wrong engine on the strength of a missing file.

`yog tool-host` has **no name to resolve**, and that is not a gap: none of its
three gestures addresses a workspace — an advertisement and the routing leg's
two address a *machine* (§5) — so what an entry adds it is a second engine to
be present at, never a name to rewrite. It therefore serves the flat channel
*and* one channel per entry, each on that entry's own material and so under
that entry's own client identity (§2's one-certificate-one-client, which is
what makes the advertisements separate without a mechanism). One thread per
channel and execution serial *within* each, so §10's deferred-concurrency row
is per-host and untouched. A channel it cannot open is said once while its
neighbours are served; only a box holding **no** channel at all refuses
outright — and a box whose flat root is merely self-provisioned (`:0`, §8)
loses that one channel and keeps its entries, which is what a pure-client box
holding a local engine looks like. With zero entries the whole of that is one
channel and one sentence: exactly what a tool host was.

**As landed, the window's half (bl-028a).** The model holds one standing-question
set per channel (`wire::channel`) and the set of them (`wire::channels`), which
the engine composes at boot because the engine is what holds the world. The
roster the frame paints is the union across those slices, local first and then
the entries in leaf order, every row carrying the channel it came from — a
client-side stamp, so no origin crosses the wire and no reply type grew a field
the engine cannot fill. Names resolve exactly as `wire::seat::channel` resolves
them: an entry is the answer to its leaf, and every other name — and every
question naming no workspace — goes to this window's own engine. **A collision
is read off the union and refuses in place of the answer**, which is the sentence
every read surface already paints, and it names the token and the remedy:
*"ambiguous workspace "home": this window's own engine and the entry "home" hold
that name and the union is one namespace — rename the entry (`mv` its directory
under `workspaces/`), never the workspace on its host."* An entry's leaf is
unique on disk, so the only token two channels can both hold is one an entry
claims and another channel already names; that question costs a look at the
composed roster and is asked only when an entry claims the name, which is why
**zero entries is byte for byte today** — nothing claims, nothing is resolved,
nothing extra is asked.

**An entry wears a row before it answers.** The entry IS a workspace this box
participates in, so it appears in the roster from the moment the operator
provisions it, carrying the zeros it honestly has — the §3.4 raise claim's shape
one noun over. What fills those zeros is the entry's own asker, below.

**As landed, the four threads (bl-670c).** The window attaches to every channel
it holds, and the four halves of that attachment differ by what each does with
the set rather than by any second mechanism.

- **One asker per channel** — its own thread, its own seat on that entry's own
  material, its own slice. That is where the isolation is: a seat's dial is
  bounded by the kernel's connect, so a host off the network parks one thread
  while every other channel keeps being answered on its own. This section's
  *"the window's cost is linear — one channel, one asker pass per cadence
  period, per entry"* is that cost, paid deliberately. **Only the loopback
  channel seats the window** (§4.1): a registration is a file on the host's
  disk, written by the operator who owns that box, and nothing on this side of
  the wire may write one.
- **The poster routes**, sending each act down the channel the workspace it
  names resolves to — which is what makes §7's amended fact-locality true for
  ACTS: a foreign workspace's seen and pin gestures land at its host. Exactly
  once is untouched and structural, routing picking a channel rather than
  duplicating one. It stays a single thread: acts were serialized among
  themselves from the day that thread existed, because *"an act runs as long as
  the verb behind it"*, so a dead entry's act is a slow act rather than a new
  class — and the reads that must never wait behind one have a thread each.
- **The lane resolves**, because one conversation is focused: it is dialled at
  whichever channel hosts that conversation's workspace, and a subject crossing
  the boundary is the re-ask the lane already performs whenever a subject moves.
- **The searcher fans out** and publishes the union *as it arrives*, local
  channel first and then the entries in leaf order. Each host's block stays
  ranked and bounded by that host — no global ordering, dedupe or clock is
  promised across engines — and the union is not re-cut to `MAX`, which would
  silently delete a whole host's answer to keep a bound already kept once per
  engine. An entry's refusal is named with the entry that gave it; the local
  channel's is not, an unattributed sentence having always meant this window's
  own engine.

**The union's collision is a fact about the ROSTER, and only the roster asks
it.** A read of a colliding name refuses with the sentence above. An act, a
follow and a search hold no roster and need none: *"an entry that exists is the
answer to its name even when it cannot be dialled"* answers them outright, which
is the rule `yog seat` has resolved by since bl-4e31. Two readings, one remedy —
rename the entry.

**A remote name still has no local PATH, on purpose.** `Snapshot::ws_path`
resolves the *painted enumeration*, whose members are directories on this box;
a workspace hosted elsewhere has none, so `focused_workspace` answers `None` for
one. Every surface whose content is a name-addressed wire read works over the
union unchanged — the roster, conversations, the transcript, marks — and every
surface that still wants a path is one whose subject is a file on **this** box:
config editing, the ball store, a spawn's working directory. Withholding those
is correct rather than pending. The one residual is the frame's implicit ack
(`ui.json`, keyed by path), which records nothing for a remote conversation
while the explicit `MarkSeen` act routes to its host like any other.

**Withholding it is the whole rule, and flattening it was destructive**
*(bl-e349)*. `None` here means *this box has no directory for that workspace*,
and it is one answer away from `None` meaning *nothing is focused at all* — so a
surface that reads the focused **path** where it wants the focused **name** reads
the two as one state. The §8.1 start did: it took `focused_workspace()`, fell
through to §3.1's bootstrap default, and fired the operator's goal at a LOCAL
workspace called `home` — founding it, focusing it, and running an agent in it,
with the tab flip as the only thing the operator saw. One rung down, the pane's
own fire flattened the same `Err` with `unwrap_or_default()` and posted the act
at `PathBuf::new()`. **No arm may resolve a start to a path it invented.** The
three answers are one rule (`AppModel::start_path`): an enumerated name is its
own path, a name no channel holds is the place a `Prepare` would found it (the
§3.1 names root — the chokepoint's `resolve_workspace` read from the frame's
side), and a name an entry holds has **none**. The frame-side folds a start
holds — the §3.4 raise claim, the start claim, the pending echo — are every one
of them keyed by path, and every one of them is local optimism standing in for a
read that arrives on the entry's own slice: they skip, and the act itself is
untouched, because an act is addressed by the NAME the poster routes it by.

That is also what makes the §8.1 pair routed end to end. The `Prepare` names the
workspace and goes down its entry's channel, where §4.1's raise founds it **on
the host** and auto-registers its creator (§1.4's material clause, unchanged);
the `Prepared` that comes back is renamed to the leaf at the channel boundary
like a roster row, because the name it carries is handed straight out again as
the next act's address — an unrenamed one would route its own `Prompt` back to
this window's own engine, which is the local misfire the mapping exists to
prevent.

**How material reaches an entry** — §1.4 verbatim, forever. On the host, the
operator mints a leaf for the visiting box (`yog wire-certs` issues an extra
client leaf under a stated common name — the one recipe, one more artifact it
can be asked for) and writes the registration (§4.1's `mkdir` and `touch`).
The anchors, leaf and key are carried to the client box by hand — the same
out-of-channel act the certificates always rode — and written into
`wire/workspaces/<workspace>/` beside an `address` the operator states. **That
directory's name is the workspace's, never the common name the leaf was issued
under** (bl-686c): a client routes a gesture by the workspace it names, so
material filed under the leaf's name is a channel no gesture can address —
present, valid and unreachable. The two names are free to differ precisely
because the identity is the name *inside* the certificate (§2), which is why
the mint's instruction spells the destination from
`material::ENTRY` and not by hand. No
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

### 8.3 Sign-in follows the wall (bl-61bf)

**The gap, found live.** A seat registered into a workspace held elsewhere
(§8.2) could fire chats there and could not sign it in: the Login flow (DESIGN
§8.3) spawned `bz --login` on the seat's own box with a locally derived wall,
so the credential landed in the seat's wall while the host's — the one the
workspace's agents read (DESIGN §16.2) — stayed empty. Structural, not a
misclick: the loopback AuthCode flow binds its redirect beside the browser by
construction, and the spawn was the window's, so no spelling existed that put
the browser at the seat and the credential in the host's wall.

**The ruling: the sign-in is an act on the boundary, executed by the ENGINE
inside the named workspace's wall, streamed to the invoking seat.** bz runs
where the wall is, so the credential lands where the agents that need it run,
and the custody stance survives unchanged at both ends: DESIGN §5.1 #22 holds
— yog pipes the flow's lines and never a credential — and **nothing
credential-shaped ever crosses yog's wire**; the token moves from the provider
to the engine's own box over the provider's channel, exactly as a local
sign-in always did. What crosses the wire is bz's human-facing stream — the
authorize URL, a device code, a failure's reason and remedy — an N-frame
answer like any other (§3).

The alternative — sign in at the seat, deposit the credential to the host —
was weighed and is recorded in §11. §1.4 was deliberately NOT the argument:
that section governs the channel's own material, and a provider credential is
a different object; the deposit loses on custody (§5.1 #22, §6), not on
bootstrap doctrine.

**Two verbs, and the wire gains nothing (§3).** Implementation is bl-c285; the
window's consumption bl-1ddb; the flow branch bl-7c9f.

- `Action::Login { workspace, provider }` starts the run engine-side — inside
  the named workspace's wall, the same lens the config reads spend — and
  answers at once with the run's standing (the `Marks` re-read discipline):
  the intake is one thread for the whole world, so an act that waited out a
  browser-minutes flow would stop every deposit converging — the same reason
  `invoke` queues and answers a handle (§3).
- A follow-class read streams the run's lines: buffered from the start, then
  live to the outcome. Re-ask replays — a dropped lane, a re-attached seat and
  a settled run are one case (the `Query::Follow` discipline, §10).
- The run is engine RAM, one per workspace × provider. A second `Login` on a
  live pair terminates and replaces it — the operator's own restart is the
  cancel, so no cancel verb exists — and a run older than an hour is swept
  (§5.3's mailbox bound). The `ops.jsonl` outcome row appends engine-side: it
  was always world state.

**The flow follows the row's capability; the browser follows the human.** The
flow-selection rule is DESIGN §8.3's and is amended there (rule 1): the device
flow where the row declares a device endpoint, the browser flow everywhere
else. What belongs here is the wire consequence. A **device-capable row
completes from any seat**: the URL and user code paint wherever the lane is
held, the human finishes in any browser anywhere (a phone's included), bz
polls the token endpoint from the engine, and the credential never touches the
wire. A **browser-only row** completes only where a browser can reach the
ENGINE's loopback: the engine's own box (the window's local case, unchanged),
or an operator's own port-forward — an operator act on boxes the operator
administers, §1.4's own posture, **stated as the remedy where the seat is
remote and never built into the channel**. At the current brazen pin the one
builtin oauth row is browser-only and the projection carries no device fact;
the upstream ask that closes both halves is bl-c5fe.

**Faces (one capability, N spellings — §3).** The window's Login pane aims at
the FOCUSED workspace across channels (§8.2): a local workspace asks this
window's own engine, an entry workspace asks its entry's channel on its
entry's material, one pane either way. `yog seat` spells the same act and read
at a terminal. The run-by-hand fallback is per-channel: a local workspace
keeps `yog exec --ws … bz --login …`; an entry-hosted workspace spells the
`yog seat` act, because there is no `yog exec --ws` for a wall this box does
not hold.

**What this does not solve, on purpose.**

- A browser-only row signed in from a seat with no shell (a phone): the
  port-forward remedy assumes a terminal somewhere. The hole closes per row as
  bl-c5fe lands; until then that pairing has no paved path, and the pane says
  so rather than offering a verb that cannot finish.
- Re-auth when a token expires mid-conversation on the host: already owned —
  bz's silent refresh renews without interaction, and where refresh fails the
  auth-failed banner (DESIGN §8.3 rule 5) routes to this same one act, which
  any seat can now complete.
- A provider with neither a device endpoint nor a reachable redirect: the same
  stated operator remedy. No paste-back arm — §11.

### 8.4 Enrollment, and the QR envelope (bl-f4e3)

**The ruling** (operator, 2026-08-30): the boundary gains one act, spelled
`enroll` (American spelling, everywhere in this tree). It mints a new device's
leaf on the engine's own CA, seats that client's registration, answers the
material, and keeps no private key. §1.4 is **not** lifted, for the reason
that clause now states in place: the new device performs no channel act, an
operator-grade seat does, and the material travels the last hop out of channel.

**The act.** `enroll` addresses a workspace like every other gesture (§8), and
that is not decoration: the act *creates the registration*, and a registration
is the pair `(client, workspace)` — an enrollment naming no workspace would
mint a certificate that authenticates and sees nothing, leaving the operator to
finish the job with a `touch`. One act, one pair.

| Field | Meaning |
|---|---|
| `workspace` | the workspace the new client is seated in — `clients/<name>/workspaces/<workspace>` (§4.1) |
| `name` | the subject common name, which **is** the client identity (§2); refused on §4.1's own rule — one path component, never `local` |
| `grade` | `operator` or `foot` (§4.2), minted into the subject by the operator's own CA |

The reply kind is `enrolled` and carries six fields: `grade`, `name`,
`address`, `ca`, `cert`, `key`. `address` is **the engine's own wire address as
clients dial it**, read from `wire/address` where the boot records it (§8) — not
the port a `:0` request became, which only the listener knows and which is a
different number after the next boot. An `address` whose port is `0` therefore
**refuses**, naming `yog wire-certs WIRE_HOST=… WIRE_PORT=…`: a QR carrying a
runtime port would be stale before it was scanned.

**What is retained and what is not.** The engine mints the pair, reads it,
answers it and **shreds the key** — mint → answer → shred, the pattern the
manual recipe already follows, made unconditional so a failed read leaves no key
either. What stays on disk is the **certificate**, deliberately: it is public
material, and its presence is exactly what refuses a second enrollment under one
name (`provision::issue` — re-issuing distrusts nothing, so both would be live).
Keeping it is the guard; keeping the key would be the leak. Custody after the
answer is the transport's, and the two intakes differ: over the wire the answer
is TLS bytes and a seat's RAM (§6), while a deposit through the `gestures/`
inbox lands it in a reply file inside the world — on the operator's own box,
beside a CA that can mint the same leaf again at will, so it discloses nothing
to anyone who could not already mint. It does *persist*, and the remedy is `rm`.
The shipped path is the wire one, because the seat is what draws the code.

**The payload contract — this section is its authority.** The QR envelope is
**compact JSON** (no whitespace), the reply's six fields under a version marker:

```json
{"yog-enroll":1,"grade":"foot","name":"phone-1","address":"engine.invalid:7737","ca":"-----BEGIN CERTIFICATE-----\n…","cert":"-----BEGIN CERTIFICATE-----\n…","key":"-----BEGIN …-----\n…"}
```

(`key` carries the leaf's own PEM verbatim; its banner is elided above because
the disclosure gate reads this file too, and must never find one.)

`ok` and `kind` do not travel: they say what a *wire answer* is, and a
photograph is not one. `yog-enroll` is the marker a scanner recognizes it by and
the version it will be told about if the fields ever move. The seat and the
Android app build to this envelope; the engine returns the JSON and draws
nothing — QR rendering is the seat's job, on the seat's screen.

**PEM rides verbatim, and that is measured rather than assumed.** With the
certificates `wire/provision` actually mints — P-256 keys, 825-day leaves, one
`OU=foot` on the larger of the two subjects — the envelope is **1567 bytes** of
compact JSON (`ca` 570, `cert` 623, `key` 241, plus the JSON escaping of every
newline). A version-40 QR code in byte mode carries 2953 bytes at
error-correction level L, 2331 at M, 1663 at Q and 1273 at H, so the envelope
fits at L, M and Q and overflows only at H. DER-plus-base64 per field was the
fallback and is **not taken**: it measures 1359 bytes, buys ~13%, and costs the
one property worth keeping — a field an operator can paste into `openssl x509
-text`. The rule is therefore **PEM as minted, at level M or lower**; H needs a
smaller envelope and there is no payer for one. `boundary::dispatch::enroll`'s
`envelope` test takes that measurement against a real mint on every run, so a
recipe that moved to RSA would fail there rather than in a photograph.

**The corpus carries the shapes and none of the material** (§3). `request/enroll`
and `reply/enrolled` are additions, so `PROTOCOL` is **not** bumped — strict
decode already refuses an unknown op in band, and the drift ledger records both
at the standing version. Their fixture strings are fabricated and marked
`notreal`; a real minted key must never enter the corpus, and the key fixture
deliberately carries no private-key banner, because the leak gate reads every
committed byte in this tree and must never find one.

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
7. **Tool hosts:** advertisement, rendering, routing, the litany seam —
   after its own design pass (§5). **Landed, in three parts.**
   - bl-4e08 (§5.1): the `advertise` gesture in all three serializations, the
     per-client `tools.json`, connection-scoped presence, and the
     `Query::Clients` roster the workspace surface renders.
   - bl-c907 (§5.2): the litany seam filled (`Fx::tool_injection`, litany
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
   - bl-3455 (§5.2, later): the fourth op, `unload` — the symmetric
     subtraction the first pass deliberately left out. The set can now shrink,
     so a finished host stops riding every later assembly. It resolves against
     the durable document rather than the roster, which is why it is the one
     op that asks no engine.

   The day the leg landed the same call succeeded with nothing above it
   changing, which is what the interim refusal was shaped to make true: an
   in-band non-zero result is what a vanished endpoint had to produce anyway
   (litany's §3.3), so the seam was complete and honest before the transport
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

It was mechanized as `rules/no-engine-tree-in-paint.yml` over `src/shell/**`
plus the three modules that rendered rather than derived. The rule went with
the paint layer it governed (bl-7942); what it enforced is now structural, a
seat being a separate program that links none of this. The test is **not which
module a type lives in** — it is whether
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

**Narrowed for one class by bl-1dd3 (§4.2): a foot reads nothing.** The
sentence above is now about **operator-grade** clients, where it stands
unchanged and for its original reason. A foot-grade leaf may send three
gestures — advertise, wait, complete — and none of them reads a trail, a
board, a project or a transcript, so the residual does not exist for the class
most likely to be running on a box the operator trusts least. That is a
narrowing by *grade*, not the per-verb policy layer §11 rejects: two values, a
fixed set each, nothing to configure (§4.2 argues it in full).

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
- `Engine::asker` — a window took it, and the engine never did. It went with
  the window (bl-7942): what the engine hands a seat is a socket.
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
  **The config tab is the same shape and has no query at all yet** — when it
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
answers over the published `workspaces` set; a `litany new` that has just
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
- ~~The config tab has no query~~ — **spelled, bl-13f9**, below.
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

**The §11 inspector family crossed, and the config tab got its query
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
- **The config tab is `Query::Governing { workspace, agent, at }`**, all three
  serializations plus a help page, answered by `answer::inspector::governing`.
  `at` is the `Files` shape and for the same reason — it names *which commit*,
  which is the question and not the view — and absent it is the agent's own tip,
  resolved engine-side off the published snapshot so no seat has to know one
  before it may ask. It is the family's one read that **refuses** where its
  siblings answer absent: the derivation is a walk of the workspace's git and
  fails as `Lineages` fails, and "this conversation has no policy" is never a
  reading. Headless it is `/governing [--at <commit>]`.

  **The query survives the follow-the-tip ruling; what it *answers* inverted**
  (yog bl-e654, upstream bl-403b/bl-e580, operator ruling 2026-09-01). The
  address, the `at` selection and the refusal are untouched. The answer is no
  longer the commit an agent is frozen on but the commit it **resolves**: DESIGN
  §5.1 #17's followed-or-held derivation, where the fork's governing commit is
  the input and the lineage's current tip is the answer. So the reply's shape
  changed with it — the `"branch"` key, which used to say *the tip of the one
  lineage this frozen commit happens to head*, is now **`"follows"`**: the name
  of the lineage the conversation resolves, or `null` where it is held — beside
  **`"diverged_lineages"`**, an integer that is `0` when followed and the count
  of distinct config tips reaching the fork commit when held. One shape says
  both arms, because a `null` name with a non-zero count is exactly what held
  means. A seat renders it through `GoverningConfig::label()`'s two wordings and
  composes no sentence of its own; nothing here reads litany's stderr notice,
  and DESIGN §9.4 says why that is a rule and not an oversight.
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
  a file.** `no-engine-tree-in-paint` forbade paint code naming `GitTree`/
  `Agent`/`CommitNode`, and `Query::Agent` / `Reply::Agent` are the words a seat
  writes to declare *a payload the codec spells in both directions* — so the
  exception matched only an identifier whose scoped parent's path was `Query` or
  `Reply`. The rule is retired with the paint layer (bl-7942); the distinction
  it drew is the one this section is about, and it survives as the shape of the
  reply vocabulary rather than as a lint.

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
  Migrating the bar re-opened bl-9acf on the first run: a wall `litany new` has
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
  `config_tip` (the §2.2 lineage tip the §9.4 row's **apart** clause reads —
  what a conversation's followed commit is compared against, since bl-e654).
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
  leaf — the *same two words* `Action::Ball`'s own `Close`/`Assign` already take — so
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

### 9.9 The roster says *when* (bl-b7d9)

**`ConvRow` gained `last_active_unix`** — the conversation subtree's last action
as a **unix epoch second**, on `reply/conversations` beside the `age_secs` that
was already there.

The ask came from the phone's parity ledger, and §12.1's closing sentence is why
it belongs here rather than there: *the wire owes a phone nothing it does not
already owe a laptop.* Every list-shaped seat orders and **stamps** by last
activity. yog answered only the distance, so a seat could sort but could not
say *when* — and the fact was never missing from the engine, only from the
spelling. That is §9.4's finding a second time.

Three things were decided and each is the kind that rots quietly if it is not
written down.

- **It is carried, not derived.** A seat holding `age_secs` alone could
  subtract it from its own clock, and then every client says a different time
  for one instant — a phone with a drifting clock, a desktop in another zone,
  a headless `/conversations` piped into a script. The engine states the fact
  once and every reader repeats it. This is DESIGN §5.1's own rule reaching the
  wire: the source of truth is `Agent::last_action_unix` (§5.1 #12, bl-cad5 —
  the fold of the tip commit, the newest `messages/` mtime and the live
  streaming tail), gathered at snapshot time, and `nav::convs::row` now spends
  that one gathered value three ways instead of two: it orders the list, it
  ages the row, and it dates it. **Nothing new is stored and nothing is stat-ed
  from the answer path.**
- **Both time fields ride, and they are not two copies of one fact.** The
  obvious subtraction — replace `age_secs` with the stamp — is wrong, and not
  only because withdrawing a field in use is the one §3 change that *does* bump
  the version. `age_secs` is the distance from the **engine's** clock at answer
  time, and it is the reply's only carrier of that clock; the stamp is
  absolute. So the pair says strictly more than either half: a seat dates the
  row from the stamp, and a seat that wants the engine's `now` — to correct its
  own skew, or to keep a sitting list's ages honest without re-asking — adds
  the two. They cannot disagree, being one value encoded twice at one instant
  in one function, which is the only shape under which two spellings of a fact
  are lawful.
- **Epoch seconds, not RFC3339.** The wire's every other time is an integer of
  seconds — `committed` on the lineage row, `tick_secs`/`lease_secs`/
  `last_act_secs_ago` on the board, the §7.2 staleness stamp, `age_secs` itself
  — and the chokepoint's clock is an `i64` minted at the process boundary
  precisely so every derivation is deterministic under test (§9.7's staleness
  ruling). A calendar string would put a zone and a format into a codec that
  has neither, and would be the second representation this section refuses.

**The corpus moved and `PROTOCOL` did not, by the rule already in force** (§3).
`reply/conversations` gained a field, so its ledger signature moved and its
`since` advanced from 1 to **2** — the standing version, believed unreleased at
the time, so the move was taken as free exactly as bl-3655's follow-lane move
was. `make corpus` adjudicated it rather than an author: the ledger refuses only
a signature that moves while its own `since` **equals** the protocol being
generated. A client vendoring the corpus decodes the new key or fails its own
fixture, which is the whole mechanism.

**That belief was wrong, and §9.10 is where the correction is paid.** `PROTOCOL`
2 shipped in **v0.0.7**, which was tagged before this row landed — so the field
above was added to a shape at a *released* version, and the ledger let it
through only because `reply/conversations` had not yet spent its one move at 2.
The mechanism enforces *one move per shape per version*; the **rule** in §3 is
stricter and is the authority: *any* change to a wire-visible shape — gained
included — bumps the version. Read the two together as: the ledger cannot see
what has shipped, so it catches the second author and not the first. What saves
a client here is that the addition is optional and absent-by-default, so a v2
seat reading a v3 row ignores a key it does not know; what a bump buys is that
it does not have to.

### 9.10 The roster says *why it did not run* (bl-9b88)

**`ConvRow` and the §6 queue row gained `failure`**, the first clause of why a
conversation's latest model call failed, and the `agent` answer gained it beside
the `refused` class it already carried. `PROTOCOL` is **3**.

The engine has held the sentence all along and no row carried it. DESIGN §6
holds the derivation and the reasoning; what belongs here is the wire's half:

- **The clause, not the whole.** A row is a glance (§11), so the wire carries
  the provider's `message` — or the adapter's first stderr line — capped, and
  the whole of it stays one query deeper on `/steps`. A seat that wants more
  asks; a seat that only lists is not made to hold a stderr dump per row.
- **Absent, not null, on the conversation row**, the same discipline `pinned`
  and `alignment` already keep there: a reader must never have to tell *no
  failure* from *a failure with nothing to say*. A `bad` tone with no `failure`
  beside it therefore reads as exactly the third thing it is — a call that
  failed and left no words. The queue row spells it `null`, because that row's
  own encoder already spells `held` that way and one encoder should not hold two
  conventions.
- **The bump is the rule, not the ledger.** Three shapes gained a field, so §3's
  rule fires; `reply/agent` and `reply/conversations` had also each spent their
  one free move at 2, so the mechanism demanded it independently. Every shape
  whose signature moved now stamps `since: 3`, and the rest stand where they
  were.
- **`tone: "bad"` widened without moving a signature.** It fired for an
  auth-shaped refusal (bl-b43b) and now fires for any failed latest call. A
  seat's action is unchanged — paint the row wrong-coloured — and the spelling
  is unchanged, so this is a widening of when a value is true, not a change to
  what it says. The distinction matters: §3's rule is about shapes and
  spellings, and a value that becomes true in more of the cases it always
  claimed to cover is neither.

### 9.11 The queue says *somebody asked you to look* (bl-6f2f)

**The §6 queue row gained `flag`** — when a flag was raised on the
conversation, and why in the raiser's own words — and `signals` gained the
token `flagged` beside it. `PROTOCOL` is **4**.

DESIGN §6 rule 7 holds the derivation and the reasoning. The wire's half:

- **A new signal token is not a shape change.** `signals` is an array of
  strings and a seat that does not know `flagged` still reads the row; §3 says
  as much of every new spelling in an existing vocabulary. The `flag` object
  beside it *is* a gained field, and that is what bumps the version.
- **`null`, not absent**, matching `held` on the row `held` already spells that
  way — one encoder, one convention, and this row's convention was set first.
- **Two bumps in one unreleased cycle, and that is the ledger's granularity,
  not a second fleet event.** §9.10 raised 3; this raises 4, and neither has
  shipped, so no peer ever spoke 3 and the release carries one number. The
  corpus ledger refuses a shape whose signature moves while its `since` equals
  the protocol being generated, which is *per bump*, not per release — so a
  second change to a shape inside one cycle costs a second integer. That is
  cheap and honest; collapsing it would mean teaching the ledger what has been
  published, which nothing in this tree knows.

### 9.12 Which config governs is an answer that moves (bl-e654)

**`reply/governing` lost `branch` and gained `follows` and
`diverged_lineages`, and its `oid` changed meaning.** `PROTOCOL` is **5**.

litany's follow-the-tip ruling (upstream bl-403b, operator 2026-09-01) is
DESIGN §9.4's subject; the wire's half is that one shape now says a different
thing under the same verb, which is exactly §3's bump condition — *what a
spelling already in use is taken to say*.

- **`oid` was the fork commit and is the resolved one.** Before, it named the
  `config/*` ancestor an agent's branch forked off, a commit that never moved.
  Now it names the commit control actually reads at every step boundary: the
  followed lineage's head. A seat that painted the old value as *what this
  conversation runs* would keep painting, and would keep being wrong, which is
  why the number has to move even though the key did not.
- **One enum, two keys, and neither is redundant.** `follows` is the lineage's
  name and `diverged_lineages` is `0`; or `follows` is `null` and the count is
  how many distinct lineage tips reached the conversation and therefore held it
  on its fork commit. The decoder rebuilds the enum off `follows` alone and
  reads the count only where it can be non-zero, so the pair cannot decode to a
  state the encoder could not have written. `null` rather than absent, matching
  what the field it replaced spelled.
- **The count is derived, never scraped.** litany announces the same fact on
  its driver's stderr at every held step (`litany: notice: N diverged config
  lineages reach […]`). yog does not read that line and grows no contract that
  would let it: bl-b95e already deleted the marker table that classified
  litany's sentences and ruled content a diagnosis, never a trigger. Both sides
  run the same git query against the same refs, which is the one place the fact
  lives.

### 9.13 The rows say which tuning a provider takes (bl-23bd)

**`reply/providers` rows gained `effort` and `priority`, two booleans.**
`PROTOCOL` is **6**. The two new ops beside them — `/effort` and `/priority` —
cost nothing: §3's rule is that a new op is a new spelling in an existing
vocabulary and a peer that has not heard of one refuses it in band, by name.
One bump, for the row, and nothing else shape-changing is batched behind it.

DESIGN §9.4 holds the gestures and the reasoning; the wire's half is three
things.

- **The vocabulary is the operator's, and the dialect's stays in the adapter.**
  The wire says `effort` and `priority` because that is what the *config* says
  and what a person means. `reasoning_effort`, `thinking.budget_tokens`,
  `service_tier`, `flex` and `batch` are provider spellings; they do not appear
  in an envelope, a reply, a help page or a refusal, and the day a cost-saving
  lane is wanted it widens `priority`'s type rather than leaking one dialect's
  word upward. This is litany's own rule at ARCH §4.3, kept at the wire because
  a boundary that spoke one dialect's vocabulary would make every other dialect
  a translation.
- **A capability is a column of the row it is about.** The two booleans ride the
  provider row rather than a query of their own, for the reason bl-7407 already
  refused a second answer about the same subject: a seat would have to join it
  back, and a capability is a fact about a row exactly as its credential model
  is. They are read as columns out of `bz --list-providers --json` (brazen
  0.0.7, upstream bl-50a5) and re-derived nowhere — which is strictly better
  than the match on `ProtocolId` yog would otherwise have written, because
  brazen folds the dialect's own declaration together with **that row's**
  `unsupported_body_keys` decline, and only the row can see the second.
- **They gate a control, never a write.** A row that takes neither knob hides
  both selectors; it does not refuse the gesture. The config field is always
  lawful — a level a model declines is the provider's own refusal, arriving in
  the step's failure where it can be read — and a boundary that refused here
  would be refusing on the strength of a question that went unanswered. Absent
  is `false` at the column read and the booleans are **required** on the wire:
  an unanswered question may not block a row, and it may not grant it a control
  either, so the seat is told outright rather than left to infer.

### 9.14 What a workspace's roles are set to (bl-2410)

**A new read, `roles`, answering the focused workspace's role assignments** —
provider row, model id, effort level, priority lane, one row per role. It joins
the §9 config family as its sixth member (bl-719a folded that family onto one
carrier), so it is a member of an existing subject rather than a new one.

**It is an addition, and `PROTOCOL` therefore stays where bl-23bd left it** —
§9.8's reason verbatim: strict decode already refuses an unknown op in band by
name, and the drift ledger records both new shapes at the standing version. The
ledger is the authority on that and it was asked: `reply/roles` and
`request/roles` entered at the standing version, and no existing signature
moved. A bump was considered and rejected, because a bump is not free — every
client re-vendors — and there was nothing to batch it with: nothing else
shape-changing stood unshipped on `main`, and this change moves no shape.

**The ruling it answers** (operator, 2026-09-01, reversing the tuning design's
stays-out decision): a control should open showing what the workspace is
actually set to, not blank until this device sets something. The engine holds
that truth and no seat could ask for it.

- **`providers` was the wrong shape, and the reason is the subject.** That
  answer is per *provider row* — brazen's table, scoped to a wall, saying what
  each row is **capable** of, `effort`/`priority` included since bl-23bd. This
  one is per *role* — the workspace's own config, saying what has been
  **chosen**. Capability and choice are different questions with different
  cardinalities, and no projection of the first contains the second. Widening
  the rows would have made one answer carry two subjects joined by nothing.
- **`config` was the wrong shape for the opposite reason.** A seat can already
  read `providers.yaml` as raw text through the §9 read, and a seat that parsed
  it would be the second reader of an anchored block grammar litany's own
  parser is private for — REMOTE §9's whole shape is that the seat holds no
  derivations. One grammar, one reader, and the wire carries the rows.
- **It is read where the write lands**: the tip of the lineage §9.3 writes,
  which is the commit `/model`, `/effort` and `/priority` stage against and —
  under follow-the-tip (§9.12) — the one every conversation there resolves at
  its next step. So a seat that just wrote reads its own write back, and what
  it reads is what governs rather than what governed.
- **`effort` is the file's own word, `priority` is a boolean, and the asymmetry
  is deliberate.** The *gesture* asserts a level from a closed set; this
  **reports** what the file holds, and yog does not own that file — the §9.1 raw
  editor is the operator's authority. Flattening an unrecognized level to
  absent would say *nothing is set*, which is the exact defect this read exists
  to end, so the word rides across and a control can show that it does not
  recognize it. `priority` has no such case: `false` and omitted are one fact
  upstream, so there is nothing a stray word could mean but *not asking*.
- **Nothing set is an answer, not a refusal.** A workspace whose config cannot
  be read, or whose `roles:` block is absent or inline, answers the empty list.
  That is the opposite of the §11 `governing` read, which refuses — and the
  difference is real: *this conversation has no policy* is never true, while
  *this workspace has assigned no role* is a state every fresh world is in.

### 9.15 The roster says who owes this op a control (bl-8758)

**Every `reply/help` row gained `surface`, one word: `control` or `machine`.**
`PROTOCOL` is **7**. It is the interface-parity contract's single new fact
(`docs/PARITY.md` §2) — `control` means every seat-class client owes that op a
discoverable interactable, `machine` that it is spoken by programs and owed by
nobody. Sixty-one of the sixty-six rows are `control`; the five that are not are
the routing leg's own ends.

- **It rides `reply/help` because that is the artifact clients already vendor.**
  A parity gate reads its roster out of `corpus/reply/help.json`, replays it
  against its own walked inventory of `act:<op>` tags, and never reads another
  client's tree — which is what makes the ledger one fact with one home rather
  than a pairwise diff that goes quadratic at the third surface. A second file
  beside the corpus would have been a second list to keep true, and §3 already
  refuses those.
- **The unit is the op token, never the `Action` variant.** yog folds families
  (`Ball`, `Monitor`, `Fan`, `Route`, `Tune`) to one variant over a family verb
  and the wire spells each member as its own `op`; a control fires an op, so a
  folded family is classed per member. `HelpRow::verb` is that token, and it is
  already the envelope's `op` and the corpus filename.
- **The bump is the rule, not the ledger** — §9.9's correction, applied. Help's
  signature last moved at **1**, so the mechanism's one-move-per-shape-per-
  version test would have passed a change at 6 in silence. §3's rule is
  stricter and is the authority: a field *gained* on a wire-visible shape bumps
  the version. Here the bump is also the point rather than a tax, because the
  classification only reaches a client through a re-vendor and a bump is what
  compels one. §9.14's `roles` rides it: that read declined a bump of its own
  for want of anything to batch it with, and this is the thing to batch it
  with — the re-vendor it did not want to force happens once, for both, inside
  the same unreleased cycle.
- **`enroll` carries a class like every other op**, being an ordinary help row
  (§1.4's mint is an operator act at a seat, so `control`). What carries no
  class is yog's **argv** surface — `yog env`, `yog bl`, `yog exec` and the
  rest are process-level words crossing no §8.5 boundary, so no seat can owe
  one a control; they share the row type for its text and never reach this
  reply.

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
  to argue the same case. **The next one is arguing it now (§14.4, bl-5f41):**
  the attention lane's case is the criterion inverted — not an ask rate an
  operator could not read at, but an asker that cannot re-ask (a pocketed
  phone) — and the ruling is pending there, not here.
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
  by ruling; convenience flows reopen the exact surface mTLS closed. **What is
  rejected is the DEVICE enrolling itself**, and bl-f4e3 sharpened that rather
  than lifting it (§1.4, §8.4). The banned shape is a machine holding no
  material opening a connection and being handed some — a pairing code, a
  claim token, a first-connect ceremony, a "trust on first use" window, any of
  which is an unauthenticated peer the engine must answer. The `enroll` act is
  none of those: the new device says nothing, an **operator-grade seat**
  performs the act over a channel already authenticated by material that
  already moved out of channel, and the result leaves the wire in the
  operator's hand. It is the operator's own tooling reached through the
  boundary — the same class as §8's engine-boot mint — and it is on the
  boundary rather than in the wire because §3 forbids a capability that exists
  on one face and not the others. The unlifted half is the one that matters:
  **no gesture will ever be answered to a peer that could not already
  connect.**
- ~~**A separate client crate or binary for v1**~~ — **lifted by the bl-37fd
  ruling (§12)**: the four-component split makes the seat its own crate and
  repo. The rejection's reasoning — no payer for a split — held until the
  ruling became the payer. **Landed (bl-0716):** the seat crate exists and
  reimplements this document, as §12's authority ruling requires and as the
  android client already did. yog keeps its window for one deprecated release
  (bl-7942 deletes it), so §8's old paragraph survives there era-stamped
  rather than deleted.
- **A wrapper meta-tool — one declared tool carrying an untyped payload**
  (bl-71d0, reasons in §5.2). It loses emission-time validation, since the
  provider validates the wrapper and not the call inside it, and it makes the
  adjudication chokepoint parse a payload before it can see what is being
  asked. Its one virtue, discovery without a server-side change, is what `load`
  already delivers. Host-qualified names (§5) are the answer that costs no
  verb.
- **A hosts list on the invocation — fan-out by broadcast** (bl-71d0, §5). One
  adjudication decision stands for one execution on one machine; a list makes
  it stand for N, with one refusal, one deadline and one capture to describe
  all of them. Fan-out, if it ever pays, is a distinct tool making sub-calls
  through the same chokepoint, never an overload of `invoke`.
- **Shipping the server's world fold across the wire** (bl-71d0, §5) — handing
  a remote executor the engine's composed env (`LITANY_HOME`, `XDG_STATE_HOME`,
  DESIGN §16.2) so its tools "see the same substrate". Those paths exist on the
  server and nowhere else; the executing end supplies its own environment, and
  substrate access is a tool the executing end advertises or an engine act.
  This is the client-side twin of the next entry.
- **Syncing or mounting the world at clients** (network filesystem, rsync,
  shared checkout) — the world never leaves the server; clients receive
  replies, not files. Two full engines over one networked disk is the
  instance-coordination shape DESIGN §14 rejects, kept rejected.
- **Per-tool or per-verb ACLs in v1** — registration is workspace-grade
  trust; a finer policy layer is speculative until a second human exists.
  **Unamended by bl-1dd3, which was checked against it** (§4.2): the foot grade
  is two values with a fixed gesture set each, in the code, with nothing an
  operator writes per verb or per tool — a policy layer is the thing with a
  place to put a rule, and this has none. Adding an `Action` adds no row
  anywhere; the foot set is enumerated, never subtracted from.
- **MCP inside the engine** (bl-1dd3, §5.4) — a second vocabulary on the
  surface §3 keeps down to one, and a transport yog would have to speak to
  reach a box it does not administer. MCP's place, when something pays for it,
  is **thrall-local**: the foot is an MCP client on its own box and
  re-advertises what it finds as ordinary §5.1 elements, so the ecosystem
  arrives without the engine learning a word of it. Deferred, not v1.
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
- **A credential deposit act — sign in at the seat, ship the credential to
  the host's wall** (bl-61bf, §8.3). The browser UX would be native, but a
  bearer credential rides a boundary payload: yog reads it at the seat,
  transports it, and writes it at the engine — custody at three points where
  DESIGN §5.1 #22 rules yog touches a credential at none — and the seat mints
  a durable secret, which §6 rules a client never holds (key material and RAM,
  nothing else). The engine-side act buys the same capability with no secret
  in channel.
- **A paste-back completion arm for browser-only rows** (bl-61bf, §8.3) — the
  seat's human copying the redirect landing back into the stream. It would
  give the login surface its first INPUT path — a boundary act feeding a
  child's stdin, and a change to the streamed-piped spawn class, which is
  stdin-null by design (DESIGN §8.3) — to serve a flow the vendor's own device
  flow supersedes (bl-c5fe). Revisit only for a provider with neither a device
  flow nor a reachable redirect, once one exists with a payer; until then the
  stated operator port-forward is the remedy.

## 12. The four-component split (adopted 2026-08-28, bl-37fd)

Operator ruling: the harness becomes four separately installed components
meeting only at the wire; brazen stays the provider adapter beneath the
engine. This section is the ruling's home — the components, the invariants and
the order live here; every other section keeps describing the tree as it
stands and is amended by the ball that moves it.

- **yog** — the standalone server: holder of the world, the balls, the
  conversations. No UI, no execution. **Landed** (bl-7942): what was `yog serve`
  is the whole binary, and the verb is gone with the second face it named.
- **lernie** — the seat: the window and the seat-client face, extracted into
  their own crate and repo (bl-0716, **landed** — the crate's first landing is
  the wire client and the verb over it; the window is filed in that
  repository's own store, because it needs a reply vocabulary the seat had not
  had to own). The crate name flips at a version
  fence: the engine's line under it ended at 0.0.11, the seat's begins at
  0.1.0, and both READMEs state the fence — the published record carries two
  eras of the name, and the fence is the one disambiguation rule. **Read
  every `lernie` in yog's tree against it**: a bare one names the seat or
  this ruling, and one bound to a `0.0.x` version names the engine at that
  release.
- **litany** — the agent-loop engine, renamed from lernie (bl-9905, upstream
  bl-2f58) and pinned exactly; `Cargo.toml` is the pin authority and no version
  is restated here. The rename took three durable-state
  surfaces with it and shipped no compatibility shim: `LERNIE_HOME` →
  `LITANY_HOME`, the XDG harness subdirs `.../lernie` → `.../litany`, and the
  in-workspace mark namespace `refs/lernie/*` → `refs/litany/*`. yog's nested
  world (DESIGN §16.2) carries all three, so a world founded before the fence
  needs the migration recorded there; yog refuses to seed over one rather
  than founding a rival empty home beside it.
- **thrall** — the foot: §2's tool host severed into its own installable
  (bl-1dd3), carrying a foot-grade certificate — advertise and execute only,
  no ask, no act. The name is likewise held. **The protocol half has landed
  here**: the grade is §4.2, the component and the local execution corpus it
  owns are §5.4, and founding the repo is the ball's other half.

Two invariants ride with the ruling:

1. **Front door only.** Every execution is transported over the real wire —
   real socket, real handshake, real leaf — extending §1.2's bl-ae05 ruling
   from the window to execution itself. No in-process executor, and no
   unix-socket second transport in v1: one transport, one code path, no
   place to hide the bug.
2. **Ship inert.** A yog with zero enrolled thralls is valid and is the
   default. The server is structurally incapable of executing anything until
   a thrall is enrolled — even a single-box install enrolls its local thrall
   as an explicit operator act. A tool call with no thrall to route to
   refuses in band (§5's use-is-attempt), which is the posture working, not
   an error state.

Consequences the ruling fixes, each landed by its own ball: with the front
door extended to execution there is exactly one invocation pipeline —
adjudicate → mailbox → execute → capture — and the engine's driver keeps no
local executor (bl-fe61); tool locality is host-qualified and subject-local,
with the server's world fold never shipped across the wire (bl-71d0);
separately installed ends make version skew possible for the first time, so
the handshake carries a protocol version and a mismatch refuses fail-closed,
naming both versions (bl-a670). With the versioned wire, this document
becomes the protocol authority all four components implement against, and
each repo's DESIGN governs only its own component.

Migration order (strangler; each step ships green): thrall founded (bl-1dd3,
landed) → engine renamed (bl-9905, landed; the pin lives in `Cargo.toml`)
→ seat severed (bl-0716, landed) → yog drops the UI and goes inert by default
(bl-7942, **landed** — the order is complete).

**The window's deprecated release was never cut, deliberately.** This order
gave yog's window one release of overlap while the seat crate was stood up. The
operator pulled the cutover trigger with every component's publish batched into
ONE coordinated moment, which makes a deprecation release a publish nobody
consumes — and the only deployments are the operator's own dev box and one
server, neither of which needs the overlap. So bl-7942 dropped the window
outright rather than deprecating it, and the strangler's middle step ended one
release earlier than planned.

Supersessions, so stale prose is not read as live: §8's "one crate, one
multi-call binary" is **era-stamped in place** and describes what bl-7942
deleted; §11's rejection of a separate client crate is lifted in place above.
And DESIGN §11 — the UI structure — is **retired** by bl-7942, with a reading
rule at its tombstone for the sentences elsewhere in that document that still
describe a face.

### 12.1 The phone is one more box (bl-15bd)

**Operator ruling 2026-08-30:** the Android app is named **yog** and ships all
three runnable components — the seat, the foot, and the server — each gated
behind an explicit **bootstrap** rather than auto-started. The default
bootstrap is mTLS client enrollment: a seat or a foot dialing a host engine on
material provisioned out of channel (§1.4). Running the server *on the phone*
is allowed, and is the deliberate non-default choice.

**This document is not amended by it, and that is the point.** A phone is one
more box — it holds §8.2 entries, presents a leaf, is scoped by its §4.1
registrations and graded by §4.2 like every other client. §1's canonical scene
already had it: *"a phone seat talks to a conversation."* The app is a
**packaging** of components this section already names, and it adds no noun, no
verb and no protocol. The one thing worth writing down is what an app that
ships three components does *not* get to invent:

- **Which component a launch runs is derived, never stored.** No material and
  nothing runs; the grade on the certificate says seat or foot (§4.2). So
  enrolling a phone as a tool host is **minting it a foot-grade leaf**, not
  tapping a setting on the phone — there is no stored choice that could
  disagree with the certificate. The first-run surface has no button, on
  purpose: §1.4 stands on a phone exactly as it stands on a laptop, and a
  button that acquired an identity would be the in-channel bootstrap that must
  never exist.
- **The wire is §3's, whole.** The app states its version and its request in
  one breath and confirms the engine's on the way to the answer, refusing a
  mismatch fail-closed in the server's own sentence (§12's bl-a670). It
  replays the vendored corpus in **both** directions — a client that only
  sends requests still decodes the request fixtures — and §3's third rule
  reaches *inside* an envelope, so a frame stating a rung that client does not
  spell is refused rather than flattened into the shape it has.
- **A local server on the phone is a platform question, not a protocol one.**
  Nothing in the Rust half stops it: yog cross-compiles for
  `aarch64-linux-android` with the whole substrate in the graph and no C
  toolchain acquired. Two rungs do, and neither is here — Android ships no
  `git`, and yog's world seeds shell shims its own agents run into app-private
  storage, which that platform has refused to execute since API 29. The second
  is yog's own shape and is filed as bl-6a6a; the app offers the bootstrap,
  states both blockers in the operator's terms, and starts nothing.

Work lands in the Android repository with its own tracking; this section is
yog's record of the ruling and of what the wire owes a phone, which is nothing
it does not already owe a laptop.

**The parity ledger's asks land in §9, not here** (bl-b7d9, the first of them).
The phone's ledger is a list of facts a list-shaped seat needs and yog does not
answer; each one is a payload question about what the engine can *say*, which is
§9's subject, and each is owed to every seat the moment it is owed to one. The
sentence above is the test: an ask that is really about a phone would be an
amendment here, and none has been.

**Interface parity between the seats is its own contract, and its authority is
`docs/PARITY.md`** (bl-80c1): each client is judged against the help table's
op roster — classified `control`/`machine` at the table and published through
the vendored corpus — by its own accessibility inventory, never against the
other client. Deliberate absences are cited exemption config in the client's
repo, loud and machine-checked in both directions.


## 13. The punched wire — dynamic reachability (bl-4d56, reworked bl-aad5; operator rulings 2026-08-31)

§8 makes the address one fact with one home, and §1's canonical scene puts the
engine on a home server behind residential NAT: its public address is not its
own fact, it cannot accept an unsolicited inbound connection, and the phone
seat dialing it sits behind a cellular NAT stricter still. The deployed answer
today is a third-party overlay network, and removing that dependency is this
section's reason to exist — the same sovereignty argument §4 makes for the CA.

The first cut (bl-4d56) stood the design on a rented public anchor performing
presence, introduction and carriage. **Three operator rulings, same day,
reshaped it**, and each is stronger than a cost preference:

1. **No rented anchor.** The operator's reachability stands on no box rented
   for the purpose. Discovery rides a commons instead (§13.2).
2. **No port-forward — and no publicly dialable listener at all.** Not merely
   "the router cannot be asked": a server that *cannot* be dialed unsolicited
   is desirable in itself. §4's posture — the open-internet surface is the TLS
   handshake and nothing behind it — is superseded upward for the roving case:
   the surface is **nothing**. The engine's NAT drops unsolicited SYNs; there
   is no port to scan, no handshake to probe, no listener to fingerprint.
3. **The data path is hole-punched.** Both ends send outbound; a connection
   exists only between peers that arranged it. A client on a stable, public
   connection is simply the easy case of the same mechanism — the punch is
   symmetric, and whichever end's SYN lands first wins.

What survives the rework untouched: the inner wire crosses byte for byte —
mTLS on the entry's material, the §3 preface, §4's identity and scoping — and
severability at every layer. What the rework deletes: the splice, the
TLS-in-TLS, the moor. A punched TCP stream carries the wire *directly*; the
engine's serving loop takes it exactly as it takes an accepted connection.

### 13.1 The physics, restated under the rulings

The anchor's three functions decompose differently than the first cut assumed.
**Presence and signaling can ride a commons** — the BitTorrent mainline DHT, a
public keyed store nobody owns, with no account, no coordination plane, and
bootstrap nodes that are third parties only in the sense root DNS servers are.
**Carriage cannot**: bytes need a mutually reachable box, and by ruling 1
there is none. This design therefore ships **without carriage, and says so**:
a NAT pair that refuses a punch has no path — not a slow one, none. That is
accepted with open eyes, bl-a9b0 measures where it bites (the
cellular-CGNAT-to-residential pair is the risk case), and §13.6 parks the
first cut's relay as the named exit if it bites where it matters.

### 13.2 The DHT rendezvous

**Both ends are pure clients of the mainline DHT.** Outbound-UDP iterative
lookups and BEP 44 get/put — never a node: no routing answers, no storage
offered, no listener, nothing for the commons to dial back. The
implementation is bencode, the KRPC query shapes, and BEP 44's
ed25519-signed mutable items; `ring` (already in the graph under rustls)
signs, std UDP carries, and the expectation is **zero new crates** — a
proposed dependency here is a rule-6 ruling, not a default.

**The material grows three facts, all minted out of channel** (§1.4's posture
byte for byte): an ed25519 rendezvous keypair per engine, one per client, and
a random **pairing salt** per entry. Public keys and the salt cross inside
the entry exactly as `ca.pem` does. The durable fact an entry holds stops
being an address and becomes *the key that finds one*.

**Presence** is the engine's mutable item: its observed endpoints (v4 and v6,
and the fixed local port it punches from), sealed under the pairing material
so the commons stores opaque bytes — a DHT node, or anyone walking the
keyspace, learns neither that this is an engine nor where it lives. Signed,
sequenced, republished on an hourly cadence against the DHT's storage decay.

**The inbox** is the signaling half: a client that wants a connection writes
a signed, sealed item at a key derived from the pairing salt — *"call me:
these observed endpoints, this nonce"* — and the engine polls that key at a
15-second cadence. Sealing and signing close the two abuses the commons
invites: an item the engine cannot verify never draws a SYN (so nobody can
make the engine disclose its address by writing garbage), and without the
salt an observer can neither find the inbox nor read a client's endpoints
from it. Both cadences are stated defaults, revisited on evidence, and both
live in frames and files tests can speak — no socket options, no timers the
suite cannot fake.

### 13.3 The punch is the data path

Both ends bind their fixed punch port and TCP-simultaneous-open toward each
other's observed endpoints — listen and connect from one port, SYNs retried
across a bounded window, v6 tried first where both ends published it
(a stateful v6 firewall punches far more reliably than a v4 NAT, having no
ports to rewrite). The first SYN pair that crosses is the connection; the
seat's inner mTLS runs over it against the same `ca.pem`, verifying the same
engine name a direct dial verifies, and the engine reads the same client
leaf. §4 is untouched. A client that is publicly reachable is the degenerate
case: the engine's outbound SYN simply lands, and the wire neither knows nor
cares which end's SYN won.

**Source-port reuse is now on the critical path** — the punch needs
`SO_REUSEADDR` (and on some platforms `SO_REUSEPORT`), which std does not
expose. The first cut deferred this ruling behind a criterion; the criterion
is spent, and the choice is the same two doors: a `socket2` dependency
(rule 6: explicit approval required) or `setsockopt` beside the four effects
in `sys.rs` (rule 3: a location-rule amendment). **Recommended: `socket2`** —
pure Rust over libc, no C toolchain, one lockfile line, and it dissolves a
rule-3 widening for calls that, unlike the four residents, run per-connection
at runtime rather than once at the process edge. Awaiting the ruling either
way (§13.7).

### 13.4 Held connections — §10's criterion is met

§10 has refused every held connection for want of a payer; the punch is the
payer, and the criterion is met in its own stated terms: a dial stops costing
milliseconds. First contact costs an inbox write, a poll period and a punch
window — seconds — so the punched connection is **held and reused**, with
idle ping frames at a 25-second cadence keeping both NATs' mappings alive
(frames, again, because std exposes no keepalive and a frame is testable).
The ladder a dial runs is now: the live held connection; the entry's direct
`address` where one exists (a LAN, a stable client, loopback); a re-punch at
the RAM-cached endpoints, which costs no DHT round trip; the full rendezvous.
What worked stays RAM for the run — the runtime half of §8's `:0` discipline,
never disk. Per-request identity is unchanged in the only sense that matters:
the certificate is read at the handshake and the scope is spent per request,
exactly the terms the follow lane already holds its connection on.

Severability, per end: an engine whose wire root holds no rendezvous material
punches nothing, polls nothing, starts no thread — it is today's listener,
byte for byte. An entry with `address` and no rendezvous material is today's
entry. The phone pays nothing for the machinery when idle: only the engine
polls; a client touches the DHT only at the moment it wants a connection.

### 13.5 What it costs, named

- **First contact is seconds, not milliseconds** — the poll period bounds it.
  Held connections hide it from every ask but the first; the cold open of a
  seat is where it shows.
- **No carriage.** A punch-refusing NAT pair has no path. Accepted by ruling;
  measured by bl-a9b0; §13.6 is the exit.
- **The commons is a dependency, honestly smaller.** Bootstrap nodes are
  third parties (cached routing tables outlive them); a network that blocks
  outbound UDP blocks the rendezvous itself, and the ladder's direct-address
  rung is the only fallback there.
- **The engine emits, quietly.** Hourly republish, a 15-second poll, pings on
  held connections. Nothing listens, but the box is not silent — a traffic
  observer on the local network sees DHT chatter where the first cut showed a
  single TLS stream to a known anchor.
- **The coverage floor meets a distributed system.** The DHT client is tested
  against a fake DHT the suite runs on loopback UDP — designed for that from
  the first line, or it cannot be built (QUALITY's rule, not a hope).

### 13.6 The carriage rung, parked

The first cut's relay — one stateless process on a public box, outer mTLS on
the operator's CA, `moor`/`reach`/`back`, dial-back and splice (bl-4d56's
§13.2, in the store's history) — remains the designed exit, unbuilt. Its
criterion: **a client the operator actually needs measurably cannot punch**
— bl-a9b0's verdict, or a live pairing that fails where it matters. Standing
one up then is severable in both directions: one file per entry opts in, and
no code the punched path runs knows the relay exists.

### 13.7 Rejections and open rulings

Rejected, with the rulings that decided them:

- **A port-forward plus dynamic DNS** — the boring answer, and it was asked
  (2026-08-31): rejected not on CGNAT fragility but on posture — ruling 2
  *wants* the unsolicited-inbound surface to be nothing.
- **A rented anchor as the primary path** — the whole first cut; superseded
  by ruling 1, preserved as §13.6's exit.
- **DNS as the discovery layer beside the DHT** — redundant: signaling needs
  the DHT regardless, and two homes for one fact is the drift rule 6 of the
  house exists to refuse; a registrar API credential on every client is its
  own disqualification.
- **A QUIC/UDP data plane, an embedded wireguard, UPnP/NAT-PMP,
  stream-multiplexing** — all stand rejected as in the first cut, for the
  recorded reasons (rules 6/8; a second key universe; a router favor; a
  framing protocol nobody needs).

Open, awaiting operator ruling:

1. **`socket2` versus `sys.rs`** for source-port reuse (§13.3; `socket2`
   recommended). This is the one gate on starting the punch work.
2. **Where the DHT client lives.** Each component reimplements the wire by
   §8's no-shared-crate rule — but that rule guards the *protocol authority*,
   and a DHT client is substrate, not protocol. Recommended: build it inside
   yog first, measure its true size, then decide whether the seat, the foot
   and the app reimplement (a Kotlin half exists regardless) or consume it as
   a published crate of its own.
3. **Cadence defaults** (15 s poll, hourly republish, 25 s ping) — stated so
   they can be wrong in public; revisit on evidence, not taste.

**Component impact.** yog: the DHT client, the rendezvous loop (publish,
poll, verify, punch), held-connection serving, and the mint growing the
ed25519 pairs and pairing salt. lernie: the client rendezvous (presence get,
inbox put, punch) and the four-rung ladder. thrall and the app: the same —
and thrall's redial discipline (bl-0a74) becomes load-bearing, since a foot
that does not come back has no roving value. The app's half is Kotlin and
lands in its own repository (§12.1 holds). Deploy: prove the punched path
live from a laptop and from the phone, then walk the engine off the overlay.

## 14. Attention without a push path — the pocketed phone (bl-80d0)

Attention is DESIGN §6's derived predicate — an unacked notify, a
conversation at rest at a tip you have not seen, an exhausted budget, a
conflict, mail nobody is driving, a held tool call — and on this wire it
exists only as an **answer**: `Reply::Attention`'s queue rows and the
rollups inside `Query::Workspaces`, each computed per ask under the asker's
scope (§9.7). The engine initiates nothing toward a client — §3's posture,
restated by §13 as the punch: both ends arrange a connection, neither is
dialled cold. So a phone seat learns its turn has come at its next read, and
a phone in a pocket performs none: the platform ends a backgrounded app's
sockets and schedules it nothing. The gap is real and it is
platform-shaped. On Android exactly three things wake a sleeping app — an
OS-scheduled job, a foreground service that never slept, and the OS
vendor's own push relay — and no wire design adds a fourth. The design
space below is that list, entire.

**The reframe: the wire needs no push path, because a standing ask already
is one.** This document dissolved push-versus-pull once (§3: the streaming
form is not a second form; §5.5): a follow-class read is an ask that
stands, and every frame after the first is still an answer to it. What a
pocketed phone lacks is not a channel the engine could push down — it is
the *right to keep its ask standing*, and that right is the operator's to
grant on the phone (a foreground service is exactly that grant), never the
engine's to manufacture. So the design is one small lane plus an honest
ladder of what a phone does with it, and this ball's own title survives
byte for byte: attention exists only when a client asks; the ask may stand.

### 14.1 The attention lane

**`Query::Attention`, answered as a sequence by an intake that can hold.**
The wire spelling does not move: the query and `Reply::Attention` stand as
they are, and — exactly as `Query::Follow` already behaves — an intake that
cannot hold a connection is answered with the answer as of now, one frame,
which is today's read byte for byte. What the lane adds is the holding
intake's cadence: the first frame is the answer at connect, and a further
frame is written whenever that asker's answer **changes**, discovered at
the derivation worker's own republish — no new clock, no new watch, no
engine-side subscription noun. The scope is spent at connect and the
identity is per request, a held read being one request (§4, the same terms
the follow lane and §13.4 already hold on).

**Frames replace; they never append.** The opposite of §5.5, on §5.5's own
argument: the append flip existed because a transcript answer grows with
the conversation and re-sending it was quadratic. An attention answer is a
handful of small rows that grows with nothing, so a delta encoding here
would buy a fold contract to save bytes that never multiplied. A seat
paints the last frame it holds, and a seat that drops the lane re-asks and
is whole on its first frame — the same no-reconciliation property, had for
free.

**Liveness is not this lane's to invent.** On a §13.4 held punched
connection the idle ping at its stated cadence (§13.7's ruling 3 owns the
number) is what proves the peer; on a direct connection the follow lane's
bounded-hold discipline applies unamended — the hold ends, and the lane
re-asks, a stream that ended and a dial that failed being one case (§10).
Severability is the follow lane's too: nothing runs for a seat that never
holds the lane, and a foot never reaches it (§4.2).

### 14.2 The ladder on the phone

Three rungs, each an operator's choice, and whose work each is — §12.1
holds, so the app halves land in the app's own repository:

- **Rung 0 — the foreground read.** The status quo, stated: a seat on glass
  asks at cadence; pocketed, it learns at the next look. Not a defect to
  paper over — it is the whole answer for an operator who treats the phone
  as a glass they pick up, and it costs nothing.
- **Rung 1 — the scheduled fetch.** App-side only, no wire change, and the
  default. The app schedules the platform's own periodic work; each run
  performs one ordinary ask — direct or punched, `Query::Attention`, one
  frame — and paints the rows as a local notification. The OS owns the
  cadence: a fifteen-minute floor, batched into Doze maintenance windows,
  hours apart in deep sleep. Zero new trust and zero engine work; the
  punch's seconds-cold first contact (§13.5) is invisible inside a
  scheduled job. What it does not solve is timeliness, and it cannot — that
  bound is the platform's, not the design's.
- **Rung 2 — the held lane.** The timely choice: a foreground service holds
  one §13.4 connection with the attention lane standing on it; a frame is
  the wake, and the notification paints the rows the frame carried — no
  second read. In-posture and third-party-free. Its costs are the phone's
  and they are real: a permanent notification (the platform's price for the
  grant), the ping cadence's radio wakes, and vendor task killers that
  fight even granted services. Off by default, enabled as an explicit
  operator act — §12.1's bootstrap discipline.

### 14.3 Shapes attacked and refused

- **Engine-initiated contact** — the engine punching toward, or writing the
  rendezvous inbox of, a phone at the moment attention fires. It fails on
  physics before posture: a pocketed phone's NAT mapping is dead, its app
  polls no inbox and holds no socket, and a punch needs both ends awake —
  the SYN lands nowhere. Where it could land (a phone on glass) the pull
  already answers. And it inverts §3: the engine initiating toward a
  client is the one direction this wire has never had.
- **A third-party wake relay** — the vendor push service, or a UnifiedPush
  distributor. Weighed honestly rather than waved off: it is the *only*
  Doze-proof, zero-battery wake, because the platform blesses exactly one
  always-on socket per device — its own — and a content-free ping
  ("something changed; ask") keeps every attention fact inside mTLS. What
  it costs is the posture itself: an account and a credential held by the
  engine, a vendor SDK inside the app, a timing-metadata stream naming
  each moment this operator's engine wants this operator's device, and a
  device token that is a durable third-party name for the phone. The
  self-hosted-distributor variant trades the vendor for a publicly
  dialable box, which is §13's rulings 1 and 2 reversed. Recommended:
  refused, and parked §13.6-style with a criterion (§14.4) rather than
  deleted — the honest record is that this rung exists and what it costs,
  not that it does not work.
- **An out-of-band adapter** — mail or messaging at attention-fire, riding
  a push channel the operator already carries. Refused: it exports the
  fact out of the wire's mTLS into a channel with its own custodians,
  grows an engine-side adapter and a credential where DESIGN §5.1 #22
  says yog holds none, and wakes a mail client, not a seat.

### 14.4 Open, awaiting operator ruling

1. **Admit the attention lane** (§14.1) as the third follow-class read —
   filed as bl-5f41. §10's held-connection row stands on every candidate
   arguing its own case; this one's case is not a rate an operator could
   not read at but the inverse, an asker that cannot re-ask. Recommended:
   **yes**. This is the one gate on the engine work (bl-09aa), and rung 2
   consumes it.
2. **The wake-relay class** (§14.3) — filed as bl-4dea. Recommended
   default: **refuse now, park behind a criterion**: reopen only if an
   operator's actual device measurably cannot hold rung 2 *and* rung 1's
   latency bites where it matters — bl-a9b0's shape, evidence rather than
   taste.

**Component impact.** yog: the follow arm of the wire intake widened to
answer `Query::Attention` as a sequence — change-detection at the worker's
republish, frames that replace, the one-frame answer untouched for every
intake that cannot hold (bl-09aa, gated on ruling 1). lernie: nothing — the
window never sleeps mid-ask. The app: rungs 1 and 2, filed and tracked in
its own repository (§12.1); rung 1 needs nothing from anyone and can land
today.
