+++
title = "boundary::consumer's dead-claimant gesture tests fail only under full parallel suite load: three sightings across three agents, green in isolation"
created = 1788416884
updated = 1788484263
claimant = "Spellbind"
priority = 4
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["flake", "tests"]
+++
`boundary::consume{r}::tests::*dead_claimants_gesture*` (bl-d1f1's tests, e.g. `the_boot_answers_a_dead_claimants_gesture_in_doubt`) failed once each in three different agents' gate or suite runs today, always under full parallel load (a tarpaulin herd or 20+ concurrent suites), and passed on every re-run and every isolated run.

bl-5510 already examined the wait: it polls for the reply file — a fact the sweeping thread and the test both observe — behind a 10 s escape deadline that asserts rather than hangs, which is the prescribed shape. A targeted 16-worker stress and twenty concurrent full-suite runs could not make it fail. The suspect recorded there is thread creation or scheduling under a tarpaulin herd, which nothing in the test can answer.

Filed because three sightings is a pattern, not a fluke. Before editing anything, CAPTURE the failure: run the test family under `make check`-shaped load with `--nocapture` and `RUST_BACKTRACE=1` in a loop with a deadline until it fails, and put the assertion text and the timing in this body. Do not widen the deadline; if the 10 s is genuinely too short under a herd, the fix is a wait on the thread having STARTED (a channel handshake) rather than a longer clock.