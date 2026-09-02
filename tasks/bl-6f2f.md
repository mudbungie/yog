+++
title = "/flag raises no attention item: it writes one exit-0 ops row and the §6 predicate reads no ops row, so the monitor's floor grant signals where nobody looks"
created = 1788235207
updated = 1788318328
claimant = "Signal"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "attention"]
+++
## Symptom

`/flag <why…>` answers `{"kind":"flagged","ok":true}` and raises nothing an
operator can see. Driven live against an engine:

```
$ yog gesture --ws <ws> --agent <conv> '/flag please look at this one'
{"kind":"flagged","ok":true}

$ yog gesture '/attention'          # the conversation is not in the queue
$ yog gesture --ws <ws> '/conversations'
… "attention": 0 …                  # unchanged, before and after
```

Repeated on a conversation that WAS already in the queue: its `signals` list is
unchanged too. The only trace is one ops row:

```
{"argv":"yog-flag <agent>","exit":0,"origin":"conversation",
 "stdout":"please look at this one", …}
```

`exit` is 0, so it is not an alarm either — `/ack` ("acknowledge every alarm on
the ops trail") has nothing to acknowledge. A flag is visible only by scrolling
`/ops` and reading `argv` prefixes.

## The contract says otherwise, in three places

- The gesture's own help: *"raise an attention item on the selected
  conversation, with a reason"*.
- `Reply::Flagged`'s doc: *"An attention item was raised on a conversation
  (VISION §4.9)"*.
- VISION §4.9's ladder table:

  | Flag | attention item + ops row; a boundary variant, so it is also grantable as a responder tool | … |

  and, in the prose beside it: *"signaling out is itself a tool call — a
  boundary action writing the ops row **attention already derives from**"*.

## Why it cannot fire

`boundary::monitor::flag` is three lines — build the row, append it, answer
`Flagged`. Its own comment says so: *"Raise an attention item on one
conversation: one row, nothing else."*

`attention::attention` (DESIGN §6) fires on six signals — notify, stopped,
budget, conflicted, mail, held — and every one of them is read off a
`refs/litany/*` mark or the inbox listing (`git_tree::marks`). **None of them
reads the ops trail.** So the row a flag writes is not an input to the
predicate that VISION says derives from it. The two halves were never joined.

## Why it matters beyond the verb

`flag` is the alignment monitor's **floor grant** (VISION §4.9, bl-7aef): a
responder granted only `flag` is "a pure judge", and the monitor ships
flag-only by design so that a false verdict acts visibly rather than
destructively. A monitor wired that way today signals into a place the operator
never looks. The verb that exists so the machine can raise its hand does not
raise it.

## Repro

    yog gesture --ws <ws> --agent <conv> '/flag anything'
    yog gesture '/attention'        # <conv> absent, or its signals unchanged
    yog gesture '/ops 4'            # the yog-flag row, exit 0

## Shape of the fix — a ruling first

Either the predicate grows a seventh signal reading the flag rows (with a
watermark, like signals 1–4, so `/seen` can clear it), or VISION §4.9's "+
attention item" and the two help sentences are wrong and should say the trail
is where a flag lands. The former is what the design says and what the monitor
needs; the latter is what the code does.