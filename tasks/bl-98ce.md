+++
title = "two boundary::consume::tests::debris beats fail under full-suite parallelism and pass in isolation"
created = 1788484252
updated = 1788484277
claimant = "Spellbind-T"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Sighted once during bl-54c1, on a tree whose only change was a refusal in `boundary::dispatch::doors::prompt` — nothing that touches the deposit inbox.

`cargo test` (whole suite, default parallelism) failed two of bl-d1f1's new beats:

- `boundary::consume::tests::debris::a_dead_claimants_gesture_is_answered_in_doubt_not_re_run` — src/boundary/consume/tests/debris.rs:19, `assert_eq!(sweep(root.path(), "T1"), 1)`
- `boundary::consume::tests::debris::an_unwritable_in_doubt_reply_leaves_its_own_step_failure_row` — same file, line 93

Both passed on three consecutive isolated runs (`cargo test --lib boundary::consume::tests::debris`) and on an immediately following full `cargo test`, which was green end to end. So the sweep's count is load- or timing-sensitive rather than wrong: each beat deposits into its own tempdir, claims through the real claim and drops the guard, then asserts the sweep answers exactly one piece of debris.

Two candidates worth reading before changing anything: whether the claim marker's staleness test is wall-clock- or mtime-keyed (a loaded box would then let a just-dropped claim read as not-yet-debris, giving 0), and whether `sweep` can see a slot mid-write. A count assertion that can read 0 under load is the shape that lands a permanent FAIL verdict in the merge queue, which is why this is worth a look rather than a re-run.

Not investigated further — out of scope for bl-54c1, and the tree it was sighted on is green.

---

Folded in from bl-fc72 (closed as the duplicate): the same family failed under load for three other agents today — the boot-side beat `the_boot_answers_a_dead_claimants_gesture_in_doubt` once in a close gate under a tarpaulin herd, and the `debris` counts twice more in full-suite runs beside concurrent gates. bl-5510 examined the boot-side beat and could not make it fail under a targeted 16-worker stress or twenty concurrent suites: its wait already polls for the reply file behind a 10 s deadline that asserts rather than hangs. So the reproducible half is likely the `debris` counts named here, and the staleness candidate is the one to check first: if a claim reads as debris only after a wall-clock age, a loaded box makes the just-dropped claim not-yet-debris and the sweep honestly answers 0.
