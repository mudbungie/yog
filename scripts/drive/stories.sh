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

# --- the run ----------------------------------------------------------------
run() {
  data=$1 ; out=$2
  mkdir -p "$out"
  rm -rf "$data" ; mkdir -p "$data"
  claim_seat
  seed "$data"
  ops="$data/yog/world/state/yog/ops.jsonl"

  # S0 — bare start: launch, type a wire-check goal, Enter.
  launch_engine "$data" ; pid=$engine_pid ; wid=$engine_wid
  sleep 1
  "$drive" shot "$wid" "$out/s0-01-launch.png"
  # Escape then `i` — §11's composer-focus idiom (the STEERING RULE above): the
  # bootstrap box opens focused via a once-only request_focus, so Escape makes
  # the state deterministic (a bare `i` into a focused box types an `i` into the
  # goal) and `i` then asks for the focus back.
  bare_start() {
    "$drive" bare "$wid" i
    "$drive" type "$wid" "Respond with exactly this text and nothing else: Wire check OK."
    "$drive" shot "$wid" "$out/s0-02-typed.png"
    "$drive" key "$wid" Return
  }
  until_landed bare_start verb_ge new 1 \
    && pass "S0 bare-start: lernie new fired" \
    || fail "S0 bare-start: lernie new fired" "no new"
  "$drive" shot "$wid" "$out/s0-03-fired.png"

  # Locate the workspace the fire minted (single dir under yog/workspaces) — a
  # READ of what happened, which is why it is a `find` and not `$BOOTSTRAP_WS`:
  # the wall the seed laid is a path this harness chose, the workspace is a fact
  # yog produced, and a beat must never assert the second by restating the first.
  # Its wall was seeded before the launch (§16.2, bl-1851 — see `seed` above): the
  # mint and the first `lernie prompt` are one gesture, so nothing laid here could
  # beat the first model call, and the payoff beat below is what that cost.
  ws_root=$(find "$data/yog/workspaces" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | head -1)

  [ "$(verb_count prime)" = 0 ] \
    && pass "S0 seeded-skip: no lernie prime" \
    || fail "S0 seeded-skip: no lernie prime" "prime spawned"
  [ "$(verb_count prompt)" -ge 1 ] \
    && pass "S0 bare-start: detached lernie prompt" \
    || fail "S0 bare-start: detached lernie prompt" "no prompt"
  # Poll, never a fixed sleep: the reply's latency is the model's, not yog's, so
  # the 3 s above is enough to assert the *spawns* but not the payoff. Waiting
  # here also orders the transcript screenshot below after the reply lands.
  await reply_exists \
    && pass "S0 payoff: wire reply on disk" \
    || fail "S0 payoff: wire reply on disk" "no gpt reply in 40s"

  # Select the conversation through the §11 binding, not pixels: ↓ steps the
  # flattened roster and lands via the focus_agent path, which sets BOTH the
  # focused workspace and the selected agent — so this one key replaces the old
  # workspace-tab (1063,12) and conversation-row (74,56) clicks, and cannot
  # drift when the list grows a header (it did: e8f7033's `recent | by ball`).
  # The send above put the keyboard back in the composer (§11 focus discipline),
  # so the release `bare` carries is what opens the shell's wants_keyboard_input
  # guard and lets the key reach the keymap.
  "$drive" bare "$wid" Down ; sleep 1
  "$drive" shot "$wid" "$out/s0-04-transcript.png"

  # S1 message-to-agent: type into the focused conversation, Enter.
  # Escape then `i` hands the keyboard to the bottom composer, now in `message
  # the selected conversation` mode, WITHOUT touching the target: `i` is the
  # focus binding, `n` would clear the selection. This is what retired the
  # (575,684) click that bl-0293 caught drifting — the text edit had moved to
  # y≈705 under the composer's header row and the type went nowhere.
  #
  # The ↓ is INSIDE the retried gesture, not the line above it, because the
  # selection is this beat's unverified precondition and it is invisible on disk
  # (per-instance RAM, §13.1). A ↓ that arrives before yog's snapshot carries the
  # new root selects nothing, so no workspace is focused, so the composer panel
  # **is not rendered at all** — the click hits blank panel and the beat reads as
  # "no message verb" (it did, at load average 37). Re-arming the selection on
  # each retry is what fixes it, and it is safe in both directions: a missed ↓
  # leaves nothing to type into, and an Enter on an empty draft is refused by
  # `message_enabled`, so a miss spawns nothing (the failed run's ops trail had
  # zero strays across five attempts). With one conversation in the roster a
  # repeated ↓ wraps onto the same row, so a landed selection survives a retry.
  before=$(verb_count message)
  message_it() {
    "$drive" bare "$wid" Down
    "$drive" bare "$wid" i
    "$drive" type "$wid" "Now respond with exactly: Second wire OK."
    "$drive" key "$wid" Return
  }
  until_landed message_it verb_ge message $((before + 1)) \
    && pass "S1 message-to-agent: lernie message" \
    || fail "S1 message-to-agent: lernie message" "no message verb"
  # THE PAYOFF, and the beat that was missing (bl-bf79). The line above asserts
  # that yog SPAWNED the verb; it is true of a `lernie message` whose revived
  # driver dies on its first `bz`, which is what shipped. The conversation went
  # quiescent after S0's reply, so this message is the revive path — no live
  # driver to hand it to — and only the reply proves the driver came back inside
  # the workspace's wall (§16.2). Await, never sleep: the latency is the model's.
  await second_reply \
    && pass "S1 message-to-agent: the revived driver replied" \
    || fail "S1 message-to-agent: the revived driver replied" "second turn unanswered in 40s"
  no_dead_step \
    && pass "S1 message-to-agent: no step died before its first token" \
    || fail "S1 message-to-agent: no step died before its first token" "empty response.json"
  "$drive" shot "$wid" "$out/s1-05-message.png"

  # S1 restart-equivalence: kill, relaunch same world, state re-derives.
  "$drive" stop "$pid" ; sleep 1
  ops_lines_before=$(wc -l < "$ops")
  launch_engine "$data" ; pid2=$engine_pid ; wid2=$engine_wid
  sleep 2
  "$drive" shot "$wid2" "$out/s1-06-restart.png"
  [ "$(wc -l < "$ops")" = "$ops_lines_before" ] \
    && pass "S1 restart: idle is pure (INV-1)" \
    || fail "S1 restart: idle is pure (INV-1)" "spawn at idle"

  # S1 prompt-into-existing: Enter in the focused workspace's composer.
  new_before=$(verb_count new) ; agents_before=$(agent_count)
  # The same composer box, back in `start a conversation` mode — selection is
  # per-instance RAM (§13.1), so the relaunch starts with nothing selected and
  # the composer retargets to a new conversation by itself. Same Escape+`i`.
  prompt_existing() {
    "$drive" key "$wid2" Escape ; "$drive" key "$wid2" i
    "$drive" type "$wid2" "Respond with exactly: Third wire OK."
    "$drive" key "$wid2" Return
  }
  # `>=`, not `=` — the same latent defect as S4-T4's, which reddened first only
  # because the box was busier that hour (bl-0e44): this gesture starts a
  # conversation, so a retry adds one and an equality is destroyed by its own
  # loop. The claim that no second workspace was minted is the neighbour below.
  until_landed prompt_existing agents_ge $((agents_before + 1)) \
    && pass "S1 prompt-existing: new root agent" \
    || fail "S1 prompt-existing: new root agent" "no new agent"
  sleep 2
  "$drive" shot "$wid2" "$out/s1-07-prompt-existing.png"
  [ "$(verb_count new)" = "$new_before" ] \
    && pass "S1 prompt-existing: no re-mint" \
    || fail "S1 prompt-existing: no re-mint" "lernie new re-fired"

  "$drive" stop "$pid2"
  verdict "$out"
}

# The per-world beat bodies, sourced rather than duplicated: each reuses `$drive`,
# `seed`, and every assertion helper above (and `in_world`/`seed_balls`, laid by
# the first). One file per world, because this file is at the 300-line cap.
. "$here/beats_s3s4s6.sh"
. "$here/beats_s5_fixture.sh"
. "$here/beats_s5.sh"
. "$here/beats_s5_run.sh"
. "$here/beats_s8.sh"
. "$here/beats_s7.sh"
. "$here/beats_s6.sh"
. "$here/beats_s3res.sh"
. "$here/beats_headless.sh"
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
