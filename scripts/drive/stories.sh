#!/bin/bash
# stories.sh — drive docs/STORIES.md against a REAL yog server and the live
# litany wire (a real API call to the gpt-5.4 codex model). It seeds the world,
# fires the beats through the §8.5 control boundary, and asserts on ops.jsonl +
# the on-disk workspace tree.
#
# The done-bar this proves is STORIES.md's second half — "the flow works against
# the real one": deposit a goal, the turn runs, the reply lands on disk.
#
# **Every beat is a §8.5 gesture** (bl-7942). There used to be two spellings and
# the harness said so at length: a DESIGN §11 keyboard binding where the beat's
# subject was the window, and the boundary where it was not — plus a `CLICK`
# escape for the views §8.5 deliberately gives no representation, each derived
# off the frame it was about to drive rather than pinned as a pixel. The window
# is the seat crate's now, so there is one spelling left, it is an address, and
# **no beat in this harness can steer a pixel** because nothing here paints one.
# That is VISION §4.8's last consequence arrived at in full: *"story beats
# address the headless surface"*.
#
# Usage: `seed <data>` lays the world seed only; `run-headless <data> <out>`
# fires the beats (the `case` at the foot). Each run wants its OWN world
# (STORIES.md's "one world per verb"), so it takes its own scratch dir; the beat
# bodies live in the sourced `beats_*.sh`, split only for this file's 300-line
# cap.
#
# The run prints one PASS/FAIL line per beat and exits non-zero if any fails.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
real_world="$HOME/.local/share/yog/world/litany"

# --- world seed (DESIGN §16.6 W3 marker + the codex provider rows) ----------
# Copy the real world's models.yaml (carries the gpt-5.4 codex entry) into the
# scratch world, and `template/providers.yaml` (codex in both roles) IF THE HOST
# HAS ONE. The models.yaml presence is litany's seeded marker, so yog skips
# `litany prime` (S0-T2 seeded-skip) — the general path with the seed present
# (§3.4).
#
# The two files are not the same kind of thing (bl-85ea). `models.yaml` is laid
# by any founded world; `template/providers.yaml` is the operator's INSTALL-WIDE
# OVERRIDE of the birth template's role rows, and nothing founds it — a world
# founded by a bare `yog` boot holds no `template/` directory at all. So a
# scratch world seeded on a box without one births on the shipped role rows,
# which is exactly what a fresh install does.
#
# The BOOTSTRAP SPHERE'S WALL is the third file of the same seed (bl-1851), not
# a later step. The template above names `openai-chatgpt`, and a newborn wall
# ships no such row (§16.2: "a newborn workspace therefore answers brazen's
# shipped rows and nothing else") — so the row and the sign-in that answers for
# it have to be on disk before the first Enter, or every run's first turn dies
# `unknown provider`. Both halves of one fact, one place. `harness.sh`'s wall
# section carries why the leaf is knowable here and why laying it after the mint
# is always too late.
seed() {
  data=$1
  mkdir -p "$data/yog/world/litany/template"
  cp "$real_world/models.yaml" "$data/yog/world/litany/models.yaml"
  # The override is copied ONLY where the host has one (bl-85ea). No founded
  # world creates it, so an unconditional copy under `set -e` made a file
  # nothing lays a hard prerequisite of every scratch world — and through
  # preflight's required tier, of the whole drive family.
  if [ -f "$real_world/template/providers.yaml" ]; then
    cp "$real_world/template/providers.yaml" \
      "$data/yog/world/litany/template/providers.yaml"
  fi
  seed_wall "$data" "$BOOTSTRAP_WS"
}

# --- the shared harness tier ------------------------------------------------
# The two waiting primitives (`await`, `until_landed`) and the verdict live in
# `harness.sh`, and every sourced beats_* file below calls them. It also
# carries the verdict's machine-keyable half, the `verdicts.jsonl` row each
# PASS/FAIL line writes beside `gestures.jsonl` (bl-56d5), and it sources the
# three tiers of its own: the assertion helpers (`predicates.sh`), the §16.2
# wall fixture (`wall.sh`) and the §8.5 boundary transport with the engine it
# is aimed at (`gesture.sh` — `boot_engine`, `engine_alive`, `gesture`).
. "$here/harness.sh"
# How a run boots an engine and reads what it said (`boot_headless`, `reply_is`,
# `row`).
. "$here/headless.sh"

# The beat bodies, sourced rather than duplicated: each reuses `seed` and every
# assertion helper above. One file per story, because this file is at the
# 300-line cap — which leaves this file the world seed, the tier sourcing and
# the verb dispatch.
. "$here/beats_headless.sh"
. "$here/beats_s10.sh"
. "$here/beats_s11.sh"
. "$here/beats_s13.sh"
. "$here/beats_s18.sh"
. "$here/beats_s19.sh"

# Everything above is now sourced into ONE flat bash namespace, so this is the
# instant the namespace is complete and the only instant the collision guard can
# run. The guard itself lives with the other shared primitives (harness.sh,
# `one_name_one_definition`) — it is a property of that namespace, not of S0/S1.
one_name_one_definition "$here"

# The verb, kept for the verdict rows: one world per verb (STORIES.md's "Four
# worlds, on purpose"), so the verb is what tells two runs' rows apart in a
# collected ladder. Read by `record` in harness.sh.
drive_run=${1:-}
case ${1:-} in
seed) shift; seed "$1" ;;
run-headless) shift; run_headless "$1" "$2" ;;
*) echo "usage: stories.sh seed <data> | run-headless <data> <out>" >&2
   exit 1 ;;
esac
