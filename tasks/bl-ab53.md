+++
title = "an in-flight step reads 'killed' in lernie steps, the one word an interrupt writes: watching a live conversation shows a false interrupt on every step"
created = 1788745977
updated = 1788752381
claimant = "Cantaloups-Y9"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r2"]
+++
Round 2, lane swdev. yog main 5cd95986, litany =0.0.10.

## The gesture

Watch `lernie steps <ws> <agent>` while a step is in flight:

    005  killed  10480 tokens

Wait for the step to land and ask again:

    005  complete  1 attempt  21545 tokens  commit c307adfd…

Seen on four separate steps across two conversations, every time. The step
that a real `lernie interrupt` cut also reads:

    005  killed  10480 tokens

and stays that way. So the two are indistinguishable while one of them is
live, and the operator watching a working conversation is told, once per
step, that something killed it.

## Cause

`steps_view::summarize` derives the framing from `response.json`. A step in
flight has a partial (or empty) response file, and the framing walk classifies
"no terminal segment" as killed — the same classification an actually-killed
step earns. Nothing distinguishes "no ending yet" from "ended by a signal".
There is a fact that does: `meta.json` has no `ended_at` until the step lands.

## Why it matters

`killed` is the word that makes the interrupt legible — round 1 called the
interrupt the suites standout capability precisely because the cut step shows
in the spine rather than vanishing. A word that also fires on every ordinary
step in progress stops meaning anything, and the first time an operator sees
it they will go looking for what went wrong.

## Expected

A step with no `ended_at` reads as in flight — its own word — and `killed`
means the signal.

## Severity

p3: cosmetic while it is happening, but it degrades the vocabulary the
interrupt depends on.

---

PROTOCOL is bumped by this ball, 17 -> 18, and main was read first: 17 is
bl-ebef/bl-6661's landing tonight, so the entry sits under it rather than
replacing it. The bump is what REMOTE section 3 calls a four-repository act.

Why a frame had to change. The seat renders one word per step, off the `steps`
reply's `framing` token. There is no other carrier: `started_at`/`ended_at`
already cross and cannot separate the two shapes (a signalled driver writes no
`meta.json` either), and a seat combining the step row with the conversation's
badge would be a seat holding a rule the engine is supposed to have spent.

The bump is a value widening, not a new key, which is exactly the class the
17 entry above records: the corpus signature cannot see it, and a strict
decoder built against 17 refuses the fourth word by name. Refusing the
connection is the loud failure; the alternative is a seat that fails on
precisely the step somebody is watching.

What it obliges, per REMOTE section 3: landing 18 on yog's main is held by
neither gate, but yog's next RELEASE is held until `mudbungie/thrall`,
`mudbungie/lernie` and `mudbungie/yog-android` carry 18 on their mains — and
each of those three must also add `in_flight` to its `framing` decode table,
or it will refuse the token it now accepts a version for. Ordering: consumers'
mains first, then yog publishes, then the consumers publish.
