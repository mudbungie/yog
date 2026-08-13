+++
title = "the wall does not ride 'lernie message', so every second message to a quiescent conversation dies at 'bz: no workspace in this environment'"
created = 1786599898
updated = 1786599898
priority = 4
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator complaint, 2026-08-12, verbatim:

> it looks like the second message in a conversation always fails. Look at
> conversations stove and procedure.

## Root cause — found, not suspected

**The wall does not ride `lernie message`.** The revived driver runs outside the
workspace's §16.2 wall, so its first `bz` call dies, and the turn produces an
empty response.

Evidence, from the operator's own two conversations in
`~/.local/share/yog/workspaces/dev`:

`agents/<agent-id>` (`name` = `stove`) —

    001-user.md              "this thing on?"
    002-gpt-5.6-sol.json     "Yep—I'm here. What do you need?"
    003-user.md              "how about now?"     <- deposited 03:33:59Z
    (nothing further, ever)

`agents/<agent-id>` (`name` = `procedure`) — identical shape:
001 user, 002 assistant, 003 user at 03:35:01Z, then nothing.

The inbox is empty for both, so the deposit *was* flushed into the transcript.
The failure is one step later. `steps/<agent-id>/002/` holds a
zero-byte `response.json`, a truncated `staging.json` (`{"content":[`) and this
`stderr.log`, verbatim and identical in both conversations:

    bz: no workspace in this environment — providers, sign-ins and the model
    cache belong to a workspace, and there is nothing shared to fall back to.
    Run this inside a yog workspace, or focus one in yog.

## Why the first turn works and the second does not

The two spawns fold different environments.

The **first** turn is `lernie prompt`, fired from `boundary::dispatch`, which
layers the wall (`src/boundary/dispatch.rs:242-248`):

    let lernie = deps
        .lernie
        .and_env(crate::world::wall::pairs(&deps.world, &prepared.workspace))
        .and_env(crate::world::marks::pairs(...));

The **second** turn is `lernie message`, and `actions::verbs::message`
(`src/actions/verbs.rs`) layers only the name:

    let named = lernie.and_env(vec![(YOG_NAME.to_owned(), ws_name(ws))]);

No `YOG_WALL`. `src/world/wall.rs` states exactly why that one var is the whole
fix, verbatim:

    //! **One var carries it: `YOG_WALL`.** It names the wall root, and every
    //! other per-workspace location is derived from it ([`BrazenPaths`]) — one
    //! fact, one home, no second var to drift. Setting it on a workspace-bound
    //! spawn is enough for the whole descendant tree: the fired `lernie` loop
    //! inherits it, lernie hands its own environment to every tool subprocess
    //! it spawns (lernie ARCH §3.3), and a bare `bz` in an agent's bash is the
    //! world's shim re-entering yog (§16.7 W9/W12) — which folds the wall back
    //! out of its own process env. So the wall is set once, at the edge that
    //! knows the workspace, and no downstream seat has to be told.

`lernie message` is precisely such an edge: `cmd/message.rs` deposits and then
**detach-launches a driver** off `Fx::driver_target` when the branch is
quiescent (ARCH §2.9: no resume verb — the deposit restarts a driver). That
launched driver inherits `lernie message`'s process env, which yog handed it
without the wall.

Confirming the mechanism from the other side: conversation `remindful`
(`<agent-id>`) received its second message *while its driver was
still live and mid-tool-call*. That message went to the already-running
process — which was launched by `lernie prompt` and therefore **does** hold the
wall — and it was answered normally (`004/005/006-user.md`, then
`007-gpt-5.6-sol.json`). So the rule is not "the second message fails"; it is
**"any message that has to revive a quiescent driver fails"** — which in
practice is nearly every second message.

## The other two verbs with the same hole

Do not fix only `message`. Every yog-side spawn that can launch a
workspace-bound driver needs the same layer, and today three do not:

- `actions::verbs::message` — the bug above.
- `actions::verbs::fork` (`lernie dispatch`) — launches the **child's** driver;
  same fold, `YOG_NAME` only.
- `actions::verbs::scan` (`lernie scan`) — its stated job is *"flush inboxes"*,
  which is the same revive path, and it layers nothing at all.

`stop` is a SIGTERM cascade and launches nothing, so it is genuinely exempt —
state that in the code rather than leaving it as an unexplained asymmetry.

## Elegance constraint

Three call sites each remembering to add a layer is a fourth bug waiting. The
fold belongs where the workspace is known, once. Prefer making the
workspace-bound spawn seam itself carry the wall (the `Cli` handed to the
workspace verbs is already workspace-scoped by the time it is used) over
pasting `and_env(wall::pairs(...))` into three functions. If a per-verb layer
really is the only honest shape, then a total mapping — every verb states
whether it is workspace-bound — beats three independent decisions.

## Acceptance

A drive beat over the real substrate: start a conversation, let it go
quiescent, send a second message, assert a **non-empty** `response.json` for
the second step and an assistant message file at `004`. The current in-crate
tests pass with the bug shipped, so an assertion that only checks argv or exit
code is a vacuous one (bl-70b8, bl-f16e) — the beat must reach the reply.

Also worth a cheap standing guard: a step whose `response.json` is empty and
whose `stderr.log` is non-empty is a *rendered* failure in yog, not silence.
Today the operator's only signal was that nothing ever came back. File that
separately if it does not fall out of this fix.