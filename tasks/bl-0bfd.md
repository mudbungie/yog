+++
title = "subtract the retired roster walk: AppModel::roster_step, Pick::Step and attention::step are dead outside their own tests"
created = 1786514885
updated = 1786684151
claimant = "Kerf"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["cleanup"]

[[blockers]]
id = "bl-fa82"
on = "claim"
+++
Filed by Alkaloid 2026-08-11 on Jasper's finding while delivering bl-d5b9.

## What is dead and why

The bl-fa82 expander ruling retired the ↑/↓ roster walk: the list now walks `convs::step` over `visible_rows`, and §6's rank keeps only the jump and the §8.5 queue. That leaves three items with **no production callers** — they are exercised only by their own tests:

- `AppModel::roster_step`
- `Pick::Step`
- `attention::step`

Jasper deliberately did **not** delete them in bl-d5b9: that ball was fenced to shell glue ('no derivation logic here'), and removing them is derivation surgery. It corrected `roster_step`'s doc to record that ↑/↓ left it, and flagged the subtraction as worth its own ball. That was the right call — this is the ball.

## Why it matters beyond tidiness

Code kept alive only by its own tests is the worst kind: it reads as load-bearing, it holds the 100% coverage floor up with tests that prove nothing about the product, and the next reader cannot tell it is vestigial without re-deriving the ruling. **Tests that exist only to cover dead code are coverage theatre** — directly adjacent to the vacuous-assertion sweeps in bl-36c3 and bl-f16e.

The house principle applies: *minimalism, subtraction in design — code is context and context is inertia.*

## Scope

1. Confirm at HEAD that each of the three has no non-test caller. **Verify; do not trust this body.** The epic (bl-fa82) is still open and may move things.
2. Delete them and their tests together. A test deleted alongside its only subject is not lost coverage.
3. Check whether anything else the ruling retired is now unreachable — §6's rank, `Pick`'s other variants, the attention plane. One sweep, not three balls.
4. Coverage must stay at 100% **without** the deleted tests propping it up. If removing them drops coverage elsewhere, that names a real gap — file it rather than restoring the dead code.

## Sequencing

Do not claim until **bl-fa82 is closed** — its last child bl-89de is still open, and this touches the same surfaces the epic is landing. Verify all cited symbols against HEAD first; ball bodies drift.

---

bl-8905 (Fretwork, 2026-08-11) adds a fourth item of the same kind, already removed there so this ball does not have to: **AppModel::conversation_members**. Retiring the altitude-1 descent tree left it with no production caller — only its own unit test in src/app/tests/view.rs and the S7-T5 story — which is this ball's exact shape ('code kept alive only by its own tests… holds the 100% coverage floor up with tests that prove nothing about the product'). It is deleted in bl-8905's delivery along with its test, and stories_s7_t5.rs is re-pointed onto visible_conversations, which is the surface that survived.

Not claiming this ball, and nothing here changes its scope: AppModel::roster_step, Pick::Step and attention::step are untouched by bl-8905. One caution for whoever takes it — verify at HEAD as this body already instructs, because bl-fa82 and bl-8905 both landed after it was filed and both moved this area. Note nav::convs::members is NOT dead and must not be swept up with it: flight.rs, doing.rs and delete/agent.rs all still fold over it.
