#!/bin/bash
# beats_s6.sh — the S6 Triager stages, fired from `run_s7` in beats_s7.sh
# (which owns world C and lays their fixtures) and, for the stop-and-ack stage
# at the foot, from `run_s3s4s6` in beats_s3s4s6.sh. Sourced by stories.sh; not
# an entry point of its own. The S4 board residual stages those same two runs
# fire are next door in `beats_s4res.sh` — one file per story, which is the
# seam this whole tier is cut on (bl-7547). **This is the S6 stage tier**: a
# run verb owns its world and its fixtures, and reaches here for the S6 beats
# that world can support — which is why an S6 stage never sits inside a run's
# body, and why moving one out of `run_s3s4s6` was a seam rather than a shave
# when that file went over the 300-line cap (bl-2d45).
#
# These are the S6 rows bl-84f3 could not reach: §6's budget / conflicted /
# mail predicates (each needs its own world state), the stop and its
# acknowledgement by identity, and the ack convergence across two instances.
# Their fixtures are laid in `lay_forensics` and here; the assertions are
# ui.json watermarks, with the screenshots carrying the halves no file can (a
# colour, a strip total).

# S6-T1 rules 3 and 4 (budget-exhausted, conflicted): a mark that lands while
# nothing is focused is attention, and walking the roster onto it is the
# acknowledgement (§6, "focusing an agent records the current evidence oids as
# seen").
#
# THIS BEAT LAYS ITS OWN TWO MARKS, and that is the whole of what makes it
# assertable (bl-1061). They used to be laid in `lay_forensics`, with the rest
# of world C, ~30 s and three beats before this one — and `s7_steps` walks the
# selection back onto this very agent in between (its ↑ from the unfolded
# child). §6's ack is a STATE, not an edge: "a focused agent's evidence oids are
# stamped on every frame it stays focused", so that ↑ acknowledged both marks
# before this beat pressed anything. All three rows below then passed on a
# watermark written by an earlier beat, and they passed just as green with this
# beat's gesture DELETED — proved by driving `run-s7` with the gesture stubbed
# out (bl-1061; the S6-T1 rows were ALL BEATS PASS either way, and the second
# root's `seen` entry, present only in the unstubbed run, is where the ↓
# actually landed). A fixture laid where the beat that spends it cannot see it
# land is a fixture whose beat proves nothing.
#
# AND THE SORT'S HEAD IS NOT THE TARGET — that premise died with bl-cad5, the
# same day and the same way it died for `s6_stop_ack` below, which was fixed
# (bl-2d45) while this beat was deleted from every run by the name collision and
# so kept it. DESIGN §6: "The conversation list is no longer one of these
# groups — since bl-cad5 it sorts by recency alone (§11); attention there is a
# badge only — not a count, and not a rank", and "the rank orders the jump, not
# the walk (bl-fa82)": ↑/↓ walk the focused workspace's visible list rows in
# paint order. The second root is strictly newer, so the head is IT, and the
# evidence says so — in the run where this gesture still fired, the only
# watermark it wrote was that root's. So the ack is a WALK: one ↓ per attempt,
# re-read after each, which is `until_landed`'s own contract. The `n` stays
# OUTSIDE that loop for the same reason — re-clearing the selection on every
# retry pressed ↓ onto the same head five times, so the walk could never advance
# past it.
#
# Rule 5 (mail nobody is driving) rides along and is the interesting half: it is
# **not** seen-gated, so the very same ack leaves it flagged — a stall you can
# dismiss is a stall you will miss.
s6_attention() {
  wid=$1 ; out=$2 ; agent=$3
  # `n` (§11 new conversation) clears the agent selection — and hands the
  # composer keyboard focus, which would swallow ↓. The `bare` verb's own
  # release is what gives it back before each bound gesture. Fired ONCE, before
  # the marks: with no agent focused there is no frame that can stamp them.
  "$drive" bare "$wid" n ; sleep 1
  # §6 rules 3 and 4, in litany's own spelling — a mark IS a ref (§2.6), so
  # there is nothing else to write. Laid here rather than in `lay_forensics`
  # because this is the only beat that spends them and the header above is why.
  tip=$(git --git-dir="$ws_root/repo.git" rev-parse "agents/$agent")
  git --git-dir="$ws_root/repo.git" update-ref "refs/litany/budget-exhausted/$agent" "$tip"
  git --git-dir="$ws_root/repo.git" update-ref "refs/litany/conflicted/$agent" "$tip"
  sleep 3
  # The strip with both flags UP and nothing acknowledged — the before half of
  # the pair, and now it is genuinely before.
  "$drive" shot "$wid" "$out/s6-01-strip-stirs.png"
  step() { "$drive" bare "$wid" Down; }
  until_landed step seen_kind "$ui" "$agent" budget \
    && pass "S6-T1 budget: the roster walk acks a mark laid while unfocused" \
    || fail "S6-T1 budget: the roster walk acks a mark laid while unfocused" "no budget watermark"
  seen_kind "$ui" "$agent" conflicted \
    && pass "S6-T1 conflicted: the same landing acks the second mark" \
    || fail "S6-T1 conflicted: the same landing acks the second mark" "no conflicted watermark"
  sleep 2
  "$drive" shot "$wid" "$out/s6-02-strip-acked.png"
  # Rule 5 is not silenceable: no `mail` watermark exists to write, in any
  # instance, ever — the screenshot shows the strip still carrying it.
  #
  # BRACKETED, because the claim is a negative and a negative is satisfied by the
  # ack never happening. A bare `! grep -q '"mail"' "$ui"` was true of a missing
  # ui.json and of every run where the ↓ above landed on nothing: `grep` exits 2,
  # the `!` turns that into a PASS, and the beat reported that mail survived an
  # acknowledgement that was never made (bl-f16e). The first clause is the
  # bracket — THIS agent was acknowledged — so the absence below is an absence
  # measured against a write that certainly happened. It names the KIND, not
  # merely the agent: a bare `seen_kind "$ui" "$agent"` is satisfied by the
  # rule-2 `stopped` watermark an earlier beat's selection already wrote, which
  # is the same leftover the two rows above were passing on (bl-1061). `budget`
  # is the one this beat laid and this beat acked.
  { seen_kind "$ui" "$agent" budget && ! grep -q '"mail"' "$ui"; } \
    && pass "S6-T1 mail: no watermark exists to silence it" \
    || fail "S6-T1 mail: no watermark exists to silence it" "mail was seen-gated, or no ack to measure"
}

# S6 stop-and-ack in world 2 — Stop the in-flight root, then acknowledge it.
# **Not `s6_attention`**, which is world C's S6-T1 stage above: bash lets a later
# definition silently replace an earlier one, so naming this that deleted three
# S6-T1 beats from every `run_s7` and no verdict could show it — a beat that
# never ran leaves no row (bl-0e44; `stories.sh` now refuses a duplicate name
# outright). Fired from `run_s3s4s6` (beats_s3s4s6.sh), which lays the two roots
# and hands this the in-flight one's id; it lives here with the other S6 stages rather than in
# the middle of that run's body, which is also what put that file over the
# 300-line cap (bl-2d45). `$1` is the window, `$2` the evidence dir, `$3` the
# conversation to stop — empty when the start that should have made it never
# landed, which is a red, not a skip.
#
# `x` is §11's Stop on the SELECTION (§8.2), children per the row's own
# checkbox — and the selection is ALREADY the in-flight root, with nothing to
# press to put it there: since bl-49cb a start focuses what it started, and that
# rule "runs all the way down to the conversation" (DESIGN §3.4), so the send
# that created this conversation also selected it.
#
# A ↓ used to stand here, on the premise that it "lands on the §6 sort's head —
# the in-flight root, running outranks settled". Both halves died on 2026-08-01:
# bl-49cb made the start select its own conversation, so the ↓ moved OFF the
# target rather than onto it, and bl-cad5 left the list sorting by recency alone
# — DESIGN §11: "attention and liveness are badges here, not ranks" — so there
# is no rank for a head to be the head of. `x` then correctly found a settled
# conversation and stopped nothing, five times over (bl-2d45).
s6_stop_ack() {
  wid=$1 ; out=$2 ; flight=$3
  "$drive" shot "$wid" "$out/s6-07-inflight.png"
  if [ -z "$flight" ]; then
    fail "S6 stop: litany stop dispatched" "no second root to stop"
    fail "S6 ack: seen watermark names the stopped conversation" "no second root"
    return 0
  fi
  # `litany stop` reaps a *streaming* driver, so its row lands when the driver
  # actually dies — a 5 s sleep here read a live stop as "no stop verb" once.
  # The predicate names the conversation, not a count: the failure this beat
  # exists to catch is a stop of the WRONG one, which a count cannot see.
  stop_it() { "$drive" bare "$wid" x; }
  until_landed stop_it stopped "$flight" \
    && pass "S6 stop: litany stop dispatched" \
    || fail "S6 stop: litany stop dispatched" "no stop verb for $flight"
  sleep 3
  "$drive" shot "$wid" "$out/s6-08-stirs.png"
  # Acknowledging is focusing (§6): walk the roster with ↓ — the bound gesture —
  # and the watermark lands in ui.json under the agent that was focused. Two
  # roots, so ↓↓ is a round trip that ends where it began, on the stopped one.
  # The watermark is read BY ID: a bare `grep '"seen"'` was satisfied by the
  # watermark the S3 selection had already written, so it reported PASS in the
  # same runs where the stop above never happened at all (bl-2d45).
  "$drive" bare "$wid" Down ; sleep 1 ; "$drive" bare "$wid" Down
  sleep 3
  "$drive" shot "$wid" "$out/s6-09-acked.png"
  await seen_kind "$ui" "$flight" \
    && pass "S6 ack: seen watermark names the stopped conversation" \
    || fail "S6 ack: seen watermark names the stopped conversation" "no seen record for $flight"
}

# S6-T2 — the acknowledgement CONVERGES: a second instance over the same
# `ui.json` stops flagging what the first acked, and it is idle-pure while doing
# it (INV-1). Focus and scroll stay per-instance (§13.1), which is why the beat
# reads the strip, not the selection.
s6_converges() {
  data=$1 ; out=$2
  lines=$(wc -l < "$ops")
  launch_engine "$data" ; bpid=$engine_pid ; bwid=$engine_wid
  sleep 4
  "$drive" shot "$bwid" "$out/s6-06-second-instance.png"
  [ "$(wc -l < "$ops")" = "$lines" ] \
    && pass "S6-T2 ack-converges: second instance adopts, spawns nothing" \
    || fail "S6-T2 ack-converges: second instance adopts, spawns nothing" "spawn at idle"
  "$drive" stop "$bpid"
}
