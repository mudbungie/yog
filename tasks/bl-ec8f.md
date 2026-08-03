+++
title = "slash commands: the line serialization of the control boundary (parity in one text spelling)"
created = 1785733833
updated = 1785733834
claimant = "Gangplank"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ask (2026-08-02): "in pursuit of everything in yog being teleoperable,
and also with tui support, implement parity to the control interface via slash
commands."

The §8.5 boundary has two serializations today: the in-RAM variant (GUI
click-glue) and the codec JSON envelope (headless deposit). This ball adds the
THIRD and last: the **line** — a slash command, the spelling a human types in
the composer and a TUI or teleoperator types anywhere.

Deliverables:
1. `boundary::line` — `parse(input, &Context) -> Result<Gesture, String>` and
   `spell(&Gesture) -> String`, both total over the Gesture surface; the roster
   table (verb, usage, summary) is the single source for /help, refusals and
   completion. Compile gate: spell's match is exhaustive, so a new variant
   without a line spelling does not build.
2. Context: the selection facts a seat holds (workspace, agent, project, name,
   ball, prepared). Omitted parameters resolve from it; an unresolvable one
   refuses naming what is missing. The line is the terse context-bearing
   serialization; the envelope stays the total context-free one.
3. Composer seat: a draft starting with `/` is a command, not a message. `//`
   escapes a literal leading slash. Actions dispatch through the same
   AppModel::dispatch a click uses; queries render their answer JSON inline.
4. argv seat: `yog gesture` accepts a line as well as an envelope, with
   explicit context flags, so slash commands are teleoperable from a terminal.
5. DESIGN §8.5 amendment recording the third serialization and the roster.

Not a second implementation: parse builds the same variants, dispatch is
unchanged.