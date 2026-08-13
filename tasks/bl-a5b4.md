+++
title = "run-s7's first conversation dispatches before seed_wall, so its first prompt resolves providers against an empty wall"
created = 1786515941
updated = 1786600615
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["drive"]
+++
Found by Fretwork 2026-08-12 while verifying bl-52c7 on the wire
(`make drive DRIVE_RUNS=run-s7`, tree `cbfd4c1`). **Not bl-52c7's beat** — that
one passes now; these are its two neighbours.

## The two reds

    S7 fixture: wire reply on disk                 FAIL — no gpt reply in 40s
    S6 stop: lernie stop dispatched                FAIL — no stop verb for <agent-id>

Both were **PASS** on the 2026-08-07 re-baseline
(`docs/drive-logs/2026-08-07-ladder-rebaseline.md`), so this is new decay. It is
also not bl-2d45: that ball's fix (`7a91ed6`) IS an ancestor of the tree run
here, verified with `git merge-base --is-ancestor`.

## The evidence

Two conversations were born in the same run. The **first** failed and a **later**
one succeeded, on the same wall, in the same world:

    steps/<agent-id>/001/response.json
      {"type":"error","kind":"config","message":"unknown provider `openai-chatgpt`"}

    steps/<agent-id>/001/response.json
      {"type":"message_start","v":1,...,"model":"gpt-5.4","role":"assistant"}

The wall's own config is fine and does declare the row —
`<world>/walls/home/brazen/config.toml` carries `[[provider]] name =
"openai-chatgpt"`. So this is not a missing config; it is a config that arrived
**after** the dispatch that needed it.

## The ordering, in `scripts/drive/beats_s7.sh`

    until_landed bare_start verb_ge prompt 1     # starts the conversation - PASSES
    ws_root=$(ws_here "$data")
    seed_wall "$data" "$ws_root"                 # the wall is seeded HERE
    await reply_exists                           # then the reply is awaited - FAILS

The start mints the workspace and dispatches its first prompt; only then does
`seed_wall` copy brazen's `config.toml` into `<world>/walls/<workspace>/brazen/`.
The first dispatch therefore resolves providers against a wall that does not yet
have any, and the run's `S6 stop` beat then fails downstream because its target
is precisely that dead conversation — nothing live to stop.

**Why now.** The run's own preflight advisory states the mechanism: *"since
bl-00ee retired the §9.2 birth gate a workspace is born whatever its template
names, and a row its wall lacks surfaces at the first dispatch (§8.3), not as a
refusal to create anything."* The gate used to make this ordering safe by
refusing; with it retired, the race is live and the fixture is what has to
sequence it.

## The real constraint, so this is not filed as a one-line reorder

`seed_wall` needs `ws_root`, and `ws_root` does not exist until the start has
minted it — so "seed before starting" is not available as written. The fix has
to seat the wall **between the mint and the first dispatch**, or make the first
prompt retriable once the wall lands. Decide which; do not just move the line.

## Scope

- Verify the sequence at HEAD before editing — this body is a reading of one run.
- The same shape may exist in the other `beats_*.sh` runs that call `seed_wall`
  after a start; check them rather than fixing S7 alone.
- `run-s3s4s6` reportedly passes all 18 beats, so whatever it does with the wall
  is the pattern worth copying.

---

Delivered by bl-1851 (`c248f9a` on work/bl-1851), whose subject is the same defect one rung up: it is not run-s7's alone — `stories.sh run` (S0) had it too, and that is where it was found. **The primary red is fixed; the S6 stop red is NOT downstream of it, and this ball's diagnosis of that half is falsified.**

## The third option this body did not see

The body says: *"`seed_wall` needs `ws_root`, and `ws_root` does not exist until the start has minted it — so 'seed before starting' is not available as written. The fix has to seat the wall between the mint and the first dispatch, or make the first prompt retriable once the wall lands."* Neither was needed. A wall is keyed by a NAME, not by a workspace that exists, and the empty-world start's name is not minted at all — DESIGN §3.1: *"The bootstrap names without asking. The empty-world start (§3.4) creates its workspace under the fixed default name `home` — a constant, not a config … and not a mint."* Every run verb in the harness opens on zero workspaces, so every wall it lays is `home`'s and is layable before yog is launched. `seed_wall` is now keyed by a name (`BOOTSTRAP_WS`) and called from `stories.sh`'s `seed`, beside models.yaml and providers.yaml; the three post-mint call sites (run, run_s3s4s6, run_s7) are gone. DESIGN §16.2 amended, since its stated reason ('that leaf is minted by the birth') is true only of a §11 `w` sphere.

## Evidence, same binary, same box, minutes apart

    pre-fix scripts (git checkout HEAD~1 -- scripts/drive/):
      S7 fixture: wire reply on disk                 FAIL — no gpt reply in 40s
    post-fix:
      S7 fixture: wire reply on disk                 PASS

Historically this beat was red on all three run-s7 drives of 2026-08-12 (05:38, 06:36, 06:40) and is green now.

## The half this ball got wrong

> the run's `S6 stop` beat then fails downstream because its target is precisely that dead conversation — nothing live to stop.

It is not downstream. In the post-fix run the wire beat PASSES and `S6 stop: lernie stop dispatched` still FAILS (`no stop verb for <agent-id>`). The screenshot `s6-07-inflight.png` (/home/u/.cache/yog-drive/20260813T042347Z/run-s7/out/) shows why: the highlighted roster row is the **laid child** (the indented `↳ happiest`), not the root `$agent` that `stopped "$flight"` names — `s7_descent` runs immediately before `s6_attention` and leaves the selection on a member, so `x` stops the child. It is also flaky rather than constant (it passed on the 06:40 run of 2026-08-12), which fits a race against the one-line reply going quiescent. So this residue is a live, separate stale-beat: an S6 stage inheriting a selection an S7 stage moved. Re-scope this ball to it, or re-file.
