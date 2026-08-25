+++
title = "a crashed tool window paints nothing — the third member of the swallowed-error class, and the only one that reads as an idle conversation"
created = 1787544496
updated = 1787622437
claimant = "Davit"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## The shape nothing renders

An agent whose executor died mid-tool-window leaves its assistant entry
committed with `tool_use` blocks nobody answered, no hold mark, and its lock
free. yog paints **nothing** for that: the conversation reads as an ordinary
idle one that simply chose to stop.

Neither existing state fires, and for good reasons that are worth stating so a
fix does not get grafted onto the wrong one:

- The **§7.3 no-response wound** (`src/steps_view/wound.rs`, bl-55d8) is
  *unanswered on disk*: `response.json` empty and `meta.json` absent. Here the
  model call returned and settled — `meta.json` exists. Not a wound.
- The **orphaned-mail banner** (`src/steps_view/orphan.rs`, bl-ace6) is *newest
  entry is mail and the lock is free*. Here the newest entry is an assistant
  `.json`, not a `.md`. Not an orphan.

So the class the two banners exist to close — a step failure rendered as a
fact, STORIES INV-2, "no swallowed errors" — has a third member with no paint.

## Why it is not urgent, which belongs in the ball

The state is genuinely transient and self-healing under the pinned lernie: the
next drive boundary settles the window with an in-band `is_error` `tool_result`
per unanswered id, before delivery, and an ordinary deposit revives the branch
(lernie ARCH §6, upstream bl-4187, consumed here in bl-4c1f). Once settled the
error results are in the transcript and the operator can read them.

What makes it worth painting anyway is the window BEFORE that deposit: nobody
deposits into an agent that looks finished. On an unattended box (the headless
deployment, bl-bf35 / bl-286b) that window has no upper bound.

## Shape of the fix, if it is taken

Third instance of the pattern the other two already established — a derived
predicate plus a reason read off disk, **nothing stored** (§5.1 #13), and **no
new badge vocabulary** (both existing banners kept it untouched; bl-d816 tracks
whether that should ever change):

- predicate: newest transcript entry is an assistant entry carrying `tool_use`
  ids with no committed `tool_result`, no hold mark, agent lock free;
- reason: the tail of `steps/<agent>/driver.log`, exactly as the orphan banner
  reads it, since the same died-driver wrote it;
- sentence: says the turn died mid-tool-window and that a deposit revives it —
  the recovery is one gesture and the banner should name it.

Attack it before building: the predicate needs the same grace window the wound
and the orphan take (bl-90bf), or a live driver between committing its
assistant entry and committing its first tool result alarms falsely. That is
the one hard part; the rest is a third copy of a shape that works.

Filed from bl-286b, which asked whether this deserves paint and answered yes,
narrowly. That ball's disk sweep found zero live agents in either shape, so
this is a promise-completeness fix, not a live defect.