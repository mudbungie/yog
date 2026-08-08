+++
title = "the model picker states what it reaches, and a conversation states the config it is frozen at"
created = 1786162399
updated = 1786162764
claimant = "Ptarmigan"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
The §9.4 picker accepts an instruction it will never carry out for the conversation the operator is looking at, and says nothing about it.

## What happened (conversation `ladies`, dev workspace, 2026-08-07)

Steps 009/010 failed on an expired borrowed credential. The operator changed the model to escape it. To the second:

    12:36:24  step 009 dispatched   claude-opus-4-6   auth error
    12:36:50  a06b090 on config/default: claude-session-direct/claude-opus-4-6 -> openai-chatgpt/gpt-5.6-sol
    12:36:57  step 010, the reprompt: request.json STILL says claude-opus-4-6, same auth error

The pick was written correctly and governs nothing here. `ladies` forked 2026-08-05 off `d93c2a9`; lernie derives a conversation's config from ancestry (`workspace::governing_config` — nearest `config/*` ancestor), so advancing the branch past the fork changes nothing about the branch. lernie ARCHITECTURE §4.1, verbatim:

> **Per-config** (`providers.yaml` in the config commit, §2.2). Role -> (provider row, model id) mapping and role toolsets only. Immutable for every agent forked from it — fork is the freeze (§2.2); governs which model that agent's roles dispatch to for the rest of its life.

Both halves are correct alone. Together they produce a UI that takes an instruction, reports no error, and does not do it. That is the defect: not the freeze, the silence about it.

## Deliverable — two sentences, one derivation, no new state

1. **The picker states its scope.** Selection is the gesture (§9.4, bl-fb6b) and stays so; what is added is the sentence beside it naming what the write reaches — the config branch it lands on, applying to conversations forked after it, not the one on screen.

2. **A conversation states the config it is frozen at.** `GoverningConfig` already carries the whole fact (`src/config_edit/branch.rs:41`): `short_oid`, `frozen_label()` ("policy frozen at <oid>"), and `branch_name_if_tip_of_one` — `Some(name)` iff the agent still runs that branch's head, `None` once it has advanced past the fork. Today only the Inspector's Config tab reads it (§11), which is not where the operator is when they pick. Surface the governing model where the conversation is driven, and when the governing commit is no longer any config branch's tip, say so — that is exactly the state in which a fresh pick will not apply.

With both, the sequence above is self-explaining at every step instead of silent. Neither adds a stored field: the picker's sentence is copy, the conversation's is the existing derivation given a second seat.

## Not in scope

Escaping the freeze on a live conversation — there is no verb for it (forking does not help: `lernie/src/prompt/fork_point.rs:16-18`, "fork is the freeze whatever the fork point is"). That is lernie's `retarget`, filed separately.