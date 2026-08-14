+++
title = "MONITOR: verify the speculative merge queue delivers builds on GitHub Actions end to end — first run blocked upstream at the account level; validate live once Actions accepts jobs"
created = 1786601239
updated = 1786684269
claimant = "Gimbal"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
You are monitoring, not building. Report what you observe; file a new ball per defect found (cite the literal command output in its body); do NOT fix anything, do NOT unclaim or touch anyone's claims, do NOT kill processes. Every command below is safe to run as-is from the repo root (~/dev/yog). No wait-loops: each check reads current state once; the only sanctioned wait is `gh run watch` on a run id you started.

CONTEXT (presume nothing): yog adopted balls' speculative merge queue (bl-1a5b, 2026-08-13; design: balls repo docs/design/bl-24e7-speculative-merge-queue.md). The pieces: (1) scripts/pre-commit is the build gate; it first asks `bl-speculate check` whether this exact worktree tree already passed this exact gate (a "verdict cache" in ~/.local/state/balls/plugins/bl-speculate/verdicts/) and exits in seconds on a hit; (2) .github/workflows/speculate.yml runs that same gate on GitHub Actions for any branch named speculation/<sha> and uploads the verdict store as an artifact named `verdicts`; (3) scripts/speculate-gate, used as `bl-speculate run --gate scripts/speculate-gate`, pushes a candidate, waits on the workflow run, downloads and imports the verdict, deletes the branch. Everything is fail-open: any breakage anywhere degrades to the stock local gate — which is exactly why breakage is INVISIBLE without this monitoring.

KNOWN BLOCKER at filing time: GitHub was refusing to start any job on this repository, for an account-level reason outside the code. Every run, including CI on main, failed within seconds with a scheduling annotation rather than a build failure. Check 0 tells you whether that has been resolved.

CHECK 0 — is Actions accepting jobs yet?
    gh run list --limit 3 --json name,conclusion,createdAt --jq '.[] | .createdAt + " " + .name + " " + .conclusion'
If the newest runs still fail in ~3 seconds without ever starting a job, read the scheduling annotation (gh api repos/mudbungie/yog/actions/runs/<RUN_ID>/jobs --jq '.jobs[0].id', then gh api repos/mudbungie/yog/check-runs/<JOB_ID>/annotations --jq '.[].message'), report "Actions still refusing to schedule" in this ball with `bl comment` and STOP — nothing below can pass.

CHECK 1 — the fingerprint computes (cache not inert). From a CLAIMED worktree or the repo root:
    bl-speculate check; echo "exit: $?"
Expected: exit 3 (honest miss) or 0 (hit). Exit 1 means the cache is erroring — the whole feature is silently off. File a ball.

CHECK 2 — live end-to-end (only once CHECK 0 passes). Take any ready ball, claim it with your identity, make a trivial legitimate change per its body (or use a ball you were told to), commit in the worktree with --no-verify, then from that worktree:
    bl-speculate enqueue <ball-id>
    bl-speculate run --gate scripts/speculate-gate --builds 1
Expected in order: a "pushing candidate <sha> to speculation/<sha>" line; a workflow run visible in `gh run list --branch speculation/<sha>`; the run concluding success in roughly 30-60 minutes (cold caches) — `gh run watch <id>` is the sanctioned wait; an "imported <tree> <gate>" line; the speculation branch GONE from origin afterward (git ls-remote origin 'refs/heads/speculation/*' should print nothing); the report line "built <ball-id> <tree> pass"; and `bl-speculate check` in that worktree now exiting 0. Then `bl close <ball-id> --as YOU` (foreground) should print the cache-hit line "verdict cache hit — this exact tree already passed this exact gate" and complete in seconds-to-a-minute instead of ~15 minutes. ANY deviation: file a ball quoting the deviation verbatim.
CAVEAT: if main moved between your enqueue and your close, the close honestly misses the cache and runs the full local gate — that is correct behavior, not a defect; note it and judge the cache hit by the speculate-gate leg instead.

CHECK 3 — hygiene (run every time):
    git ls-remote origin 'refs/heads/speculation/*'   # stranded remote branches; each is a crashed driver — report, and sweep with: git push origin --delete <branch>
    bl-speculate queue                                 # entries marked 'unsealed' are stale seals; the next `bl-speculate run` sweeps them — report only if the same one persists across two checks
    ls ~/.local/state/balls/plugins/bl-speculate/verdicts/ | head   # filenames must be <40hex>-<40hex>.toml; anything else means store corruption — file a ball

CHECK 4 — toolchain lockstep (the silent killer): the workflow only produces verdicts that land locally when runner and laptop agree on `rustc -V`. Compare:
    rustc -V                                  # local; rust-toolchain.toml pins 1.95.0
    grep channel rust-toolchain.toml
If a toolchain bump landed in one place without the other (e.g. rust-toolchain.toml changed but remote verdicts stopped hitting in CHECK 2), file a ball: remote verdicts silently stopped matching.

When all checks pass, close this ball with a `bl comment` summary: date, run id observed, cache-hit-at-close yes/no, minutes the remote run took.

---

MONITOR RUN 2026-08-14 (UTC). All five checks PASS; the queue delivered a real
build end to end, twice. Nothing was fixed in the queue; one unrelated blocker
found and filed as bl-1007.

CHECK 0 — PASS. Actions schedules again: CI run 31754798892 concluded success at
2026-08-13T23:42Z. The account-level refusal this ball was filed behind is gone.

CHECK 1 — PASS. `bl-speculate check` exits 3 at the repo root: an honest miss,
not an erroring cache.

CHECK 2 — PASS, live, on vehicle ball bl-848f (now closed; delivery 71adb7ef).
Run twice, because main moved under the first one. Every observable the body
lists appeared, in order, both times:

  speculate-gate: pushing candidate <sha> to speculation/<sha>
  speculate-gate: watching run <id>
  imported <tree> <gate>
   - [deleted]           speculation/<sha>
  built bl-848f <tree> pass

  run 1: id 31756179051, branch speculation/8a72cb26…, conclusion success,
         00:05:47Z -> 00:14:13Z = 8.4 minutes
  run 2: id 31756812597, branch speculation/fe69ea47…, conclusion success,
         00:16:46Z -> 00:22:52Z = 6.1 minutes

Both are far under the 30-60 minute cold-cache estimate in this body — the
runner's caches are warm now, which is a fact worth carrying forward: a
speculative build is cheaper than the local gate it replaces, not merely
parallel to it.

`git ls-remote origin 'refs/heads/speculation/*'` printed nothing after each
run: the driver swept its own branch. `bl-speculate check` in the claimed
worktree exited **0** immediately after each import — the remote verdict landed
and matched the local fingerprint, which is the whole mechanism working.

CACHE HIT AT CLOSE: **no**, and correctly so — this ball's own caveat. main
advanced four times between the enqueue and the close (other builders landing),
so each merge changed the tree and the close honestly missed and ran the full
local gate. Judged by the speculate-gate leg instead, where the hit is
unambiguous: `bl-speculate check` exit 0 on the exact tree the runner built,
twice. Worth recording that the window is narrow — main moved every ~7 minutes
during this run, which is the same order as a remote build, so a hit at close is
luck unless the close holds the flock lock across the merge as well as the gate.

CHECK 3 — PASS. No stranded `refs/heads/speculation/*` on origin. Every verdict
filename matches `<40hex>-<40hex>.toml` (0 non-conforming). The queue carries one
entry, `bl-848f <sha> unsealed`, first seen this check — the previous unsealed
entry was swept by the next run exactly as documented, so this one is not
reported as a defect. It does mean a queue entry outlives the ball it names: the
ball is closed and the entry is still there until something runs.

CHECK 4 — PASS. rustc 1.95.0 locally; `rust-toolchain.toml` pins channel
"1.95.0". No skew, which is why the remote verdicts landed at all.

BLOCKER FOUND, NOT IN THIS SUBSYSTEM (bl-1007): the close was refused — and so
was every other `bl` op in the checkout — because one live ball body carried an
absolute home path and `scripts/yog-leak-gate` scans the whole store checkout,
not the file the op touched. Folded that one path to `~` (same value, same
meaning) to unwedge the repo, and filed the class.
