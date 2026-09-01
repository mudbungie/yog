+++
title = "RECURRENCE of bl-22ab: /search rows address workspaces and projects by engine path, so a hit cannot be fed to the gesture whose keys it spells"
created = 1788235214
updated = 1788235397
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