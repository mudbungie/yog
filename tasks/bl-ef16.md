+++
title = "RECURRENCE of bl-22ab: /search rows address workspaces and projects by engine path, so a hit cannot be fed to the gesture whose keys it spells"
created = 1788235214
updated = 1788412328
claimant = "Spellbind-E"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "wire", "addressing"]
+++
## Symptom — the same defect bl-22ab closed, one reply over

`/search` addresses a workspace by the **engine's absolute path** while every
other reply and every gesture addresses it by its name:

```
$ yog gesture '/search poem'
{"kind":"search","ok":true,"rows":[
  {"at":"conversation",
   "workspace":"<engine-data-root>/yog/workspaces/<name>",
   "agent":"<agent-id>","field":"summary","excerpt":"…"}]}

$ yog gesture '/attention'
{"kind":"attention","ok":true,"rows":[{"workspace":"<name>", …}]}
```

Feed the search row's `workspace` back into the act it addresses and it
refuses: `unknown workspace "<engine-data-root>/yog/workspaces/<name>"`. Ball
hits carry `project` the same way.

## It is exactly the invariant bl-f5f6 landed and bl-22ab re-fixed

REMOTE §8:

> **Paths never cross the wire.** *(Landed, bl-f5f6.)* Boundary types addressed
> workspaces and projects by absolute `PathBuf`; across machines those are
> meaningless and a disclosure besides. The wire spelling is now the **name**…

and its identify/locate rule:

> **A reply speaks the name where it IDENTIFIES and the path where the path IS
> the answer.**

A search hit is pure identification — the module's own header says the subject
is *"something you can already select — a ball, a workspace, a conversation"*.
So it is the name, by the rule, not by preference.

bl-22ab (closed) found the same shape at `/attention` and closed with an
explicit instruction that this reply was inside:

> Audit adjacent wire replies that serialize `cwd`, workspace or project paths
> against the same landed invariant.

`/attention` was fixed. `/search` was not, and REMOTE §8.1 — "The path-typed
reply residuals, closed" — enumerates four fields, none of them this one.

## Where it lives

`src/boundary/reply/search.rs`, whose own doc-comment states the intent the
code then breaks:

> One hit as data: the address it names, spread flat so a consumer reads the
> same `project`/`id`/`workspace`/`agent` keys the gestures take.

The keys are right; the values are `path_text(…)` of a `PathBuf`, because
`search::Address` is `PathBuf`-backed exactly as `QueueRow` was.

## Two costs, both bl-22ab's

- **Broken teleoperation.** The search reply's whole purpose is to hand back
  something selectable, and what it hands back is refused by every act that
  takes a workspace.
- **Disclosure.** A remote seat is told the engine's filesystem layout for
  every hit.

## Repro

    yog gesture '/search <needle>'      # rows carry engine-absolute workspace/project
    # copy a row's workspace into any --ws gesture → unknown workspace

## Note on scope

`decode` (`hit_of`) reads the same keys as `PathBuf`, so the round-trip test
moves with the encoder — this is a protocol change under REMOTE §3, not a
one-line swap, which is why it is filed rather than patched in passing.

**Adjacent, unaudited, same class:** `/ops` rows carry `cwd` and an `argv`
string full of engine-absolute paths, and `Prepared::binding` (the `/prepare`
reply) is an absolute worktree path. Those may be "the path IS the answer"
fields under §8.1's rule — but bl-22ab asked for the audit and no ball records
one having been done.

---

## Verification: the headline premise is dead, the journal note is not

`search::Address` has spelled the §3.1 workspace leaf and the §5.1 #1 project
name since bl-764a — `reply/search.rs`'s own header says so, `hit_of` reads
strings, and `path_text` no longer exists anywhere in the tree. REMOTE §8.1
already records that audit and already named what remained. So the ball's
symptom, its `Where it lives` section and its `Note on scope` are all stale;
what survived verification is the journal note (two refusal strings) and
bl-22ab's standing ask for the sweep.

## The sweep bl-22ab asked for — every wire reply field that serializes a path

The §8.1 question of each: does this IDENTIFY something the asker can name, or
is the path itself the answer?

**Identify — leaks, fixed in this ball:**

- `fleet::Facts::workspace` / `::project` (`Reply::Board`'s `fleet` list) —
  engine-absolute paths under the keys `workspace` and `project`, the two words
  every gesture and `--project` take, one array away from a `BoardRow` spelling
  the same two facts as names. This is the bl-22ab shape, not the disclosure
  §8.1 had it filed as. Now the leaf and the wire name; the §4.3 pilot still
  needs the directory and resolves it at one seam, `Snapshot::armed_path`, over
  the `cadence.yaml` arming table's own keys rather than the §3.1 enumeration
  (an entry arms a directory verbatim, so a loop may be armed on a workspace the
  enumeration has not reached, and refusing it there would stop it planning).
- `control::answer_hold`'s `nothing is held on X in <ws path>` and
  `answer::queue::mark_seen`'s `no conversation X in <ws path>`. Refusal strings
  are reply bodies and had never been swept, because the residual list only ever
  asked about fields. Both say the name now — the token the seat addressed the
  gesture by.

**Locate — the path IS the answer, all kept:**

- `Prepared::binding` — already ruled in §8.1; litany's `--cwd`.
- `Reply::Files`'s `working_dir` — the §3.3 cwd mark, whose whole job is saying
  the work went where this listing does not reach and where that is.
- `OpRow::cwd` — the subject is where a command ran, the case §8 exempts.
- `OpRow::argv` — the same fact one field over: an argv rewritten into names is
  not the argv that ran.
- the drift finding `app::drift` writes into a row's `stderr` — its subject is
  the root a derivation failed over.
- `dispatch::enroll`'s refusals (`run … where the CA lives`, `remove it by
  hand`) — the remaining act is the operator's own, on that exact file, and
  enrolment is operator-grade.
- `Held::reason` (`moves to <dest>`) — the drone's own tool input under
  adjudication; rewriting it would put a different call in front of the operator.

**Not residuals:** `FileEntry::path` and `WorkDiff`'s per-file `path` are
repo-relative; `Lineage::files` / `GoverningConfig::files` are tree paths inside
a config commit; `Payload::Path { dir }` rides a gesture, where the operator
typed the directory; `Workspaces::stale` and `growth` carry an age and a §3.3
conversation name.

No protocol bump: the ledger signature is string-to-string on both changed
fields, and the bl-22ab reading holds — the fields always meant the address.
The verdicts are written into REMOTE §8.1's residual list, which now closes.
