+++
title = "tool roster vs the prompt cache: advertisement becomes durable, the conversation freezes its roster at the root, absence answers at invocation — REMOTE.md §5 rework"
created = 1786683026
updated = 1786683026
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator objection (2026-08-13) to REMOTE.md §5 as landed by bl-b9a2: advertisements were ruled connection-scoped RAM, but tool definitions live in the model's stable context prefix, and the prompt cache is keyed on that prefix — a connectivity flap would rewrite the tool block and trash the cache of every live conversation in the workspace, besides mutating the roster under the model mid-conversation.

Resolution to write into §5 (+ prune the settled half of the §10 open question):
- Split the fact: ADVERTISEMENT (which tools a client offers) becomes durable in the registration, updated on connect when it differs; PRESENCE (connected right now) stays RAM. They change at different rates — tool sets when the operator reconfigures a machine, presence on every blip.
- The conversation freezes its remote-tool roster at the root, composed from registered advertisements; presence never enters the prefix. Consistent with the frozen-config direction (bl-aa8b, bl-a0d4).
- Absence and staleness answer at invocation, in-band: routing to an absent client, or a client refusing a tool it no longer carries, returns an error tool result — an appended message, never a prefix change. Deadline unchanged.
- A live conversation adopts a changed roster only by explicit gesture, knowingly paying one prefix rebuild; new conversations compose from current registrations.
- The freeze point is context composition, which is lernie's — folds into the §5 lernie-seam ask.