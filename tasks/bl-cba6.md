+++
title = "acceptance worlds mint from real entropy: pin the fixture's mint seed so a painted name cannot collide with a test's needle"
created = 1786162402
updated = 1786162410
claimant = "seed-pin"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
`shell::acceptance::drafts::a_draft_belongs_to_the_target_it_was_typed_for`
failed once in ~10 runs during bl-f908, on an assertion that has nothing to
do with the rule under test:

    assert!(!back.contains("ping"), "the message draft stayed with its agent");

The screen it searched contained `will be named dripping`. The needle `ping`
is a substring of a **minted wordlist name** (§3.3), and the acceptance world
mints from real entropy (`shell::clock::entropy_seed`), so the painted name is
a different word every run. bl-f908 changed that one needle to a phrase as a
stop-gap; the class is still open, and every acceptance test that asserts over
`Screen::text` shares it.

## The fix: the fixture's mint seed is an input, not entropy

A test that renders a minted name must know which name it rendered. Seed the
acceptance `World` with a **fixed** mint seed so the §3.3 preview is the same
word every run, then assertions can name it outright instead of dodging it.
The seed is already a value the shell holds (`ShellState::start.mint_seed`,
minted once at startup from `shell::clock::entropy_seed`) — the fixture should
set it rather than inherit the process's.

What to check while doing it:

- `acceptance/mint_seed.rs` is the bl-28ba drive that *a landed fire retires
  the seed it spent and a failed one does not*. A fixed seed must leave that
  test asserting the same thing — it is about the seed **changing**, which a
  known starting value does not weaken (it strengthens it: the retired value
  becomes assertable too).
- With the name known, revisit the needles bl-f908 dodged
  (`acceptance/drafts.rs`) — an exact assertion is better than an evasive one,
  but do not churn tests that were never at risk.
- Sweep the other `Screen::text` / `painted()` assertions for the same
  collision shape (a short lowercase word searched for in the whole screen)
  and say in the close what you found, even if the answer is "no others".

## Not this

Do not make the assertion fuzzier, do not retry the test, and do not exclude
the preview from the painted text — the preview reaching the paint layer is
itself a §3.3 rule other tests rely on. The entropy is the defect.