+++
title = "scrub balls/tasks and publish it: content redaction across the ref's history, not a private remote"
created = 1786677230
updated = 1786678339
claimant = "Ciabatta"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["publication"]
+++
OPERATOR RULING 2026-08-13, superseding the audit's remedy: do NOT move the
store to a private remote. Scrub `balls/tasks` of anything that should not be
public, then publish it alongside the source.

Companion ruling: **the `mudbungie` identity is intentionally public** — the
handle and `mudbungie@gmail.com` stay, in `LICENSE`, `Cargo.toml`, README, the
`release-plz.yml` `repository_owner` guard, task bodies, and commit metadata.
The leak scanner's existing `personal-email` EXCEPT for that address is
correct and stays. **Every OTHER identity must go**: third parties, other
operators, and any other address.

## What "scrub" has to reach

Editing the current task files is not the job. The scanned ref carries ~1,500
commits and 400-odd task paths; a closed ball has no file, so its record lives
only in history, and the worst material the audit found is in DELETED blobs
that no `bl update` can reach.

The classes, not the quotations (naming them here would re-commit them):

- an operator's live credential state and the model roster it returned;
- an operator's home paths, private workspace names, conversation and agent
  ids, wire request paths, and quoted user/model turns;
- an operator report, machine load, branch lineage and object ids from a
  live incident;
- verbatim operator dialogue quoted as a design ruling;
- account-level billing and payment text from a CI annotation;
- provider auth state ("signed in", "no credential stored") and the host
  config paths behind it;
- live process ids alongside the commands they ran.

At the audited ref, aggregates rather than instances: ~34 task paths carried an
absolute operator home path, ~105 carried operator report/ruling markers, ~40
referenced session artifacts, ~22 carried live-auth/provider markers. **No
vendor token, private key, JWT, credential value, auth URL, routable IP, MAC,
machine id, hardware serial or host credential was recovered** — the exposure
is private context, not a proven secret. Store commit authorship is only
`mudbungie` (public, stays) and the `balls` bot, so no author-metadata rewrite
is needed.

So publishing the ref means **rewriting its history**, not editing its tip.
That is destructive: it invalidates every existing clone of the store,
including other agents' working clones under
`$XDG_STATE_HOME/balls/clones/*/tasks`.

## Required work, in order

1. **Bundle the ref first** (`git bundle create` of `balls/tasks`, stored
   outside any repo) — the pre-scrub record must survive somewhere private.
2. **Scrub live task bodies** with ordinary `bl update --body` / `bl comment`.
   Keep the reasoning; drop the identity, the chronology and the machine
   state — the same editorial rule bl-2368 applied to the source tree.
3. **Rewrite history** so deleted blobs carry the same scrub. Content-based
   redaction (`git filter-repo --replace-text` plus `--replace-message`) over
   the whole ref, not a path drop — the paths are the record.
4. **Re-scan the rewritten ref** with the leak gate's rule table, over every
   blob in the rewritten history rather than the tip, before any push.
5. **Then, and only with a fresh operator go-ahead, force-push.** Everyone with
   a clone must re-clone; `bl prime` will not reconcile a rewritten ref for
   them.

## The part a one-time scrub does not solve

The task log keeps growing, and nothing scans it. `scripts/leak-scan.sh`
(bl-167d) scans the yog worktree's index, not the store. A published task store
needs a standing guard: run the same rule table over `tasks/` and refuse a
store commit that trips it, plus a stated rule in AGENTS.md for what may never
enter a ball body — other people's names, transcript prose, machine state,
provider auth state. Without that, the store re-leaks within a week and this
ball is a treadmill.

Gates `bl-4f96`.