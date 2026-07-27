+++
title = "REGRESSION on main: bl claim fails under the embedded balls — 'no ball on the wire: neither command.id nor a sealed bl-id trailer (§7)'"
created = 1785131409
updated = 1785131735
claimant = "waxier-seam"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Caught by the drive harness on merged main b42af4d (bl-b5d1's batteries + W13's gate deletion). Evidence: docs/drive-logs/2026-07-27-s5-s7-s8-wire-green.md.

REPRODUCE (the runner is the whole repro):

    export PATH=<worktree>/target/release:$PATH
    scripts/drive/stories.sh run-s3s4s6 /tmp/b3 /tmp/outB3

9 of its 15 beats go red, all downstream of the first. The ops row:

    {"argv":["bl","claim","bl-90fe","--as","family-sassy"],
     "cwd":"<scratch>/proj","exit":1,
     "stderr":"bl-delivery: no ball on the wire: neither `command.id` nor a sealed `bl-id` trailer (§7)\nplugin bl-delivery aborted the op (exit status: 1)\nabort plugin bl-delivery aborted the op (exit status: 1)\nbl: plugin bl-delivery aborted the op (exit status: 1)"}

So the WHOLE BALL RUNG is dead on main: S3-T1 (claim), S3-T7 (close + the delivered re-derivation), S4-T2 (assign / release / move), S3-T4 (the close gate), and every S5/S8 beat that needs a workspace bound to a project — the marks knob then renders with no project at all, which also re-exposes bl-9cb0.

WHAT IS AND IS NOT BROKEN (same build, same runner, same wire, same hour):
- `run` (S0/S1) — 8/8 PASS. lernie new / prompt / message are fine.
- `run-s7` — 12/12 PASS (that world spawns no bl).
- `run-s5s8` — the three §9 editors, both §8.4 hatches and the nesting/severability beats PASS; the 7 that need a claimed ball FAIL.
- `run-s3s4s6` — 6/15 PASS; the 9 ball beats FAIL.

READING: the embedded balls' bl-delivery plugin now requires the claim op to identify its ball (`command.id` or a sealed `bl-id` trailer, balls arch §7) and yog's claim does not supply it — either the pinned balls rev moved ahead of yog's call contract, or the multiplexed `bl claim` path drops the id on the way. Hours earlier, pre-batteries (main 81dafdd, host bl on PATH), the identical runner and identical argv were green twice: {"argv":["bl","claim","bl-7522","--as","bucket-flashily"],"exit":0}. So this is a phase-2 regression, not a fixture problem.

WHY IT LANDED UNSEEN: bl-b5d1's own drive proof (W14 clean room, docs/drive-logs/2026-07-26-w14-cleanroom.md) drove S0/S1 ONLY — the two rungs that never touch bl. `cleanroom.sh` passes any stories.sh verb straight through, so the cheap guard against a repeat is to run `run-s3s4s6` (and now `run-s5s8`) in the room as part of the batteries' done-bar.

FIX: in yog's claim dispatch (src/actions/verbs and the start flow's claim step) supply what the embedded balls requires to identify the ball, or re-pin balls to a rev whose plugin contract matches. Then `run-s3s4s6` and `run-s5s8` must both go green again — they are the acceptance test, and both are already written.

STATUS (2026-07-26, waxier-seam): ROOT CAUSE CONFIRMED AND FIXED — a plugin-wire protocol skew, exactly the 'pinned balls rev moved' reading. balls bl-a5f3 (upstream b2041b2b) changed the §7 wire: the payload's command now carries the ball id (command.id) and bl-delivery REQUIRES it (resolve_id: 'no ball on the wire'). The drive's world was primed by the HOST bl (~/.local/bin, rebuilt 19:56 from post-a5f3 source — the bl-44a5 fallthrough), binding the checkout's config/plugins/bin symlinks to the NEW host bl-delivery; yog's embedded balls at pin 15b50589 (pre-a5f3, Command has no id field) then drove claim with the OLD wire and the new plugin refused. FIX: re-pin balls to 9ee8d016 (branch yog-delivery-bin-pin; carries a5f3's wire plus the U-balls-3 delivery_bin::run lib seam for bl-2930). Verified headless on the exact repro shape: host-primed checkout + 'yog bl claim' exit 0 with worktree printed, 'yog bl close' delivers the tagged squash. Full runner acceptance (run-s3s4s6 15/15, run-s5s8 ball rows) recorded in the arc's drive log with bl-2930/bl-44a5, which make the rung self-contained (no host plugin binding at all).