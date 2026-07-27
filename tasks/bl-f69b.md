+++
title = "wire evaluation blocked: installed bz 0.0.4 vs lernie linked brazen 0.0.3; the W5 gate is blind to it"
created = 1784955879
updated = 1785124227
claimant = "waxier-f69b"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found by the bl-8e07 real-substrate evaluation (docs/drive-logs/2026-07-24-s0-s1-wire-blocked.md).

WHAT WAS MISSING
The second half of the STORIES done-bar ("the flow works against the real
one") could not be evaluated on this machine: every conversation died before
the model call.

    $ LERNIE_HOME=<scratch>/lernie XDG_STATE_HOME=<scratch>/state \
        lernie prompt <ws> "Respond with exactly this text and nothing else: Manual wire OK."
    lernie prompt: bz version "0.0.4" does not match the linked brazen crate
    "0.0.3" (§4.4 — install the pinned binary: cargo install brazen --version =0.0.3)

Reproduced OUTSIDE yog, so a host-tuple condition, not a yog defect.

## STATUS — done (2026-07-26)

### 1. Tuple re-pinned FORWARD, and the wire is up

The installed `lernie` was simply stale against its own source: `/home/u/dev/lernie`
at `39c10a5` already pins `brazen = "=0.0.4"` (Cargo.lock resolves brazen 0.0.4),
so the coherent pair was the NEWER one — reachable by rebuilding lernie, not by
downgrading `bz`. Ran `make install` in the lernie repo:

- `~/.local/bin/lernie` replaced: mtime 2026-07-22 19:09 → 2026-07-26 12:17,
  `lernie 0.0.1` built from `39c10a5`, linking brazen 0.0.4.
- `~/.cargo/bin/bz` UNTOUCHED (`bz 0.0.4`, mtime 2026-07-25 01:53). lernie's own
  `install-bz` target reported `provider adapter: bz 0.0.4 already on PATH`.

Proof, the task's own repro on a scratch LERNIE_HOME: `lernie prompt` now exits 0
printing the branch name, no version refusal, and
`agents/…/messages/002-gpt-5.4.json` holds `[{"type":"text","text":"Manual wire OK."}]`
beside a 629-byte `response.json` (0 bytes before).

### 2. Wire evaluation RE-RUN — all eight beats PASS

`scripts/drive/stories.sh run` on the bl-20f4 keyboard-steered harness (merged
main `dfe1d28` into the worktree first — the run drove this worktree's own
`target/release/yog` via PATH prefix, on Xvfb :99, scratch world):

    S0 seeded-skip: no lernie prime                PASS
    S0 bare-start: lernie new fired                PASS
    S0 bare-start: detached lernie prompt          PASS
    S0 payoff: wire reply on disk                  PASS   <- was FAIL
    S1 message-to-agent: lernie message            PASS   <- was FAIL
    S1 restart: idle is pure (INV-1)               PASS
    S1 prompt-existing: no re-mint                 PASS
    S1 prompt-existing: new root agent             PASS
    ALL BEATS PASS

Three real model calls to `gpt-5.4`, all answered: `Wire check OK.` /
`Wire check OK.` / `Third wire OK.` on disk. `s0-04-transcript.png` is the first
drive screenshot in this repo showing a reply PAINTED in yog: header
`budget 993 tok (in 985 · out 8 · cache r 0 · w 0)`, the dim `gpt-5.4` origin line
above `Wire check OK.`, the conversation row titled by the reply and selected by
`key Down` alone, strip `nothing stirs`, `activity · 2 ops` zero ⚠.

Recorded in `docs/drive-logs/2026-07-26-s0-s1-wire-green.md`; STORIES.md's
real-substrate section now carries a "Where the second half stands: S0 and S1 are
GREEN" paragraph.

One honest observation, not a defect: the S1 message beat asked for
`Second wire OK.` and the model answered `Wire check OK.` again — it kept obeying
turn 1's "respond with exactly this text and nothing else". The beat asserts the
`lernie message` verb reached the EXISTING agent, which it did. Wording nit only.

### 3. DESIGN §16.6 W5 amended — the gate's blind spot, stated

Checked the freshly installed lernie: it has NO read-only verb reporting its
linked-crate versions. `lernie --version` prints `lernie 0.0.1` and nothing else;
the `--help` verb list (`new config prompt dispatch stop message scan bundle
replay advance tool prime`) has no version/doctor verb; and the guard's only
speaker is `check_bz_version` in `src/prompt/resolve.rs`, reached from the
MUTATING `prompt` path. So no probe can close the hole from yog's side.

W5 therefore gained a "What this gate does NOT cover: linked-crate skew between
two present, capable binaries" block that quotes the refusal verbatim, records
the no-verb finding at lernie `39c10a5`, states plainly that **W5 does not claim
this class**, and carries two owned consequences:

- UPSTREAM ASK (lernie): print the linked brazen pin as part of `lernie --version`
  (e.g. `lernie 0.0.1 (brazen 0.0.4)`). The gate already spawns `bz --version`;
  one extra read-only token on a call it makes anyway turns this class into an
  ordinary probe — compare two strings, classify Mismatch with
  `cargo install brazen --version =<pin>` beside it (the §8.3 pattern). W5 wires
  it the day it exists. **Not filed in lernie's tracker** — that is a lernie-repo
  ball and outside this ball's repo; whoever picks it up files it there.
- INTERIM COVERAGE IS RENDERING, NOT GATING: the §7.3 no-response wound (bl-7f2e,
  landed) plus the §8.1/§13.3 spawn-stderr sink (bl-a649, landed) make a skew that
  reaches dispatch a dead conversation WITH its cause named. Detection after the
  fact, not prevention before it — the accepted phase-1 position; phase 2 (§16.4)
  dissolves the class by making the pin definitional.

No gate wiring implemented in this ball, and no yog task filed: there is nothing
to wire until lernie exposes the token.

### 4. S2+ story / drive-coverage inventory (for follow-up dispatch)

Stories beyond S0/S1 in docs/STORIES.md, with their test rows:

| Story | Tests | Drive coverage in scripts/drive/stories.sh |
|---|---|---|
| S2 Director — point the conversation at a directory | S2-T1 | NONE |
| S3 Tracker — work rides a ball | S3-T1..T7 | NONE |
| S4 Organizer — spheres, corrals, and the board | S4-T1..T7 | NONE |
| S5 Operator — the tools and their configuration | S5-T1..T6 | NONE |
| S6 Triager — what needs you, and what went wrong | S6-T1..T5 | NONE |
| S7 Forensic — every byte inspectable | S7-T1..T5 | NONE |
| S8 Neighbour — yog beside your own shell | S8-T1..T4 | NONE |
| S9 Settler — the only thing you installed is yog | S9-T1..T4 | NONE (phase 2) |

`grep -c 'S[2-9]' scripts/drive/stories.sh` = 0. The harness drives S0 and S1 only;
S2+ drive beats are unwritten (deliberately not implemented here). STORIES.md's
"Drivable next" paragraph is the standing triage of what the harness could reach:
S3's close, S4's second conversation + grouped-by-ball toggle, and S6's attention
strip + activity chip need **no new machinery**; S5's editors, S7's inspector tabs
and S8's hatches are drivable but need world fixtures the runner does not lay (a
primed project, a config branch); S9 is not drivable until phase 2 exists.

Related: bl-7f2e (landed), bl-a649 (landed), bl-20f4 (landed, `dfe1d28`).