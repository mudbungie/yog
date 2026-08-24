#!/bin/bash
# stories.sh — drive S0/S1 of docs/STORIES.md against a REAL yog + the live
# lernie wire (a real API call to the gpt-5.4 codex model). Layers on
# yogdrive.sh (same dir): the low-level primitive is an isolated Xvfb seat,
# claimed per run; this file is the STORY runner that seeds the world, fires the
# beats, and asserts on ops.jsonl + the on-disk workspace tree.
#
# The done-bar this proves is STORIES.md's second half — "the flow works against
# the real one": type a goal, hit Enter, the reply streams back and renders. It
# never touches the user's seat (input goes to this run's own display, claimed
# by `claim_seat` below and torn down by `verdict`).
#
# Usage: `seed <data>` lays the world seed only; `run` / `run-s3s4s6` /
# `run-s5s8` / `run-s7` each take `<data> <out>` (the `case` at the foot).
# Each run wants its OWN world (STORIES.md's "Four worlds, on purpose"), so each
# takes its own scratch dir; the beat bodies live in the sourced `beats_*.sh`,
# split only for this file's 300-line cap.
#
# Screenshots land in <out>/*.png for visual review; the run prints one
# PASS/FAIL line per beat and exits non-zero if any beat fails.
#
# STEERING RULE (STORIES.md "A gesture must not ride on pixels"). A coordinate
# is not a spelling — it is a measurement of a layout, and a row inserted above
# a target silently retargets every click below it (the drift that produced this
# rule; the drive log recording it was burned with docs/drive-logs/, bl-244f).
# So every beat drives a
# NAMED spelling, and there are exactly two, in this order: (1) the DESIGN §11
# keyboard binding, where the beat's subject is the window — its focus
# discipline, its selection, the verb the operator presses; (2) the DESIGN §8.5
# control boundary (`gesture`, in `gesture.sh`), where it is not. That is VISION §4.8's
# last consequence — "the drive harness stops steering pixels: story beats
# address the headless surface, and screenshots become what they always should
# have been, visual confirmation, not the transport." A §11 rule-2 *pick*
# ("which ready ball?") is a pick only at the pointer; at the line the thing
# picked has an address, so `/assign <id>` says it outright.
#
# A coordinate survives ONLY for a VIEW — focus, tab or mode selection, a pick
# with no address, a pin, a fold — because §8.5 gives views no boundary
# representation *by design*, so no spelling exists to prefer. Each is tagged
# `CLICK` where it fires.
#
# **And where one survives, it is DERIVED, not pinned (bl-b9f2, bl-5cce).**
# "Re-check the number against a screenshot when the layout changes" was this
# file's rule for three weeks and it failed three times (bl-2622 → bl-f8dc →
# bl-b9f2): the re-check only ever happens after a beat has gone red for a day —
# or never, if the beat's assertion is a negative a missed click satisfies, which
# is how three S7 coordinates spent twelve days on blank panel printing PASS. A
# pinned pixel is a second representation of a layout, and two representations of
# one fact drift. So a beat that must use the pointer reads its point off the
# frame it is about to drive: `locate.sh` has a surface per anchor — the §12
# Config column's rules, the centre's header/tab-strip rule, the §11 top bar's
# rule with the window's right edge, the window's bottom edge for the docked
# activity trail — and what stays written down is a distance INSIDE one widget,
# which only that widget's own contents can move. **There is no pinned pixel
# left anywhere in this harness**; a new one is a defect, not a shortcut.
#
# §11's table, which every windowed beat steers by:
#
#   ↓ ↑ roster    1-5 inspector tab (ctrl+1-5)   i composer focus (ctrl+i)
#   n new conversation (ctrl+n)   w new workspace (ctrl+shift+n)
#   s ▶ Start (top ready row)   x Stop   f Flush (scan)   c Close ball
#   r Release ball   b balls fold (ctrl+b)   g recent|by ball (ctrl+g)
#   a activity (ctrl+j)   Return send goal  Escape cancel goal
#
# A BARE binding is SUPPRESSED while a text box holds the keyboard, and §11's
# focus discipline parks the keyboard in the composer after EVERY operation, so
# the box holding it is the resting state, not the exception. Every bare key
# therefore goes through the `bare` verb (Escape, then the key) — Escape is what
# egui spends surrendering that focus, and `Escape` then `i` is that idiom named
# outright — and never `key`, which types a silent letter into the draft and
# looks like a missed beat. A COMBO is not suppressed (§11 rule 3: no combo
# fires a verb at the selection), so `ctrl+n` stays on `key`; so do Return and
# Escape, since a leading Escape would cancel a pending start goal.
#
# **This file carries no coordinate at all**: S0/S1 are the windowed rungs.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
drive="$here/yogdrive.sh"
real_world="$HOME/.local/share/yog/world/lernie"

# --- world seed (DESIGN §16.6 W3 marker + the codex provider rows) ----------
# Copy the real world's models.yaml (carries the gpt-5.4 codex entry) and
# template/providers.yaml (codex in both roles) into the scratch world. The
# models.yaml presence is lernie's seeded marker, so yog skips `lernie prime`
# (S0-T2 seeded-skip) — the general path with the seed present (§3.4).
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
  mkdir -p "$data/yog/world/lernie/template"
  cp "$real_world/models.yaml" "$data/yog/world/lernie/models.yaml"
  cp "$real_world/template/providers.yaml" "$data/yog/world/lernie/template/providers.yaml"
  seed_wall "$data" "$BOOTSTRAP_WS"
}

# --- the shared harness tier ------------------------------------------------
# The assertion helpers, the two waiting primitives (`await`, `until_landed`),
# the per-run seat and the verdict all live in `harness.sh` — none of them is
# about S0/S1, and every sourced beats_* file below calls them. It also carries
# the verdict's machine-keyable half, the `verdicts.jsonl` row each PASS/FAIL
# line now writes beside `gestures.jsonl` (bl-56d5), and it sources the two
# tiers of its own: the §16.2 wall fixture (`wall.sh`) and the §8.5 boundary
# transport with the engine it is aimed at (`gesture.sh` — `launch_engine`,
# `engine_alive`, `gesture`). The `gesture` helper lived HERE until bl-5cf7 gave
# it a deadline; it is no more about S0/S1 than `await` is, and it could not
# grow its watch where it sat, this file being at the 300-line cap.
. "$here/harness.sh"

# The per-world beat bodies, sourced rather than duplicated: each reuses `$drive`,
# `seed`, and every assertion helper above (and `in_world`/`seed_balls`, laid by
# the first). One file per world, because this file is at the 300-line cap —
# S0/S1's own body went the same way (`beats_s0s1.sh`), leaving this file the
# world seed, the tier sourcing and the verb dispatch.
. "$here/beats_s0s1.sh"
. "$here/beats_s3s4s6.sh"
. "$here/beats_s5_fixture.sh"
. "$here/beats_s5.sh"
. "$here/beats_s5_run.sh"
. "$here/beats_s8.sh"
. "$here/beats_s7.sh"
. "$here/beats_s6.sh"
. "$here/beats_s3res.sh"
. "$here/beats_headless.sh"
. "$here/beats_s13w.sh"
. "$here/beats_s19.sh"
. "$here/beats_s10.sh"
. "$here/beats_s18.sh"
. "$here/beats_unseeded.sh"

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
run)  shift; run "$1" "$2" ;;
run-s3s4s6) shift; run_s3s4s6 "$1" "$2" ;;
run-s5s8)   shift; run_s5s8 "$1" "$2" ;;
run-s7)     shift; run_s7 "$1" "$2" ;;
run-headless) shift; run_headless "$1" "$2" ;;
run-unseeded) shift; run_unseeded "$1" "$2" ;;
*) echo "usage: stories.sh seed <data> | run <data> <out> |" \
        "run-s3s4s6|run-s5s8|run-s7|run-headless|run-unseeded <data> <out>" >&2
   exit 1 ;;
esac
