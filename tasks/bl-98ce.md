+++
title = "two boundary::consume::tests::debris beats fail under full-suite parallelism and pass in isolation"
created = 1788484252
updated = 1788484538
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

---

Reproduced, root-caused and fixed. **The staleness candidate is dead: nothing in this path is wall-clock or mtime keyed.** `deposit::unheld` is `File::open(path).is_ok_and(|f| f.try_lock().is_ok())` — an `flock(LOCK_EX|LOCK_NB)` probe, no age anywhere. So no clock injection was needed and none was added.

## Capture

Four full-suite lib runs at once (`target/debug/deps/yog-*` whole, no filter, 16-core box), looping. Reproduced inside three batches, and the family is FOUR beats, not two:

- `boundary::consume::tests::debris::a_dead_claimants_gesture_is_answered_in_doubt_not_re_run` — `assertion left == right failed / left: 0 / right: 1` at debris.rs:19
- `boundary::consume::tests::debris::an_unwritable_in_doubt_reply_leaves_its_own_step_failure_row`
- `boundary::consumer::tests::the_boot_answers_a_dead_claimants_gesture_in_doubt` — `the boot never swept the debris` (it polls behind a 10 s deadline; the boot sweep runs ONCE, so a skip is permanent and no deadline can save it — which is why bl-5510 could not reproduce it by widening waits)
- `boundary::deposit::tests::a_claim_is_lock_held_for_the_claimants_life_and_unheld_after` — `a dropped claim is debris, tellably` at deposit/tests.rs:117

The last one is the smoking gun: no sweep, no count, just `drop(held); assert!(unheld(&path))`.

## The mechanism

Instrumented `unheld` to print the errno, the inode, the matching `/proc/locks` row and every descriptor in the tree pointing at the file. On the failing probe: `try_lock` answered `WouldBlock` while `/proc/locks` had ALREADY dropped the record — a holder that existed at the syscall and was gone microseconds later. `flock` belongs to the **open file description**, not to the descriptor: `fork` hands a child a second descriptor onto the same description, and `O_CLOEXEC` closes that copy at the child's `exec`, not at the fork. So `drop(Claim)` closed the claimant's descriptor and released **nothing** for as long as any child forked from any other thread during the claim's life had yet to exec — and a sweep in that window reads live work where there is debris.

Same family as the ETXTBSY hazard `git_env`'s module doc records, arriving from the other side: there a peer fork copies a WRITE fd, here it copies a LOCK fd. It is not a harness artifact — the engine forks constantly and from every thread, and the sweep is production code reading a production lock.

## Fix

At the root, in production, no test changed: **the release is an `unlock`, never a close.** `impl Drop for Claim` calls `File::unlock`, and `unheld`'s probe unlocks its own momentary hold before returning. `LOCK_UN` drops the lock off the description itself, so a descriptor copy in a not-yet-exec'd child holds nothing.

Regression beat lives in `deposit/claim.rs` (it needs the private descriptor): `try_clone` is a fork's descriptor copy, in-process and deterministic — clone the claim's lock, drop the claim, assert `unheld`. Verified to bite both ways: with the `unlock` removed it fails on exactly that assertion.

DESIGN §8.5 amended with the unlock rule beside the kernel-release sentence.
