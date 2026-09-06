+++
title = "a fresh box has no git identity, so the first start dies with a verbatim git error dump instead of a named prerequisite"
created = 1788673617
updated = 1788673617
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Round-1 install lane (yog docs/STORIES.md S0/S9: a stranger installs the suite on a fresh Linux box).

## Scenario step

Clean ubuntu:24.04 container, `cargo install yog litany thrall lernie brazen --locked` (all five build clean). Mint at a stated address, boot the engine, copy the four seat files, then start the first conversation.

## Gesture

    WIRE_HOST=127.0.0.1 WIRE_PORT=7741 yog wire-certs
    yog &
    lernie start home "hi"

## What came back

    {"error":"`new` failed (exit 1): litany new: git error: git [\"commit\", \"-m\", \"config: init [config/default]\"] exited with exit status: 128: Author identity unknown\n\n*** Please tell me who you are.\n\nRun\n\n  git config --global user.email \"you@example.com\"\n  git config --global user.name \"Your Name\"\n\nto set your account's default identity.\nOmit --global to set the identity only in this repository.\n\nfatal: unable to auto-detect email address (got 'root@<host>.(none)')\n","ok":false}

## Expected

A named prerequisite, refused at the same door the sign-in gate is refused at.
`src/boundary/dispatch/signin.rs` already establishes the shape: the Prompt door
holds a rung, the refusal is one sentence, fact first and remedy last, and a
refused fire spends nothing. Git identity is exactly that kind of fact — static,
knowable before anything is written, and universal on a box nobody has configured
git on. A stranger following the READMEs has no reason to have set it.

## Severity

p1: it is the FIRST act of the Settler story and it fails on every genuinely
fresh box. The reply is also a raw subprocess dump crossing the control boundary
as an `error` string, which is the shape a boundary is supposed to remove: the
seat prints twelve lines of embedded `\n` at a terminal.

## Related

The wedge this failure leaves behind is filed separately (a half-created
`workspaces/<name>` that refuses every later start on that name).