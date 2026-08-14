+++
title = "publication gate: move the balls/tasks operations diary off the publication remote onto a private one"
created = 1786677230
updated = 1786677230
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["publication"]
+++
Source: publication audit follow-up 2026-08-13 (item 1), snapshot yog
`e758814`, remote `balls/tasks` scanned at `ed1fd13`.

`balls/tasks` rides the same GitHub object store yog would publish from, and it
is an internal operations diary. The scanned ref had 1,461 commits, 417 task
paths in history, 34 current task files. Deleted task files remain recoverable
from the ref's history.

Current records, verbatim:

- `tasks/bl-b7dc.md`: "[redacted: account-level scheduling annotation]" (account-level
  trouble)
- `tasks/bl-9b52.md`: "the operator's own live world does exactly this", naming
  `~/.local/share/yog/world/lernie/template/providers.yaml`, `openai-chatgpt`,
  `~/.config/brazen/config.toml`.
- `tasks/bl-20cb.md`: "openai-chatgpt auth oauth2 · <state>" and
  "anthropic auth api_key · <state>" (provider/auth state, not a
  credential value).
- `tasks/bl-a0d4.md`: live process IDs, provider/login commands, model changes.

Deleted-but-recoverable:

- `tasks/bl-5426.md` quotes the operator on the live Codex account: "Run against a live credential it returns seven", followed by the model inventory.
- `tasks/bl-bd9d.md`: `/home/u/...`, private workspace names, conversation
  IDs, request paths, user/model messages, tool grants.
- `tasks/bl-ebbd.md`: an operator report, an agent chat name, the branch-storm
  incident, machine load, paths, lineage, OIDs.
- `tasks/bl-4fb1.md`: "I don't have any useful work in yog".

Aggregate: 34 task paths contain `/home/u`; 105 carry operator
report/ruling markers; 40 reference session artifacts; 22 carry
live-auth/provider markers. Commit metadata carries `mudbungie@gmail.com`.

No vendor token, private key, JWT, credential value, auth URL, routable IP,
MAC, machine id, hardware serial, or host credential was recovered from task
blobs or commit messages. The exposure is private context and reputation, not a
proven secret leak.

Manual redaction is the wrong boundary: the log changes continuously and
deleted records stay in history.

Required: move the task store to a private remote (rebind the balls tracker's
remote for this checkout), then remove `balls/tasks` from the publication
remote. A sanitized new repository is safer than flipping the existing GitHub
object store public.

OPERATOR DECISION REQUIRED before any remote is touched: which private remote
hosts the store, and whether publication is a new repo or a flip of the
existing one. Nothing outward-facing may be executed on this ball without that
ruling.