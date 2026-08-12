+++
title = "per-agent task branches: each agent tracks on its own balls branch by default, settable at launch, inherited by subagents, amendable by the agent"
created = 1786508785
updated = 1786508785
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-9dde"
on = "claim"

[[blockers]]
id = "bl-c0e2"
on = "claim"
+++
Operator ruling 2026-08-11: "The balls interface is broken right now. The
default is going to be that each agent gets its own balls branch for tracking,
but that an agent can have its branch set at launch, and that subagents, by
default in yog, get their parents' space passed to them. Obviously an agent can
amend their own branch, to change their config (necessary when an agent is
launched but told to work on a project)."

Implement the amended DESIGN §16.3: per-agent tracking branch as the default,
a launch-time branch parameter, parent-space inheritance for subagents, and a
runtime path for an agent to amend its own branch. The bl conf surface
(world::marks) is the existing write path — drive it, don't duplicate it.

Also: "the balls interface is broken" is an observed failure, not a metaphor —
before building, exercise the current balls board/section against a real store
and file (or fix here, if in scope) what is actually broken. Verify every
premise against the tree; the world env composition will have just moved under
the blast-radius ball.