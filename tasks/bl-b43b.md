+++
title = "a conversation refused at the provider rung paints as `stopped` with an empty transcript and an empty-stderr trail row: only /steps carries auth_failed, and `stopped` is the word /stop already owns"
created = 1788150402
updated = 1788150495
priority = 7
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["ux", "remote"]
+++
A conversation started on an engine with no provider credential dies at the
first model call. The fact is recorded correctly and then shown on exactly one
surface, which is not one the operator would think to open.

## What each surface says

Driven against a server whose role model resolves a provider row with no
credential stored:

- `lernie start <ws> <goal>` — exits 0 and prints `{"kind":"started","ok":true}`.
  The start genuinely succeeded; the failure is a step later and asynchronous.
  Nothing wrong here on its own, but it is the last thing the operator sees.
- the roster (`conversations`) — one row, `state: "stopped"`, `tone: "plain"`,
  `uncertain: false`, `attention: 1`.
- the transcript — the user message and nothing else. No model row, no wound,
  no note.
- `agent` — the surface whose own help says "what it is doing, what may be done
  to it" — carries `state: "stopped"`, `nudgeable: true`, zero spend, and no
  `marks` at all.
- the ops trail — one row, the `litany prompt` argv, `exit: -2`, and **empty
  stderr**. The child's own words about why it could not start are gone.
- `steps` — the only surface that answers: `{"seq":"001", "framing":"failed",
  "auth_failed":true, "auth_row":"anthropic", "attempts":1}`.

## Why "stopped" is the wrong word here, specifically

`stopped` is what an operator-issued `/stop` produces. Both were driven in the
same session against the same build, and on the roster they are the same row:

    operator /stop      state=stopped, steps[0].framing="killed",  auth_failed=false, marks=[notified, abandoned]
    provider refusal    state=stopped, steps[0].framing="failed",  auth_failed=true,  marks=[] (absent)

Two different events, one word, and the word is the one that means *you did
this*. An operator who did not stop anything is told their conversation is
stopped, on a plain-toned row, with an empty transcript and an attention count
of one — and the remedy (sign a provider in) is named nowhere on any surface
they are looking at.

The data to fix this is already there and already correct. `auth_failed` and
`auth_row` are computed, stored and answered; they simply never reach the row,
the transcript or the trail.

## What comprehensible would look like

- The roster's state (or its `tone`/`signals`) distinguishes *stopped by you*
  from *could not start*. It does not need a new field if `signals` can carry
  it — the seat already renders that array.
- The transcript is not empty. A conversation that has been refused should paint
  the refusal where the operator is already reading, naming the row
  (`anthropic`) and the act (sign in on that workspace). This is the same class
  as the swallowed-error balls already on the board: a conversation that reads
  as idle when it is actually wounded.
- The ops trail's `exit: -2` should not come with empty stderr. Whatever the
  child said is the only first-hand account of the failure, and the trail is
  where an operator goes for it.

## Note on scope, so a fixer does not chase it

An unscoped `attention` ask answers the OWN engine's workspaces only, so a
conversation on a second engine never appears in it regardless. That is
consistent with the seat holding one channel per entry and reading through, and
is not part of this ball — but it does mean the roster row above is the operator's
only passive sighting of the failure when the engine is a remote one.

---

Confirmed on the WINDOW as well as the CLI, which is the surface that matters. The refused conversation's roster row reads `<name> [stopped] 7m 1 waiting` — the identical shape an operator-stopped conversation gets — and the conversation pane paints the user message and then nothing at all. No wound row, no provider row named, no remedy offered. The window carries none of the auth_failed / auth_row facts the steps surface holds, so on the operator's primary face a provider refusal is indistinguishable from a conversation they stopped themselves.
