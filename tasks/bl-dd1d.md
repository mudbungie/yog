+++
title = "scrub balls/tasks and publish it: content redaction across the ref's history, not a private remote"
created = 1786677230
updated = 1786678489
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

---

## Steps 1-4 done (Ciabatta). STOPPED before step 5 — the force-push awaits a fresh operator go-ahead. No remote was written.

**1. Bundles (private, outside every repo, mode 0600).** Both verify "complete history".

- `<operator-home>/balls-tasks-prescrub-20260813-7fda4b0.bundle` — 1,196,320 B, tip `7fda4b0`, 1,493 commits. The ref as the audit found it.
- `<operator-home>/balls-tasks-prerewrite-20260813-45828f5.bundle` — 1,233,415 B, tip `45828f5`, 1,510 commits. The exact ref the force-push would replace, so the rewrite is reversible. Re-bundle immediately before the push if the store has moved again.

**2. Live bodies scrubbed** (`bl update --body`, body edits only — nothing closed, unclaimed or retitled). Thirteen balls:

- absolute operator home paths, rewritten to the house synthetic root `/home/u`: bl-0e44, bl-20cb, bl-3aa1, bl-3f70, bl-648a, bl-71fc, bl-9b52, bl-fb1c
- account/payment text from a CI annotation: bl-b7dc
- provider auth state ("signed in" / "no credential stored" per row): bl-20cb
- live process ids and wall-clock chronology: bl-a0d4
- an operator's own machine configuration, named as theirs: bl-9b52
- verbatim operator dialogue, paraphrased to its substance: bl-a33d
- the maintainer's given name: bl-4f96
- this ball, whose evidence list quoted the material it exists to remove: classes now named, instances dropped

**3. History rewritten** — `git filter-repo` (fb3de42e), `--replace-text` + `--replace-message` over the whole ref, `--prune-empty never`. No path was dropped: 426 task paths before and after, 1,510 commits before and after, 1,190 blobs before and after. 39 expressions covering: the maintainer's given name (with lookarounds so the Rust type `Mark` survives untouched), a third party's name, a second address of the maintainer's, absolute home paths on both platform shapes, account/payment text, provider auth state, live process ids, conversation/agent ids, five private workspace names, one operator remark disclosing a live account's model roster, the `bl-actor: mark` commit trailer, and the project's SSH remote URL (which the `personal-email` rule reads as an address).

`mudbungie` and `mudbungie@gmail.com` were left everywhere by ruling: 978 of 1,510 commits still carry them as author, and the history still carries them in 24 blobs.

**4. Re-scan of the rewritten ref** — `scripts/leak-scan.sh` with `scripts/leak-rules.sh` at yog `58ddd17`, run over all 1,190 blobs of the rewritten history (not just the tip). **Zero findings.** The same scan over the 1,173 pre-scrub blobs produced 135. Every one of the 1,189 blobs with frontmatter still parses as TOML.

Deliberate residuals, all reviewed: `mudbungie@gmail.com` (allowed by the rule's own EXCEPT, by ruling); the occurrences of `Mark` that are the Rust type `watch::Mark`; the agent codename `mark-placer`; the house synthetic home roots, which the rule table exempts by name.

**5. NOT DONE, awaiting go-ahead.** The rewritten ref sits in a throwaway clone under this session's scratch directory, never in the live store; its path was reported to the operator out of band. The command is a single `git push --force` of `refs/heads/balls/tasks` from that clone to the project's own remote, spelled out in full in the handoff report. Re-run the filter first if the remote has moved off the base it was built on — the store is live and every commit since would otherwise be lost. Everyone with a clone must delete `$XDG_STATE_HOME/balls/clones/*/tasks` and re-prime; `bl prime` will not reconcile a rewritten ref.

**Open questions the ruling does not settle** — three, all left as-is:

1. **bl-b7dc's title** still carried the account-billing phrasing; a retitle was out of scope for this pass, so the content redaction reached it instead. The rewritten ref's title reads "blocked upstream at the account level".
2. **The maintainer's own design dialogue** (the logo balls' quoted specs and rulings) was left as prose with the attribution genericized. It is the maintainer's own direction for a published artifact, not third-party content — but it is still verbatim dialogue, which the ruling says goes.
3. **Agent codenames** (`Pizzeria`, `Fretwork`, ...) and **provider row names** (`openai-chatgpt`, `claude-session-direct`) were left. Neither is a person; the codenames are synthetic and the row names are vendor products yog's own source references. One sentence in closed ball bl-bc06 still implies which of two rows held a credential, with the state value itself redacted.


---

The store is live, so any base sha named here is stale the moment another agent commits. The rewrite is therefore scripted, not a one-off artifact: re-run the filter (and re-bundle) against whatever the remote tip is at the moment of the push, and compare commit counts, blob counts and the 426 task paths across the rewrite as the acceptance check. Two rounds of self-scrub were needed first: this result note itself carried an operator home path, a scratch path, the literal rule strings it was describing, and an SSH remote URL — evidence for the standing guard this ball's last section asks for. A store that is only scrubbed by hand re-leaks in the act of recording the scrub.