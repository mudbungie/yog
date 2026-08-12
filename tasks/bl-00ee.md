+++
title = "run-s5s8 fixture dies at the bl-c3a9 birth gate: its scratch BRAZEN_CONFIG has no provider rows, but the seeded template names openai-chatgpt"
created = 1786162686
updated = 1786511432
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["drive"]
+++
Found by the 2026-08-07 re-baseline drive (bl-c63e), docs/drive-logs/2026-08-07-ladder-rebaseline.md. Reproduced identically on two runs of `scripts/drive/stories.sh run-s5s8` against release `aafa438`. **Ten beats red, one cause.**

    S5 fixture: ball bound, no wire spend          FAIL — no clean claim row
    S5-T5 config-branch: lernie config exit 0      FAIL — no clean config row
    S5-T5 config-branch: staged file lands on config/default FAIL — branch/file absent
    S5-T5 config-branch: descriptions/ survived the staged copy FAIL — checkout files lost
    S5 brazen: Apply lands (the bracket for the negatives) FAIL — marker not on disk
    S5-T4 hash-guard: reload then the same Apply lands FAIL — marker not on disk
    S3-T2 new ball: bl create from the boundary    FAIL — no clean create row
    S3-T2 new ball: converges to exactly one claim FAIL — claims=0
    S4-T2 release: bl unclaim --as the claimant    FAIL — no clean unclaim row
    S3-T4 close gate: hook stderr verbatim in the ops row FAIL — no failed close row

`<world>/yog/workspaces` NEVER EXISTS in this run — the harness even prints `find: .../yog/workspaces: No such file or directory`. The world is not damaged; yog REFUSED to build it, and said why. From `ops.jsonl`, once per retry:

    {"argv":["yog-step","template"],"exit":-3,"origin":"balls","stderr":
     "the workspace-birth template <world>/yog/world/lernie/template/providers.yaml
      names provider rows brazen s table does not have (worker -> `openai-chatgpt`,
      compactor -> `openai-chatgpt`) — every workspace born from it dies at its first
      dispatch, so this one was not created; add the row in the brazen config editor,
      or repoint the template"}

## The refusal is correct and NEW — the fixture is stale

The gate is the §9.2 workspace-birth gate added by `27910af` "world template/providers.yaml births every new workspace, but no yog surface can see, edit, or validate it [bl-c3a9]":

- `src/start/exec.rs:36` — "The §9.2 gate over the world s workspace-birth template (bl-c3a9)."
- `src/boundary/dispatch/deps.rs:41` — "brazen s effective table (`bz --list-providers`, asked at the shell boundary), which the world s birth template is gated against before a workspace is created ([`template::gate`]). Empty is no answer and gates nothing — §9.2 s rule at every site."

The fixture contradicts itself on this sha:

1. `scripts/drive/stories.sh` `seed()` copies the REAL world s `template/providers.yaml`, whose rows name `openai-chatgpt`.
2. `scripts/drive/beats_s5.sh:22` then overrides brazen s config with a scratch file holding NO PROVIDER ROWS AT ALL — the function writes exactly one comment line:

       brazen_scratch() {
         printf "# yogdrive scratch brazen config (BRAZEN_CONFIG override)\n" > "$1"
       }

   and `beats_s5.sh:47` exports it: `export BRAZEN_CONFIG="$bzcfg"`.

## The override is right; it is just now incomplete

Keep it. Its reason is stated at `beats_s5.sh:13` and still holds: "brazen s config is ambient by design (§16.2), so an Apply beat would otherwise edit the operator s own `~/.config/brazen/config.toml` — the machine it is testing on."

The repair is to give the scratch config the row its own seeded template names — an `openai-chatgpt` provider entry — or to repoint the seeded template at whatever the scratch config does carry. The ambient config has the row (`~/.config/brazen/config.toml:5`, `name = "openai-chatgpt"`), which is why every OTHER world in the same drive births fine and only this one dies.

Note the beat comment at `beats_s5.sh:19` claims the scratch config is "A tiny, VALID brazen config"; valid for `bz --dump-config` is no longer sufficient now that the birth gate reads `bz --list-providers`. Update that comment with the fix.

## Not affected

Sixteen beats in this script still pass, including the whole S8 nesting group and `S8-T1 severability: rm -rf the world, ambient intact`.

---

Reproduced on `96d5f4e` via `make drive DRIVE_ROOT=/tmp/yog-sparrow-nowire DRIVE_RUNS=run-s5s8`: 17 beats pass, the same 10 listed here fail. The diagnosis in this body is now stale after bl-c0e2: brazen no longer reads ambient `BRAZEN_CONFIG`; config and credentials live in each workspace wall. bl-49c6 now tracks repairing the harness at that new authority. Keep this ball as the exact red-table witness, but do not implement the retired ambient-provider fix.

---

Diagnosis correction from bl-49c6 (this ball's stated cause is retired; the failure is real and now named at preflight).

This ball says the fixture's 'scratch BRAZEN_CONFIG has no provider rows'. There is no scratch BRAZEN_CONFIG any more — bl-c0e2 moved brazen's config, credentials and model cache inside the per-workspace wall (`<world>/walls/<name>/brazen/*`, DESIGN §16.2 as amended), `bz_host::run` injects the wall's path as BRAZEN_CONFIG so brazen never reaches its own fall-through, and `beats_s5.sh` was reseated onto the wall. An exported BRAZEN_CONFIG is inert.

The 10-red table is still real, and the cause is one layer over: a NEWBORN workspace's wall is an empty directory, so its brazen table is brazen's seven shipped rows and nothing else. Measured, not assumed:

    $ YOG_WALL=$(mktemp -d) yog bz --list-providers --json
    anthropic, openai, mistral, openai-responses, google, ollama, claude-code

The world seed copies the operator's `world/lernie/template/providers.yaml`, which names `openai-chatgpt` in both roles. That row lives only in the operator's ambient `~/.config/brazen/config.toml`, which no scratch wall inherits. `start::ensure::create_workspace` runs `template::gate` BEFORE `lernie new`, and against the TARGET workspace's wall (`boundary/dispatch.rs`: 'The §9.2 birth-template gate reads the **target** workspace's providers'). So the gate refuses with DeadRows, no workspace is created, and every beat goes red. template.rs's 'an empty provider table gates nothing' escape does not fire: the table is not empty, it is the defaults.

The ordering is the crux and it is why bl-49c6 could only report it: the wall is keyed by the §3.1 leaf yog MINTS at the fire, so no harness fixture can exist before the gate reads it. `harness.sh`'s new `seed_wall` lays the wall (host config + credentials) right after each mint — which fixes every post-birth brazen surface and the wire — but cannot precede birth.

Two exits, and this ball owns the choice:
(a) FIXTURE — seed a birth template naming a row a fresh wall already carries. Workspaces are born and the world renders; the wire then depends on a credential for that row (the host has `google.json` and `openai-chatgpt.json`; `google` is a shipped row, `openai-chatgpt` is not). Cheapest, no Rust.
(b) GATE — the §9.2 birth gate's premise died with the sharing it was designed under (bl-c3a9 predates the wall). It judges a template against a wall that cannot exist yet, so it refuses on the strength of a question that cannot have been answered — the same posture template.rs already takes for an unreadable file. This is a DESIGN §9.2 amendment plus a Rust change, not a silent tweak.

Until one lands, `make drive-preflight` names it at step 0 with the fix in the message, instead of the ladder discovering it a beat at a time.
