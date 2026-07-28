+++
title = "conversation display name: one derived function — first payload line, harness workspace line stripped, root id fallback"
created = 1785201512
updated = 1785201512
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["implementation"]

[[blockers]]
id = "bl-2706"
on = "claim"
+++
Implementation of the bl-68d9 design ruling (DESIGN §3.3 as amended). Needs
bl-2706 (the preamble wording it strips is settled there). Read DESIGN §3.3
and §11 first.

## The ruling being implemented

DESIGN §3.3:

> **A conversation's name is its first payload line — derived, never minted,
> never stored.** The §11 row title, the composer's `message <x>` target
> label, and every "what do I call this row" fallback are **one function**:
> the first line of the conversation's goal with the harness-stamped workspace
> line stripped (the strip is the inverse of the stamp — compose and strip
> live in one module, exactly the `Ball <id>:` stamp's idiom; it also
> recognizes the retired `You are <name>.` shape, one extra line that
> lernie's 30-day workspace retention ages out), falling back to the root
> agent id when nothing else exists.

DESIGN §11 altitude 0: the conversation row shows "the conversation's
**display name** (§3.3's conversation-name rule: the first payload line,
harness workspace line stripped, root id fallback)". §11 altitude 1: the
center header shows "the conversation's display name with the root id weak
beside it — the id is the identifier, the name is the title".

## Today (the defect)

The preview is `steps/<id>/001/request.json` first-message content
(`src/git_tree/detect.rs:11-33` `extract_request_preview` +
`truncate_preview`, which collapses ALL whitespace to spaces before capping
at 80). The stamped goal leads with the harness preamble, so every
conversation in one workspace previews as "You are <workspace>. …" — the
sphere name burns the first ~22 chars of every row identically.

## Changes

1. `src/start/goal.rs` — add the strip beside the compose (the
   `parse_ball_stamp` idiom): given a goal's raw text, drop the first line iff
   it matches the workspace-preamble shape (`Your workspace is <x>.` — and the
   retired `You are <x>.`) plus the following blank line; return the rest.
2. `src/git_tree/detect.rs` — `extract_request_preview` strips BEFORE
   whitespace-collapsing (the strip is line-wise; collapsing first destroys
   the lines).
3. One display-name function (per bl-2f30's note: the navigator fallback and
   the composer label "should come from one function, not two"): preview
   non-empty => preview's first line, else root_id. Natural home: a method or
   fn beside `ConvRow` (`src/nav/convs.rs`); `src/shell/navigator.rs:223-227`
   (the `if row.preview.is_empty() { row.root_id }` inline fallback) calls it.
4. §11 altitude-1 center header: display name first, root id weak beside it
   (today the header leads with the raw conversation id).

## Acceptance

- A goal stamped "Your workspace is a-b.\n\nfix the parser" previews as
  "fix the parser"; legacy "You are a-b.\n\n…" strips the same way; a
  hand-typed goal with no stamp is untouched.
- Ball-rung conversations display "Ball <id>: <title>" (their first payload
  line); the row title, navigator fallback, and center header all derive from
  the one function.
- `make check` green.