+++
title = "a conversation whose driver died renders no cause: zero-byte response.json reads as a quiet step"
created = 1784955876
updated = 1784958004
claimant = "Exultantly-7f2e"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found by the bl-8e07 real-substrate evaluation (docs/drive-logs/2026-07-24-s0-s1-wire-blocked.md).

REPRO (any silent driver death; the one observed was a substrate pin skew)
1. make `lernie prompt` fail after it writes the step but before any response
   (observed live: installed bz 0.0.4 vs lernie linked brazen 0.0.3, so
   `lernie prompt` aborts with a version-mismatch line on stderr).
2. start a conversation in yog. lernie writes steps/<agent>/001/request.json,
   then leaves response.json at ZERO bytes and no meta.json.

WHAT RENDERED WHEN THIS WAS FILED
- attention worked: the agent classifies Stopped from the framing, §6 rule 2
  stirs the strip ("3 need attention"), each conversation row badges ⚑1, and
  focusing a row acknowledges it (strip 3 -> 2). Correct.
- cause was nowhere: the Steps tab showed `001 · 0 attempts · 0 tok` with the
  ash "stopped" badge (x3-steps.png) and the Transcript showed only the user
  goal. The zero-byte response.json it left behind rendered as an ordinary
  quiet step.

CORRECTION TO THE ORIGINAL FILING (bl-a649 landed after this ball was written)
The body said the driver's stderr "went to /dev/null (§13.3 accepts that
trade)". That is no longer true. Detached lernie stderr goes to a per-spawn
sink `<yog_state_root>/detached/<ts>-<workspace-leaf>.err`, and each ops sweep
folds the sink's tail into the `-2` ops row, lighting the §7.3 banner and the
§11 activity chip's ⚠ count. So the WHY was already surfaced — on the OPS
surface. What was still owed, and is what this ball delivered, is the
CONVERSATION surface.

DELIVERED
- The §7.3 **no-response wound** (src/steps_view/wound.rs): a step whose
  `response.json` carries no bytes (absent or zero-length) AND whose
  `meta.json` is absent AND whose agent nobody is driving (§3.5 liveness,
  threaded in as `steps_view::build`'s `AgentState` argument) is the distinct
  derived state "driver produced no response". Derived at read time from the
  files, stored nowhere. lernie writes meta.json only after the model call
  returns (ARCH §2.3), so the pair says exactly: emitted nothing, never
  settled. The liveness half is what keeps it off a live driver's newest step,
  which is legitimately empty between opening response.json and the first
  streamed event (§10, never a false definite); only the newest step can be
  the one a driver is filling, so the gate applies to it alone and a
  historical wound stays rendered after a resume.
- It renders twice, one rule: an ichor ✗ badge plus the sentence "driver
  produced no response" beside the Steps row (outranking the framing read,
  which paints the same ash ■ a mid-stream kill gets), and an ichor banner at
  §11 Altitude 1 — above the inspector, so the cause is on the conversation
  surface whichever tab is open, including the Transcript that by construction
  has nothing to show.

RULING: NO REPRODUCTION HATCH AT THE WOUND (recorded in DESIGN §14, §8.4)
A "re-run with stderr attached" button (`yog exec lernie prompt …`) is
rejected on the evidence, not the ergonomics: post-a649 the driver's own words
are already captured and rendered on the ops surface, so the button would ask
the operator to re-create a fact yog holds — and would fire a second driver at
a conversation that already has one, with a goal yog would have to re-compose.
The wound names where the cause lives instead. `yog exec` stays a hatch for a
human at a shell (§8.4), not a verb yog fires at itself.

The detached spawn (§8.1/§13.3) is untouched: yog dying must never kill a loop.

Docs reconciled: DESIGN §5.1 #13, §6, §7.3 (new failure row), §8.4, §11
(Altitude 1 + Steps bullet), §12 module map, §13.3 (the conversation-surface
half), §14 (the rejection); STORIES S0 step 4.

Related: bl-20f4 (harness), bl-a649 (the stderr sink), and the substrate skew
ball filed alongside.