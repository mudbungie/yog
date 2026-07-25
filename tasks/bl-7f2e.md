+++
title = "a conversation whose driver died renders no cause: zero-byte response.json reads as a quiet step"
created = 1784955876
updated = 1784955876
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found by the bl-8e07 real-substrate evaluation (docs/drive-logs/2026-07-24-s0-s1-wire-blocked.md).

REPRO (any silent driver death; the one observed was a substrate pin skew)
1. make `lernie prompt` fail after it writes the step but before any response
   (observed live: installed bz 0.0.4 vs lernie linked brazen 0.0.3, so
   `lernie prompt` aborts with a version-mismatch line on stderr).
2. start a conversation in yog. The detached spawn is clean (ops row exit -2,
   empty stderr, no warning chip), lernie writes
   steps/<agent>/001/request.json, then leaves response.json at ZERO bytes and
   no meta.json.

WHAT RENDERS TODAY
- attention works: the agent classifies Stopped from the framing, §6 rule 2
  stirs the strip ("3 need attention"), each conversation row badges ⚑1, and
  focusing a row acknowledges it (strip 3 -> 2). Correct.
- cause is nowhere: the Steps tab shows `001 · 0 attempts · 0 tok` with a
  stopped badge (x3-steps.png) and the Transcript shows only the user goal.
  The actionable sentence lived in the detached driver stderr and went to
  /dev/null (§13.3 accepts that trade), and the zero-byte response.json it did
  leave behind is rendered as an ordinary quiet step.

WHY IT MATTERS
STORIES S0 step 4 promises "any step failure is a rendered fact" and §7.3 says
a failed action is never stderr-only. This class satisfies neither: the
operator is told THAT the conversation died and never WHY, with a
zero-attempt/zero-token row that reads like nothing happened.

FIX DIRECTION (not done here; design call first)
- The empty/truncated response.json IS evidence: a step whose response file is
  zero-length with no meta.json is a distinct derived state ("driver produced
  no response"), not a quiet complete step — render it as a failure row with
  that wording, in ichor, beside the step.
- Consider surfacing the reproduction hatch at the wound: `yog exec lernie
  prompt …` re-runs the same verb with stderr attached (§8.4), which is how
  this instance was diagnosed.
- Do NOT undo the detached spawn (§8.1/§13.3): yog dying must not kill a loop.

Related: bl-20f4 (harness), and the substrate skew ball filed alongside.