+++
title = "the terminal seat tells the operator to focus a workspace it cannot focus, and never names --ws: the flags exist only in a refusal that missing-target refusals do not print"
created = 1788235234
updated = 1788318657
claimant = "Signal"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "language"]
+++
## Symptom

The terminal seat's refusals send the operator to a control it does not have,
and its help never names the flags it does.

```
$ yog gesture '/conversations'
yog gesture: /conversations: no workspace in context — focus one, or use the envelope
```

There is nothing to focus: `src/boundary/sugar/argv.rs` opens by saying so —

> The terminal is a seat like any other, and **it holds no selection**: nothing
> is focused, so a line typed here states its targets outright. That is what the
> flags are…

— and the remedy that *is* reachable, `--ws NAME`, is not named. "Use the
envelope" points at the JSON spelling, which is a second, larger detour past
the flag that exists for exactly this.

The flags are also absent from the only place an operator looks for them:

```
$ yog gesture --help
  /message <text…>
      …                       # the whole gesture list, and nothing else
```

`yog --help` says only `yog gesture <gesture>`. The usage line naming
`--ws/--agent/--project/--as/--prepared` exists (`argv::usage`) and is printed
**only on a refusal** — so the way to learn how to aim a gesture is to type one
wrong, and the refusal you get for a *missing target* is the one refusal that
does not print it.

## Why the sentence cannot simply change

`args::workspace` is shared with every seat, and "focus one" is right for a
windowed one. The module doc states the obligation the shared sentence is
currently failing at one of its two seats:

> Every refusal here says *which* verb refused and *what* was missing, because a
> typed control obeys §11's discoverability rule exactly as a clicked one does:
> **the operator must learn what the gesture needed without reading the
> source.**

So the fix belongs at the seat, not in the shared string: the argv layer already
appends its own usage to the refusals it owns (`unknown flag --x; usage: …`,
`nothing to do; usage: …`) and does not to the ones the line parser hands back.

## Repro

    yog gesture '/conversations'      # no --ws → "focus one, or use the envelope"
    yog gesture --help                # the gesture list; no flags anywhere

## Shape of the fix

Two small choices, one ruling each:

- Whether the argv seat appends its own usage to a **context** refusal from the
  line parser (it already does for its own two), or the shared sentences learn a
  seat-neutral remedy that names both the focus and the flag.
- Whether `yog gesture --help` prints the seat's usage line above the shared
  gesture list. The `--help` rewrite exists so *one answer serves both seats*, so
  this has to be the seat printing its own line around that answer, never a flag
  list inside it.