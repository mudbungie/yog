+++
title = "unattended engine CD: reconcile the box against ghcr released tags, restart only when the boundary says idle"
created = 1788398596
updated = 1788398612
claimant = "Shipward"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
OPERATOR INSTRUCTION (2026-09-02): full CI/CD — engine boxes rebuild and redeploy without a human at the keyboard. This deliberately REVERSES the ruling recorded in scripts/deploy/seat.sh ('an upgrade is this script, run by a human') and must fix the doc, not deviate silently: amend seat.sh's comment block, DESIGN sec 10.1, and README's image section to state the new shape and why the old objections no longer hold.

The old reconciler died for two reasons and both now have answers:
1. 'No registry it may poll' — no longer true for RELEASES: the release workflow publishes the image to the ghcr package at tag time (DESIGN sec 10.1, immutable version tag + digest). The reconciler polls ONLY released tags; dev builds still travel by save|load through seat.sh, which remains the bootstrap/first-seat and emergency path.
2. Unattended restart kills an in-flight turn — so the restart defers on idleness asked OVER THE SEC 8.5 BOUNDARY (the old cgroup read is wrong for a container). Determine the right boundary read for 'no turn in flight' from the current roster; if none exists, adding one is in scope (it is a machine-class read per PARITY). Defer with a bounded retry cadence; never kill a running turn.

Shape: a timer unit + script installed by seat.sh alongside the engine unit — poll ghcr for the newest release version tag, compare to the running tag in deploy.env, pull, retag, rewrite deploy.env, reset-failed + restart when idle, then run verify.sh (the existing five-beat TLS proof) and roll back to the prior tag on a failed verify. Yank/supersede on ghcr stays the fleet-wide rollback lever. Auth: the package is public per DESIGN sec 10.1 — verify, and fail with a named remedy if a pull needs credentials.

Leak rules for everything committed: no hostnames, no addresses, no account names — box-specific facts stay in deploy.env or arguments.

Test per house rules: the script's decision logic (version compare, idle defer, rollback arm) must be drivable by the suite with fake curl/docker shims through the existing fake-substrate pattern — no live box or registry. Verify every premise against the tree first: seat.sh, verify.sh, yog.service, the release workflow, DESIGN sec 10.1.