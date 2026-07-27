+++
title = "REGRESSION on main: bl claim fails under the embedded balls — 'no ball on the wire: neither command.id nor a sealed bl-id trailer (§7)'"
created = 1785131409
updated = 1785131409
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Caught by the drive harness on merged main b42af4d (bl-b5d1's batteries + W13), evidence in docs/drive-logs/2026-07-27-s5-s7-s8-wire-green.md.

REPRODUCE (no GUI needed beyond the runner):

    export PATH=<worktree>/target/release:$PATH
    scripts/drive/stories.sh run-s3s4s6 /tmp/b3 /tmp/outB3

9 of its 15 beats go red, all downstream of the first. The ops row:

    {"argv":["bl","claim","bl-90fe","--as","family-sassy"],
     "cwd":"<scratch>/proj","exit":1,
     "stderr":"bl-delivery: no ball on the wire: neither `command.id` nor a sealed `bl-id` trailer (§7)\nplugin bl-delivery aborted the op (exit status: 1)\nabort plugin bl-delivery aborted the op (exit status: 1)\nbl: plugin bl-delivery aborted the op (exit status: 1)"}

So the **whole ball rung is dead on main**: S3-T1 (claim), S3-T7 (close + delivered), S4-T2 (assign/release/move), S3-T4 (the close gate), and every S5/S8 beat that needs a workspace *bound to a project* (the marks knob renders with no project — which also re-exposes bl-9cb0).

WHAT IS AND IS NOT BROKEN (same build, same runner, same wire):
-  (S0/S1) — 8/8 PASS. /home/u/.local/share/lernie/workspaces/2b2f7fc4// are fine.
-  — 12/12 PASS (no  in that world).
-  — the §9 editors, both §8.4 hatches, the nesting/severability beats PASS; the 7 that need a claimed ball FAIL.
-  — 6/15 PASS; the 9 ball beats FAIL.

READING: the embedded balls'  plugin requires the claim op to carry a ball id ( or a sealed  trailer, balls arch §7) and yog's claim does not supply it — either the pinned balls rev moved ahead of yog's call contract or the multiplexed  path drops the id. Pre-batteries (main 81dafdd, host  on PATH) the identical runner and identical argv were green, twice, hours earlier: . So this is a phase-2 regression, not a fixture problem.

WHY IT LANDED UNSEEN: bl-b5d1's own drive proof (W14 clean room, docs/drive-logs/2026-07-26-w14-cleanroom.md) drove **S0/S1 only** — the two rungs that never touch .  takes any stories.sh verb, so the cheap guard against a repeat is to run  in the room as part of the batteries' done-bar.

FIX: in yog's  dispatch (src/actions/verbs, and the start flow's claim step) supply whatever the embedded balls now requires to identify the ball, or re-pin balls to a rev whose plugin contract matches. Then  and  must both go green again — they are the acceptance test and both are already written.