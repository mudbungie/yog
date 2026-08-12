+++
title = "run-s7's first conversation dispatches before seed_wall, so its first prompt resolves providers against an empty wall"
created = 1786515941
updated = 1786515941
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