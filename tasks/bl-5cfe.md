+++
title = "/seen is a write whose help reads as a read: it clears the badge for the selected conversation and its receipt is the queue without it"
created = 1788235239
updated = 1788235239
priority = 4
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "language"]
+++
## Symptom

`/seen` is a **write** whose one-line help reads as a read:

```
/seen
    answer the selected conversation's place in the attention queue
```

It is `Action::MarkSeen { workspace, agent }` — it stamps the `ui.json`
watermark for the selected conversation, which removes it from the attention
queue and drops the workspace's attention count. The reply then carries the
queue *as it now stands*, so the change is invisible in the answer: the
conversation you asked about is simply not in the rows.

Driven live: three conversations at rest, three in the queue. One `/seen`
aimed at one of them, and the reply lists the other two — no marker of which
one was acted on, no statement that anything was written, and the workspace
count silently goes 3 → 2.

## Why the sentence misleads

"Answer" is being used in the sense *respond to / discharge*, and every other
verb in the same list uses plain imperatives for acts (`kill`, `send`, `close`,
`claim`, `truncate`). Beside them, "answer … the place in the attention queue"
reads as *tell me where it stands* — a `/attention` variant scoped to one
conversation. An operator reaching for a scoped read gets a mutation, and the
badge they were about to look at is gone.

The line verb's own doc has the true sentence already:

> `/seen` — the §6 queue's answer. The seat's own selection is the item, exactly
> as it is for `/message`: **answering and acknowledging aim alike**, so the line
> names neither and the context supplies both.

*Acknowledging* is the word the help is missing.

## Repro

    yog gesture '/attention'                      # N rows
    yog gesture --ws <ws> --agent <conv> '/seen'  # replies with N-1 rows
    yog gesture --ws <ws> '/conversations'        # that conversation's attention: 1 → 0

## Shape of the fix

The help line says what it does — acknowledge the selected conversation and
answer the queue that remains. Optionally the reply names the item it marked,
the way `Reply::Answered` deliberately does ("It answers with the *held
invocation* rather than the queue that remains … a receipt that lied"); the
same argument applies here one verb over, and `/seen` is the precedent that
doc cites.