+++
title = "output-limit exhaustion is classified as clean quiescence, leaving an empty answer and a Nudge that cannot advance"
created = 1787544234
updated = 1787544345
claimant = "Playlists"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["defect", "agent-state"]
+++
A provider call can finish as a transport without finishing the agent turn.

Canonical failing shape:

- the request sets `max_tokens = N`;
- the assistant emits thinking but no text and no `tool_use`;
- usage reaches N;
- the stream ends with `Finish{Length}` and `End`, with no `Error`.

Today `git_tree::terminal` calls every `Finish + End` segment with no error `Complete`, and `git_tree::state` therefore derives `Quiescent`. The steps view discards the finish reason. Linked lernie commits the tool-free assistant turn as a final response; a later `advance` sees an assistant-side tail and derives `NothingDue`. The window consequently presents a clean rest with no answer, then offers a Nudge that cannot create a step.

## Invariant

Transport completion is not task completion. A terminal, tool-free `Finish{Length}` turn is truncated, never clean quiescence. The canonical finish reason is the authority; derive from it rather than guessing from token counts.

## Required outcome

1. Extend the one response-segment fold to preserve the semantic terminal result beside transport framing.
2. Derive an actionable stopped/failure reading for a terminal `Finish{Length}` turn. Keep the coarse badge vocabulary ruled by bl-d816; the existing workspace/step failure carrier must say that the output limit ended the turn.
3. Do not offer Nudge for a shape linked lernie deterministically reads as `NothingDue`. Point to the existing Message action as recovery. Add no verb, flag, or blind retry.
4. Preserve partial text as visible but mark the turn truncated. A complete tool-use continuation must not be misclassified. Existing `Finish{Stop}`, refusal, `Error + End`, and missing-`End` behavior must remain intact.
5. Amend DESIGN before code if its current complete/quiescent invariant disagrees.
6. Pin the whole matrix with tests: thinking-only Length, partial-text Length, clean Stop, Length with a tool continuation, Error plus End, missing End, rendered reason, and control gating.

## What does not solve it

- Raising the 4,096 default only moves the failure.
- Painting explanatory text while leaving the agent Quiescent and Nudge enabled leaves the mechanism false.
- Treating every Finish as failure destroys clean completion and refusal semantics.
- Inferring exhaustion from `usage == max_tokens` duplicates a fact the finish reason already states.

If yog cannot derive tool continuation from its existing disk facts, correct the design and file a linked upstream lernie task before implementation; do not add a local heuristic.