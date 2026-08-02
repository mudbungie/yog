+++
title = "alignment watcher sidecar: cheap model streams agent thinking/output, scores intent alignment, drives graduated interventions"
created = 1785652106
updated = 1785652106
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["safety"]
+++
Operator concept 2026-08-01, filed alongside the codex side-car safety-watcher replication work. This is design-first: the deliverable is a design ruling (VISION/DESIGN amendment) before any implementation ball.

## The mechanism

A second, cheaper model watches the agent live — its thinking and output token stream — and continuously answers three questions:

1. What is the user trying to do?
2. What is the model trying to do?
3. Are they aligned?

On detected misalignment, a graduated response ladder fires (cheapest first):
- revoke tool auto-approval (further actions need review)
- escalate to a stronger reviewer model
- inject a system notice reminding the agent of the pinned user intent
- kill the inference cycle outright

## Prior art (research exists; no known shipped implementation in an agent harness)

- Redwood Research "AI Control: Improving Safety Despite Intentional Subversion" (2023): trusted-monitoring protocol — weak trusted model scores every action of a strong untrusted model, audit budget for escalation. This ball is a production instance of that.
- OpenAI "Monitoring Reasoning Models for Misbehavior" (2025): weak model watching a frontier model's CoT catches reward hacking better than action-only monitoring. Negative result that matters here: optimizing the agent against the monitor teaches obfuscation. We don't train against it, but injected notices leak the monitor's existence — expect in-context adaptation; keep notices as a late rung, not the default.
- Anthropic runs streaming classifiers that halt generation mid-stream (constitutional classifiers) — proves the kill mechanism, for harm categories not intent.
- Llama Guard / NeMo Guardrails classify per-message *harm*, not alignment to this user's intent. Intent alignment is the underbuilt part and the actual novelty.

## Design constraints (attack these in the ruling, don't assume)

1. Enforcement point is the control boundary. Actions happen only at tool calls; thinking can only be read. The watcher's veto sits at the existing typed action chokepoint (bl-8aab control boundary); the token stream is early context, not an enforcement point. One cheap call per action, not per token chunk — this also bounds cost.
2. Pin intent once per user message; store it. "What is the user trying to do" is extracted when the user speaks, not re-derived every check. Drift from the pinned intent is the signal. Single source of truth.
3. The ladder is the audit-budget shape: cheap monitor everywhere, expensive review only on flags. Each rung must be a distinct, testable action on the existing surface.
4. Watcher reads tool results, so prompt injection can target the watcher too — it treats the transcript strictly as data, never as instructions.
5. Verify what the watcher can actually see per provider: API thinking blocks may be summarized, not raw.
6. Testability: the watcher's verdict must be a pure function of (pinned intent, transcript window) so it can be replayed and regression-tested against recorded transcripts.

## Verify before designing

Check the then-current control boundary surface and the codex-watcher replication work's landed shape — this ball was filed before that landed and premises may have moved.