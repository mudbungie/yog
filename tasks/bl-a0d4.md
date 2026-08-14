+++
title = "the model picker states what it reaches, and a conversation states the config it is frozen at"
created = 1786162399
updated = 1786683219
claimant = "Halyard"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
**Verified against the tree 2026-08-07 (Ptarmigan): both halves of this ball are already shipped. What is left is a weight question, not a missing fact.**

## What I filed, and why it was wrong

Filed on the claim that the §9.4 picker "accepts an instruction it will never carry out and says nothing about it", and that a conversation never states the config it is frozen at. Reading the code, both were already answered — by bl-9786 and bl-824e, landed 2026-08-01, and present in the build the operator was running at the time (a debug binary from `target/debug/yog`; both commits predate it).

**The picker states its scope.** `model_pick::scope_sentence` (`src/model_pick/mod.rs:165`), painted at the top of the pane (`shell/model_pick/mod.rs:109`), verbatim:

> changes config/<branch> for the whole <workspace> workspace — it governs the NEXT conversation started here; this one stays frozen at <short-oid>

The `change…` button's hover says it again: *"A change here takes effect for the NEXT conversation — this one stays frozen on the config it started with"*.

**The conversation states its frozen config, and its drift.** `model_pick::header::frozen_header` (`src/model_pick/header.rs:84`) composes the conversation header's model line as `model · <model> · frozen at <oid>`, and when the config lineage has advanced past it, appends `· workspace default is now <model> at <oid>` and sets `drifted`. Drifted additionally paints `NEW_CONVERSATION_EXIT` — the one exit that exists today.

In the open conversation at that moment, that line would have read:

    model · claude-opus-4-6 · frozen at d93c2a9 · workspace default is now gpt-5.6-sol at a06b090

## What actually went wrong, from process evidence

The operator was not under-informed about the freeze. They were mis-routed on the **credential**: about a minute after two steps failed on one provider row, they launched `yog bz --login --provider <row> --browser` against a *different* row. The auth affordance offered Login without naming the row, so the sign-in went to the wrong provider. That is bl-8e34, filed and landed the same session; this ball's evidence is the confirmation of that one.

## What is genuinely left here

One question, and it is the operator's to rule on, not an implementer's to guess: **the true sentence is painted in the weakest style on screen.** The freeze and the drift are `ui.weak()` in a header row, while the gesture that misleads — the picker accepting a pick — is a bright, immediate interaction that reports success. The facts are present and correct; what is untested is whether they carry enough weight to be read at the moment they matter.

Do not implement a restyle off this ball as filed. Either rule on the weight, or close this and let the real remaining gap carry it: there is no way to move a live conversation onto newer config at all — the only exit offered discards the conversation's whole history. That is lernie bl-22a5 (`retarget`) and yog bl-2d19.