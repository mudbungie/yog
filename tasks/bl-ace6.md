+++
title = "an unanswered delivered message with no live driver paints nothing: the chat just stops — derive the dead-driver banner, and finally read driver.log"
created = 1786843880
updated = 1786843886
claimant = "Dills"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## The operator experience

Send a second message into a conversation whose driver dies (provider 400 on a wedged tail, UnpairedToolUse, a crashed launch, a version skew): the deposit succeeds, ops.jsonl records exit 0, the transcript stops, and no pixel anywhere says why. The cause is on disk twice — steps/<agent>/driver.log (lernie 0.0.9 bl-55f9 binds every launched driver's stderr there; yog pinned 0.0.9 partly FOR that file, Cargo.toml comment, and nothing reads it), and the error event inside the failed step's response.json, whose derived error_text has exactly one consumer, login::auth::classify. DESIGN §13.3's claim that 'the §7.3 no-response wound and the §8.1 driver-stderr sink surface any dead driver, embedded or ambient' is false for the ordinary case: the 2nd..Nth message of every conversation is driven by a child LERNIE launched, which has no §8.1 sink and, when it dies before creating a step, no wound either. STORIES INV-2 (no swallowed errors) is the governing invariant this violates.

## The derivation (store nothing)

A delivered message nobody is answering is derivable from the branch alone: the newest transcript entry is a delivered NNN-<sender>.md AND the agent's executor lock is free. Delivery only ever happens under a driver's lock and the driver holds that lock through the model call, so on a healthy branch the state exists only for the relaunch gap — grace-gate it like wound_grace and it is exact. It stays silent on: a held branch (tail is the unpaired window), a stopped agent (newest entries are settled tool results, not .md mail), a live or in-flight driver (lock held).

## The banner

Third workspace-pane banner beside auth and wound (shell/workspace.rs): 'a delivered message has no driver — the last driver said: <tail>', tail = last lines of steps/<agent>/driver.log, read lazily only when the banner fires (the wound's own stderr.log discipline; driver.log is append-only across launches, which is why file content must never be the trigger, only the diagnosis). Absent or empty log: say so ('no driver output recorded').

## Gate

tests (the derivation's arms: fires on mail+free-lock after grace; silent on held/stopped/live/fresh), docs (amend DESIGN §13.3 — strike the falsified 'any dead driver' claim, record this banner), alignment vs STORIES INV-2.