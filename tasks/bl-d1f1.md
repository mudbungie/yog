+++
title = "a lost boundary reply has no safe recovery: re-deposit can repeat a completed non-idempotent action"
created = 1787206349
updated = 1787275393
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design", "boundary", "security"]
+++
`boundary/deposit.rs` describes crash recovery this way:

> “A crash between claim and reply leaves the claimed file as debris naming exactly what was in flight — re-deposit to re-run.”

`README.md` also says exit 124 means no consumer answered while the deposit remains and may converge later.

That is an ambiguous outcome for non-idempotent actions. Create, update with a journal note, message, prompt and routed execution can land their effect before the reply is lost. Re-depositing the same request may perform it twice; waiting may also perform it later. The caller has no claimed-gesture query, terminal-status query, resume/abandon verb, or downstream idempotency key with which to distinguish pending from committed.

## Required design result

Make one recovery rule safe for every action. Likely ingredients are a stable request identity and a durable terminal receipt that a retry can read rather than re-execute, but the design should attack simpler alternatives first. Specify and drive the crash windows: before claim, after claim/before effect, after effect/before reply, and after reply.

---

## Design analysis — parked for a ruling, not implemented

Both quoted premises verified live at HEAD. `src/boundary/deposit.rs`'s module
doc still says "A crash between claim and reply leaves the claimed file as
debris naming exactly what was in flight — re-deposit to re-run", and README's
exit table still says "`124` no consumer answered (the deposit remains and
converges later)". The ball's four negatives are all true: no claimed-gesture
query, no terminal-status query, no resume/abandon verb, no idempotency key
anywhere on the boundary.

### The four windows, as the code actually behaves

Read from `boundary/deposit.rs`, `boundary/consume.rs`, `boundary/sugar.rs`,
`boundary/consumer.rs`.

1. **Before claim.** The deposit sits in the inbox; the next consumer pass on
   any yog over that world takes it. Exactly-once holds; it converges later,
   which is what README says.
2. **After claim, before effect.** `claim` renamed `<id>.json` into `claimed/`,
   and `pending()` lists only the inbox — so nothing ever re-takes a claimed
   file. The effect never ran, and no artifact anywhere says so.
3. **After effect, before reply.** The on-disk state is *identical* to window 2
   and indistinguishable from it. This is the unsafe window.
4. **After reply.** `read_reply` returns it. Safe.

### Two amplifiers the ball did not name

- **The CLI mints a fresh id per invocation** (`sugar::run` → `deposit::mint`),
  so the operator's natural recovery after a 124 — re-run the command — is a
  *second gesture*, not a retry of the first, and the first deposit is still
  pending, so both may run. Re-depositing under the *same* id also succeeds and
  re-runs, because `deposit()` only refuses an id still sitting in the inbox and
  the claim moved it out. "Re-deposit to re-run" is therefore executable advice,
  which is what makes it dangerous rather than merely stale.
- **The wire path has no recovery artifact at all.** `ConsumerCtx::answer_as`
  runs a decoded gesture straight through the chokepoints — no id, no claim
  file, no reply slot. Since the client/server split the wire is the primary
  remote seat, so the deposit bus's identity is absent from exactly the path
  this defect most affects: a connection that dies after the act is
  unobservable except through `ops.jsonl`.

### What is already ruled

REMOTE §9.8 answer 1: "a gesture is not idempotent — two clicks of Nudge are two
nudges — and a resend is never free, so nothing about an act's own bytes can be
its handle." §9.8 answer 2, on the acts that paint nothing: "their durable
record is the `ops.jsonl` line the §7.3 banner reads back (INV-2)."

So the recorded intent is **at-most-once, with the world as the record** — read
the trail, never blind-retry. `deposit.rs`'s sentence instructs the opposite and
contradicts a landed ruling; it is the one line here that needs no ruling to
fix.

### Alternatives, cheapest first

- **A. Doc-truth only.** State the contract: at-most-once; a lost reply is *in
  doubt*; the recovery is to read the world, not to re-deposit. Invents nothing
  and deletes an actively unsafe instruction. It does not give the caller a way
  to *tell* — it stops telling them a wrong one.
- **B. Key the ops row by the gesture id.** A row per attempted action already
  exists. But `actions::verbs` writes it *after* the verb returns ("runs
  synchronously … then appends its completed outcome"), so this narrows window 3
  to effect-landed/row-unwritten rather than closing it — and it does nothing
  for the wire, which has no id to key on.
- **C. A gesture-status query.** Turns the three on-disk states into an answer a
  remote seat can read. Still cannot separate window 2 from window 3, so its
  honest answer for both is "in doubt" — worth saying, but a new verb to say it.
- **D. A pre-effect stamp.** The consumer marks the claimed gesture immediately
  before running it. The only shape that separates "never ran" from "may have
  run", at one extra write per gesture. Still not definite: the crash can land
  between the stamp and the effect.
- **E. Idempotency key plus a durable committed-receipt table** (the ball's own
  suggestion — real exactly-once). This adds durable state whose single-source
  home is unclear: the effect belongs to `bl`, to `lernie` or to a routed
  executor, never to yog, so yog's receipt can only assert *it dispatched*, not
  *the effect committed*. It is also a second record of what `ops.jsonl` already
  records. Attack it the way REMOTE §8.1 attacks `Prepared::binding`: a
  mint→resolve table is durable state for a fact that is otherwise computed.

### Recommendation, for the ruling

**A + D as the shipped contract; refuse E.** Say at-most-once outright, replace
the "re-deposit to re-run" line and README's 124 note, and add the one stamp
that makes "never ran" a *definite* answer so the in-doubt set shrinks to what
genuinely is in doubt. Whether C's verb is worth minting — and whether anything
can be offered on the wire path, which today has neither an id nor a slot — is
what this ball is parked on.

**Exactly-once is not achievable by yog alone**: the effect is a subprocess's,
and the only thing that can make a retry safe is idempotency in `bl`/`lernie`
themselves. A design that promises it at the boundary would be promising
something the boundary cannot enforce.

### Drivability

Windows 1–4 are drivable on the deposit bus with no new mechanism (claim then
stop; claim, run the effect, then stop), and `boundary/consume/tests` already
has the fixture shape. The wire path's window is not drivable without a way to
drop a connection mid-answer, which is itself a finding: the path with the worst
recovery story is the one with no test seam for it.
