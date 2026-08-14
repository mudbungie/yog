+++
title = "scrub balls/tasks and publish it: content redaction across the ref's history, not a private remote"
created = 1786677230
updated = 1786678227
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

Editing the 34 current task files is not the job. The scanned ref (`ed1fd13`)
carries 1,461 commits and 417 task paths; closed balls have no file, so the
record lives only in history, and the worst material named in the audit is in
deleted blobs:

- `tasks/bl-5426.md` — "Run against a live credential it returns seven",
  plus the model inventory.
- `tasks/bl-bd9d.md` — `/home/u/...`, private workspace names, conversation
  ids, request paths, user/model messages, tool grants.
- `tasks/bl-ebbd.md` — an operator report, an agent chat name, the branch-storm
  incident, machine load, paths, lineage, OIDs.
- `tasks/bl-4fb1.md` — "I don't have any useful work in yog".

Still live at the time of the audit:

- `tasks/bl-b7dc.md` — "[redacted: account-level scheduling annotation]"
- `tasks/bl-9b52.md` — "the operator's own live world does exactly this", naming
  `~/.local/share/yog/world/lernie/template/providers.yaml`, `openai-chatgpt`,
  `~/.config/brazen/config.toml`.
- `tasks/bl-20cb.md` — "openai-chatgpt auth oauth2 · <state>",
  "anthropic auth api_key · <state>".
- `tasks/bl-a0d4.md` — live process ids, provider/login commands, model changes.

Aggregate at `ed1fd13`: 34 task paths contain `/home/u`, 105 carry operator
report/ruling markers, 40 reference session artifacts, 22 carry
live-auth/provider markers. No vendor token, private key, JWT, credential
value, auth URL, routable IP, MAC, machine id, hardware serial or host
credential was recovered — the exposure is private context, not a proven
secret.

So publishing the ref means **rewriting its history**, not editing its tip.
That is destructive: it invalidates every existing clone of the store,
including other agents' working clones under
`$XDG_STATE_HOME/balls/clones/*/tasks`.

## Required work, in order

1. **Bundle the ref first** (`git bundle create` of `balls/tasks`, stored
   outside the repo) — the pre-scrub record must survive somewhere private.
2. **Scrub live task bodies** with ordinary `bl update --body` / `bl comment`:
   account and payment text, other operators' and third parties' names and addresses,
   `~/.config` and `~/.local/share` operator paths, provider auth state, live
   process ids, conversation ids, and verbatim operator dialogue. Keep the
   reasoning; drop the identity, the chronology and the machine state — the
   same editorial rule bl-2368 applied to the source tree.
3. **Rewrite history** so deleted blobs carry the same scrub. Content-based
   redaction (`git filter-repo --replace-text`) over the whole ref, not a
   path drop — the paths are the record.
4. **Re-scan the rewritten ref** with the leak gate's rule table before any
   push.
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