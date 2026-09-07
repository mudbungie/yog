//! **The wire version** (REMOTE §3): the integer each end writes in its
//! preface, and the changelog of what moved it. Split off [`super`] at §12's
//! cap — the constant is one line, the record of why it is that number is the
//! file, and the two are read by different people at different moments.
//!
//! **A version is never restated anywhere else.** REMOTE §3 says so outright:
//! a second home for this number went five versions stale before anybody
//! noticed, so the doc points here and this is the only place it lives.
//!
//! **Two integers live here, not one** (bl-9ced): the version this build
//! SPEAKS and the newest version yog has PUBLISHED. They are the two halves of
//! the `<n> → <m>` heading every entry below already writes in prose, and the
//! second is here rather than derived because the one reader that needs it —
//! the corpus ledger's gate — is an ordinary unit test on a bare checkout.
//!
//! **The file grows by design, and its next seam is an era, not a line
//! count.** A ledger's whole value is that no entry is ever deleted, so §12's
//! pre-split band cannot be answered here by moving prose around: when this
//! file next crosses the cap, the entries at or below a spent version move to
//! a closed `version/<n>.rs` record whole, and the live entries stay.

/// The protocol this build speaks.
///
/// **One integer, and a new verb is not a bump.** A `Query`, an `Action` or a
/// reply kind the peer has not heard of already refuses in band, naming it
/// (REMOTE §3's strict decode) — which is the boundary correcting itself, not
/// two protocols meeting. This changes when the *existing* shape changes
/// meaning: the framing, the envelope, or what a spelling already in use is
/// taken to say.
///
/// 4 → 5 (bl-e654): the `governing` reply's `branch` key — *the lineage whose
/// tip the frozen commit still is* — is gone, replaced by `follows` and
/// `diverged_lineages`, and the `oid` beside them stopped meaning the fork
/// commit and started meaning the commit control resolves from. Same verb,
/// same question, a different thing said: exactly the case above.
///
/// 5 → 6 (bl-23bd): `reply/providers` rows gained `effort` and `priority`, the
/// two per-row tuning capabilities a seat decides a control's existence by. The
/// **two new ops beside them cost nothing** — `/effort` and `/priority` are new
/// spellings in an existing vocabulary, and §3's rule is that a peer which has
/// not heard of one already refuses it in band by name. One bump for the row,
/// and nothing else shape-changing is batched behind it: clients re-vendor per
/// bump, and this number walked 2 → 5 inside one unreleased cycle already.
/// 6 → 7 (bl-8758): every `reply/help` row gained `surface`, the word saying
/// whether a seat-class client owes that op a control (`docs/PARITY.md` §2).
/// The ledger would have let it through — help's signature last moved at 1, so
/// its one free move at 6 was unspent — but §3's rule is the authority and is
/// stricter than the mechanism: any wire-visible shape change, *gained
/// included*, bumps the version, and the ledger cannot see what has shipped
/// (REMOTE §9.9's correction). It is also the bump that pays for itself: the
/// classification is the artifact clients vendor and judge themselves against,
/// so a client must re-vendor to read it, and a bump is exactly what makes it.
/// 7 → 8 (bl-66d4): `reply/advertised` gained `wrote`, the word saying whether
/// this engine WROTE the advertised set or found it identical and compared. It
/// is required rather than optional-absent-reads-false, because absent would
/// read as *"nothing was restored"* — the reassuring answer — on exactly the
/// build that cannot tell, and the field exists to make one event audible.
/// 8 → 9 (bl-015b): `reply/transcript` gained the `wounded` entry — the §8.5
/// settled-failure notice, the third virtual entry — and `reply/steps` LOST
/// `auth_failed`, the §8.3 affordance now being the `refused` arm of the wound
/// vocabulary both shapes spell. A gain and a loss on two shapes, which is
/// two of the four things §3 says bump the version; the ledger's one free move
/// would have covered neither, since it cannot see what has shipped.
/// 9 → 10 (bl-09aa): **no field moved, and that is why this bump is the rule
/// rather than an exception to it.** `attention` became follow-class (REMOTE
/// §14.1): the same ask, the same reply shape, but a *sequence* — the first
/// frame at connect, a further frame whenever the answer changes, the
/// terminator when the hold ends. That is precisely "what a spelling already in
/// use is taken to say", and it is the one class of change the corpus ledger
/// cannot see, since frame count is not a field signature. A seat built against
/// 9 would read the first frame and then wait on a terminator up to a hold
/// away; strict equality here is what turns that into an upgrade sentence.
/// 10 → 11 (bl-4d81): `reply/ops` gained the three readings a §7.3 failure
/// banner is made of — `failed`, `exit_label` and `standing` — so the row
/// answers what it *is* and not only what was logged. A field gained on a shape
/// already in use, which §3's rule bumps outright; and the bump is the point
/// rather than a tax, since the whole gain is a classification that reaches a
/// client only through a re-vendor.
/// 11 → 12 (bl-09ef): every queue row — `reply/attention` and the
/// `reply/acknowledged` remainder that spells rows the same way — gained
/// `says`, the firing rules **in words**. The escalation those words existed
/// for was ruled a **seat's** act (DESIGN §6: a desktop notification belongs on
/// the box the operator is looking at), and `AttentionKind::says` stays this
/// engine's one home for the sentence, so the sentence has to cross rather than
/// be re-worded per seat. A gain on two shapes, which the ledger sees and §3
/// bumps for regardless.
/// 12 → 13 (bl-dc3f): `reply/config` gained `settings` — the file's own schema
/// applied to the text answered beside it (§9.5), so a config destination is
/// read as the typed thing it is and not as bytes a seat must parse. A field
/// gained on a shape in use, which §3's rule bumps outright; **the ledger could
/// not have caught it**, since `reply/config`'s signature had stood at 1 and
/// its one free move would have covered exactly this — REMOTE §9.15's
/// correction, applied again. It does **not** batch onto 12 the way §9.15
/// batched two reads: 12 landed on `main` ahead of this, and whether a release
/// captures it before this lands is a race no reader could resolve later —
/// batching is the exception, and an exception taken against a running release
/// train is how a version comes to mean two things.
/// 13 → 14 (bl-fec6, bl-d542): two additive fields on two shapes in use, which
/// §3's rule bumps for outright. `request/enroll` gained an optional
/// **`address`** — the endpoint the DEVICE being enrolled will dial, which need
/// not be the one this engine wrote for itself (an emulator reaches its host
/// through the emulator's own alias, a phone on the LAN, an overlay peer by
/// name), so the envelope stopped being correct only for a device sharing this
/// box's loopback view. And `reply/clients` gained an optional **`last_seen`**,
/// the unix second a client last spoke: `present` alone could not tell a
/// machine that spoke ten seconds ago from one that never once connected, and
/// on a terminal seat it reads `false` for everything. Both are ABSENT rather
/// than null when unstated — the absence is the fact in each case ("this
/// engine's own address", "never") — and both land at ONE version, because two
/// bumps a minute apart would make every client re-pin twice for one wave.
/// 14 → 15 (bl-5305): `reply/follow` gained `tools`, the **tool window** — an
/// entry as a call is posted (its name, which for a routed call carries the
/// machine, and its bounded input) and another when the capture lands (its exit
/// code). The lane carried the model's prose alone, so an operator watching an
/// agent administer their boxes saw a thinking marker and two sentences while
/// eight commands ran on two machines. A field gained on a shape in use, which
/// §3's rule bumps outright; and the field is **required rather than
/// optional-absent-reads-empty**, on `reply/advertised`'s own precedent at 8 —
/// absent would read as *nothing ran*, the reassuring answer, on exactly the
/// build that cannot tell.
/// 15 → 16 (bl-e59e): `reply/ops` rows gained `client` — the identity that made
/// the act: a connection's certificate common name, or `local` for the window,
/// the `gestures/` inbox, `yog gesture` and yog's own loops. Round-1 ruling 5
/// (*identity is the leaf*) landing on the durable half: §4.1 already makes the
/// leaf's common name the identity, §4 narrows the published derivation by it
/// and §5's roster renders it, and then the trail dropped it at the one place a
/// person later has to reconstruct what happened. It is the one fact on the row
/// that **cannot** be recovered afterwards — presence is a point-in-time
/// observation by design, so nothing later can say who a row belonged to. A
/// field gained on a shape in use, which §3's rule bumps outright.
/// 16 → 17 (bl-ebef, bl-6661): **two shapes, one version**, because both are
/// the same litany 0.0.11 pin landing and two bumps a minute apart would make
/// every client re-pin twice for one wave (14's own reasoning).
///
/// The §6 signal vocabulary gained **`truncated`** — rule
/// 2's rest said in the word that is true of it when the turn was cut off at
/// the request's output cap, the second refinement beside `refused` and
/// standing where `stopped` would, never beside it. litany 0.0.11 makes that
/// cut a named failure (`Error::OutputTruncated`, upstream bl-155f / bl-ecf9):
/// the staging sink is never sealed, so nothing is committed and no tool call
/// runs — while the seat read *"came to rest — your turn"*, the sentence the
/// upstream ball measured against nine `apply_patch` calls arriving as
/// `input: {}`. **No field moved and the ledger cannot see this one**: a
/// signature is field shapes, and this is a new VALUE in `signals`, which a
/// strict decoder built against 16 refuses by name. That is §3's rule reaching
/// past the mechanism for the third time (REMOTE §9.9, §9.15), and it is why
/// the number and not the ledger is the authority. It does **not** batch onto
/// 16: that landed on `main` ahead of this, and whether a release captures it
/// first is a race no reader could resolve later.
///
/// And a delivered row — on `reply/transcript` and on `reply/inbox`'s deposit
/// envelope alike — gained an optional **`sender_name`** / **`from_name`**
/// (bl-6661, litany 0.0.11 upstream bl-a457): the sender's display name, present
/// exactly when the sender is an agent wearing one. The framing sender is the
/// FILENAME's origin token — the addressing key litany's own inbox scan derives
/// from, and always will be — so every message a child sent was attributed by
/// sixty characters of timestamped hex, which on a phone is the whole row
/// header. ABSENT rather than null when there is no name (`user` never wears
/// one, an unnamed agent never does), the absence being the fact, on
/// `reply/enroll`'s `address` precedent at 14. The id keeps riding beside it and
/// is not replaced: it is the durable handle once the agent is deleted and the
/// name recycled.
/// 17 → 18: **the shapes below, one version.** Three lanes raised the wire in
/// one night, and one number carries all of them on 14's and 17's own
/// reasoning said a third time — two bumps a minute apart make every client
/// re-pin twice for one wave — with a second argument this release now has:
/// under bl-bca2's gate a raise HOLDS the release until three consumer mains
/// vendor it, so each extra number is another window in which no published
/// suite composes. **A later lane landing on this version adds its shape to
/// this entry rather than taking 19.**
///
/// **`reply/follow`'s tool-window entry gained `held`** (bl-58bb) —
/// the capability control's reason for parking the call, beside the `tool_use`
/// id and the tool name the entry already carried. The window's two entries
/// come off the pair of files litany lands, and a held invocation is parked
/// *before* the executor is entered, so it lands neither: the one lane an
/// operator has open while a command they did not expect is about to run on
/// their server reported the conversation as at rest and ended the stream, at
/// the exact moment the operator was the thing it was waiting for. On a foot
/// lane every call to a non-shell tool is held, so this was most of the
/// conversation. A field gained on a shape in use, which §3's rule bumps
/// outright; its presence is the status, the discipline `exit_code` already
/// carries on the same entry.
///
/// **`reply/steps` rows gained a fourth `framing` word, `in_flight`**
/// (bl-ab53) — the step being written right now, told apart from the one an
/// interrupt cut. §4.4 `killed` is a tail with no terminal segment, and §2.9
/// says outright that a kill, a crash and a call in progress are
/// indistinguishable there — so watching a working conversation reported
/// `killed` once per step, on the one word that makes the interrupt legible.
/// `meta.json` cannot separate them either: a signalled driver writes none, so
/// an absent `ended_at` is equally both. The agent's §3.5 liveness can, and
/// `steps_view::build` already spends exactly that observation on the §7.3
/// wound, so the judgement crosses already made. Not a new key but a new VALUE,
/// which a shape signature cannot see and a strict decoder built against 17
/// refuses by name — §3's rule reaching past the mechanism again, and why the
/// number and not the ledger is the authority. Refusing the connection is the
/// loud failure that buys: an older seat would otherwise fail on precisely the
/// step somebody is watching.
///
/// **`request/answer` and `reply/answered` gained `scope`** (bl-94a5) — how far
/// one capability answer stands: the held `call` (what every answer was, and
/// still the default), the `conversation` and its descent, or the `workspace`.
/// An answer with no scope settled one call, so an operator holding a
/// conversation answered the same question for every call of a kind they had
/// already decided about — eleven holds and eleven releases of one narrow
/// routed tool in a single measured run. Required in both directions rather
/// than optional-defaults-to-`call`, on `reply/advertised`'s precedent at 8 and
/// `reply/follow`'s at 15: an absent field would let two ends disagree about
/// how wide the instruction was, and *wider* is the reading nobody may arrive
/// at by accident. Two shapes, because the gesture and its receipt are one
/// move.
///
/// **The `prepare` reply's `prepared` body gained `role`** (bl-9ced) — the
/// role the conversation is born on, beside the `lineage` it already carried
/// and read out of the same config commit. litany 0.0.12 (upstream bl-946c)
/// made `litany prompt --role <name>` resolve a root's soul, provider
/// assignment and tool grant as any role the governing commit declares, which
/// is what turns plan mode from *a lineage a workspace must author, forever,
/// to restate a config commit* into one field of a start. yog derives nothing
/// into it: `prepare` answers `null` — litany's `worker`, spelled as an
/// absence exactly as `lineage`'s default is — and a seat that wants a
/// planning conversation deposits the same body back as `/prompt` with
/// `"role": "planner"`. Every shape carrying a `prepared` gains it, which is
/// four; a field gained on a shape in use, so §3's rule bumps. It lands on
/// THIS entry rather than on 19, which is what this version's own heading asks
/// for and what nothing could honour until [`PROTOCOL_PUBLISHED`] below: the
/// corpus ledger refused a moved signature at the version the record was last
/// GENERATED at, a proxy that advances whether or not the bump ever shipped.
pub const PROTOCOL: u32 = 18;

/// **The newest `PROTOCOL` yog has PUBLISHED**, and therefore the newest a
/// peer out there can be speaking (REMOTE §3, §9.11 as amended by bl-9ced).
///
/// It exists for one reader: the corpus ledger's rule that *a wire-visible
/// shape may not change at a version already in use*
/// (`crate::boundary`'s standing record). That rule used to be judged against
/// the version the record was last **generated** at, which is a proxy for this
/// and not the thing itself — so the second lane of one unreleased wave was
/// refused and had to take another integer, and REMOTE §9.11 accepted that
/// cost with the reason *"collapsing it would mean teaching the ledger what
/// has been published, which nothing in this tree knows"*.
///
/// **Two facts have changed since.** Something in this tree does know: this
/// constant, which is the left-hand side of the `<n> → <m>` heading every
/// entry above already writes in prose, now written once where a program can
/// read it. And an extra integer is no longer cheap — under bl-bca2's release
/// gate a raise HOLDS the release until three consumer repositories vendor the
/// number on their mains, so a wave that costs three numbers costs nine
/// consumer edits and three windows in which no published suite composes.
///
/// **How it moves.** A lane that raises `PROTOCOL` leaves this alone unless
/// the number it is raising FROM has shipped; then it raises this to that
/// number in the same edit. So the pair reads exactly as the heading does —
/// `17 → 18` is `PROTOCOL_PUBLISHED = 17`, `PROTOCOL = 18` — and every later
/// lane of the same wave shares 18 by leaving both alone.
///
/// **What it does not do.** It is not a compatibility window and the handshake
/// never reads it: the wire is still fail-closed on `PROTOCOL` alone, with no
/// negotiation. And it is a stated fact rather than a derived one — the
/// derivation exists (`scripts/protocol-gate.sh read` over yog's newest
/// `v<x.y.z>` tag is what `.github/workflows/release-automerge.yml` already
/// does), but it needs git and a network, and the ledger's gate is an ordinary
/// unit test that must run on a bare checkout. The residual is that a lane
/// which raises `PROTOCOL` after a release and forgets to raise this one is
/// under-strict for one wave; that is strictly better than the standing state,
/// where the answer to every shared wave was *take another integer*, and the
/// pair is read together in one file so the forgetting is visible where the
/// raise is made.
pub const PROTOCOL_PUBLISHED: u32 = 17;
