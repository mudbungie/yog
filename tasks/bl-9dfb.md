+++
title = "in-flight strip gains elapsed: the call's start is in the structure (step/dispatch commits, landed tool inputs), not a file timestamp"
created = 1785729860
updated = 1785729860
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Operator ruling (2026-08-02, verbatim)

'in-flight strip is also telemetry we have; we know when the commit is made, when lernie/brazen is invoked. it's not a file timestamp per se, but it's definitely in the structure.'

This OVERRULES bl-905f's elapsed refusal (recorded in DESIGN §5.1 #28 and §11): that refusal rejected mtime (last-token, not first), birth time (non-portable), and a stored yog-side flag — all correctly — but did not exhaust the STRUCTURAL starts. The world's own records mark when each call began; derive elapsed from those.

## Candidate structural starts (verify each against the tree and lernie's commit discipline before trusting — this is the heart of the task)

- **Inference:** the commit that opened the current step. While a model call streams, the agent branch tip is the commit lernie made immediately before invoking the model (the delivered message / tool result / step commit) — its committer timestamp is already in the snapshot as `tip_timestamp_unix`. Verify the invariant ('tip during InFlight = the commit that preceded the invocation') against lernie's dispatch/advance flow rather than assuming.
- **Tools:** the tool call's `input.json` lands when the call starts (§5.1 #10 uses exactly that landing as the state signal). Its commit's timestamp — or, if it rides uncommitted, its mtime, which for input.json IS the start (written once at invocation, never appended) — bl-905f already plumbed input.json reads at enumerate time (ToolCall::name).
- **Subagents:** the child's dispatch commit timestamp (the `dispatch: <role> [<id>]` commit).

Whichever facts hold, gather them at snapshot/enumerate time beside the fields bl-905f/bl-cad5 added — never stat or walk git from the render path. If one class has NO honest structural start, that class simply shows no elapsed — partial delivery is lawful; fabrication is not.

## Rendering

Append `· <elapsed>` to the strip line (compact form like the age labels: 42s, 7m). Ticking display: the strip already repaints on the pulse cadence, so elapsed = now − start recomputed per frame from the snapshot fact; nothing stored. Amend DESIGN §5.1 #28 and §11's third-seat ruling: replace the elapsed refusal with the structural-start doctrine (record the operator ruling and keep the refusal's reasoning for the three rejected sources).

## Discipline

Tests per class: strip states elapsed derived from the structural fact; a class lacking the fact omits it; label formatting shared with age_label (one home — reuse it, don't restate). Verify bl-905f's files as they exist now (src/nav/convs/flight.rs strip, src/git_tree/{model,tools}.rs, src/app/view.rs, src/shell/flight_strip.rs).