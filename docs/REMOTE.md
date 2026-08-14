# yog — The Remote Seat (client/server split)

Status: normative for the split; direction adopted by operator ruling
2026-08-13 (bl-b9a2). Nothing in the tree implements this yet — §9 is the build
sequence, and each step lands under its own ball. `docs/DESIGN.md` remains the
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
- **Framing** is length-delimited JSON request/reply over the TLS stream, with
  one streaming form for follow-class reads (the live tail), same envelope,
  chunked. The exact framing is an implementation-ball decision, not doctrine.
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
  enrollment surface §1.4 forbids.)

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
current headless boot plus the wire listener); bare `yog` runs the window as a
pure client. Local desktop use is two processes on one box over loopback mTLS,
certificates provisioned by the same out-of-channel act (a make target may
script the local CA; that is operator tooling, not an in-channel protocol). A
TUI or web seat later is another consumer of the same wire and needs nothing
new from the engine.

**Paths never cross the wire.** Boundary types today address workspaces and
projects by absolute `PathBuf`; across machines those are meaningless and a
disclosure besides. The wire spelling is the **name** (workspace name, project
display name); the engine resolves names to paths at the dispatch chokepoint.
This migration is a pre-wire refactor of the boundary types themselves (§9.3).

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
4. **The shell paints only boundary payloads** — retire the remaining raw
   `GitTree`/`Agent` imports from paint code.
5. **The wire:** mTLS listener in `yog serve`, client transport in the shell,
   the window becomes a seat of loopback by default. The deposit inbox
   remains for in-world callers (§3).
6. **Registration and scoping:** the per-workspace registry, reply filtering,
   auto-registration on create, per-seat `ui.json` split (§7).
7. **Tool hosts:** advertisement, rendering, routing, the lernie seam —
   after its own design pass (§5).

## 10. Open questions (living)

- The follow/streaming frame shape, and when polling graduates to a
  subscription channel.
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
