#!/bin/bash
# beats_s6.sh — the S6 Triager and S4 board residual beats, fired from `run_s7`
# in beats_s7.sh (which owns world C and lays their fixtures) and, for the
# attention stage at the foot, from `run_s3s4s6` in beats_s3s4s6.sh. Sourced by
# stories.sh; not an entry point of its own. **This is the S6 stage tier**: a
# run verb owns its world and its fixtures, and reaches here for the S6 beats
# that world can support — which is why an S6 stage never sits inside a run's
# body, and why moving one out of `run_s3s4s6` was a seam rather than a shave
# when that file went over the 300-line cap (bl-2d45).
#
# These are the rows bl-84f3 could not reach: §6's budget / conflicted / mail
# predicates (each needs its own world state), the ack convergence across two
# instances, the tab strip's pins and overflow (needs a foreign workspace), and
# the conversation badge's uncoloured-id case (needs a goal stamped with a ball id
# this machine's join does not know). Their fixtures are laid in `lay_forensics`
# and here; the assertions are ui.json watermarks and on-disk goals, with the
# screenshots carrying the halves no file can (a colour, a strip total).

# S6-T1 rules 3 and 4 (budget-exhausted, conflicted) and the §6 sort's head.
#
# Both marks were laid AFTER this conversation was last focused, so their
# watermarks can only appear if a ↓ lands on that agent now — and a ↓ from a
# cleared selection lands on the §6 sort's HEAD. The world holds a second root
# whose tip is strictly newer and which carries no mark at all, so a sort by
# recency alone would land there and record nothing: the watermark's presence is
# the sort assertion too (attention outranks recency).
#
# Rule 5 (mail nobody is driving) rides along and is the interesting half: it is
# **not** seen-gated, so the very same ack leaves it flagged — a stall you can
# dismiss is a stall you will miss.
s6_attention() {
  wid=$1 ; out=$2 ; agent=$3
  "$drive" shot "$wid" "$out/s6-01-strip-stirs.png"
  # `n` (§11 new conversation) clears the agent selection — and hands the
  # composer keyboard focus, which would swallow ↓. The `bare` verb's own
  # release is what gives it back before the bound gesture.
  clear_and_step() {
    "$drive" bare "$wid" n ; sleep 1
    "$drive" bare "$wid" Down
  }
  until_landed clear_and_step seen_kind "$ui" "$agent" budget \
    && pass "S6-T1 budget: ack watermark after the sort's head" \
    || fail "S6-T1 budget: ack watermark after the sort's head" "no budget watermark"
  seen_kind "$ui" "$agent" conflicted \
    && pass "S6-T1 conflicted: ack watermark on the same landing" \
    || fail "S6-T1 conflicted: ack watermark on the same landing" "no conflicted watermark"
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
  # measured against a write that certainly happened.
  { seen_kind "$ui" "$agent" && ! grep -q '"mail"' "$ui"; } \
    && pass "S6-T1 mail: no watermark exists to silence it" \
    || fail "S6-T1 mail: no watermark exists to silence it" "mail was seen-gated, or no ack to measure"
}

# S4-T7 — the tab strip's overflow and pins. A **foreign** workspace (lernie's own
# auto-id territory under the nested `LERNIE_HOME`, §3.1) is real but not a
# regime, so it falls to the ⋯ menu rather than widening the wall row; ★ hoists it
# into the tabs, and that pin is durable (§4.1) — which is what makes this
# assertable rather than a screenshot claim.
s4_overflow() {
  wid=$1 ; out=$2 ; data=$3
  foreign="$data/yog/world/lernie/workspaces/20260727T093000Z-f0reign"
  # The §8.4 hatch again, without a project cwd this time (world C has none): one
  # `lernie new` at a path under the NESTED lernie data root is exactly what makes
  # a workspace foreign (§3.1) — nothing yog owns says so.
  XDG_DATA_HOME="$data" yog exec lernie new "$foreign" >/dev/null 2>&1
  sleep 4
  "$drive" shot "$wid" "$out/s6-03-overflow.png"
  # CLICK (a VIEW, and a pick — §11 rule 2): a pin is a §4.1 presentation
  # durable, which §8.5 puts on the views' side of the line outright ("durability
  # does not promote presentation state into an operation"), so it has no
  # boundary spelling by design. Both points are DERIVED from the frame above
  # (bl-5cce, `locate.sh tabbar`) rather than measured: the ⋯ overflow is painted
  # FIRST in a right-to-left bar, so it holds the window's right edge as soon as
  # it is non-empty, and the menu it opens is wider than the gap left beside it,
  # so egui clamps the popup into the frame and the ★ — the last widget of the
  # entry's row — lands one inset from that same edge. Two window edges and a
  # panel rule, none of them a number that a row can put wrong.
  # WHICH foreign workspace to pin is the pick, and the ★ is the ONLY safe target
  # in that row: the entry's own label focuses the workspace instead of pinning.
  read -r more_x more_y pin_x pin_y < <("$here/locate.sh" tabbar "$out/s6-03-overflow.png")
  pin_foreign() {
    "$drive" click "$wid" "$more_x" "$more_y" ; sleep 2
    "$drive" click "$wid" "$pin_x" "$pin_y"
  }
  until_landed pin_foreign file_has "$ui" 'f0reign' \
    && pass "S4-T7 tab strip: ★ pins the foreign workspace (ui.json)" \
    || fail "S4-T7 tab strip: ★ pins the foreign workspace (ui.json)" "no pin record"
  sleep 2
  "$drive" shot "$wid" "$out/s6-04-pinned.png"
}

# S4-T4's uncoloured-id case — the badge is honest about what it cannot know: a
# goal stamped with a ball id **this machine's join does not know** renders the id
# with no colour, because the stamp is truth and the colour is the join's only
# when it has one. World C holds no project at all, so every stamped id is
# unknown; the assertion is the stamp on disk, the colour is the screenshot's.
s4_uncoloured() {
  wid=$1 ; out=$2 ; ws_root=$3
  before=$(agent_count)
  # `n` (§11 new conversation) hands the composer focus itself, so nothing else
  # touches the box. The first line of the composed goal is the §3.3
  # `Ball <id>: <title>` header the stamp parse reads back.
  phantom() {
    "$drive" bare "$wid" n ; sleep 1
    "$drive" type "$wid" "Ball bl-9999: phantom. Respond with exactly: Phantom OK."
    "$drive" key "$wid" Return
  }
  # `>=`, not `=`. `phantom` is NOT a no-op when it misses — it starts a whole
  # conversation — so under load the slow first attempt lands late, the retry
  # starts a second, and an equality pinned to `before+1` is stepped straight
  # over by the loop that was waiting for it. This beat burned all five attempts
  # and reported "no new agent" against a world holding FIVE phantom
  # conversations, while the neighbour below PASSED on the goal.md files they
  # left: same evidence, opposite verdicts, and only the counting was wrong
  # (bl-0e44). The invariant now lives on `until_landed` itself.
  until_landed phantom agents_ge $((before + 1)) \
    && pass "S4-T4 uncoloured id: stamped conversation started" \
    || fail "S4-T4 uncoloured id: stamped conversation started" "no new agent"
  await goal_stamped "$ws_root" bl-9999 \
    && pass "S4-T4 uncoloured id: goal.md carries the unknown stamp" \
    || fail "S4-T4 uncoloured id: goal.md carries the unknown stamp" "no stamp on disk"
  sleep 3
  "$drive" bare "$wid" Down ; sleep 2
  "$drive" shot "$wid" "$out/s6-05-uncoloured-id.png"
}

# Any agent's `goal.md` carrying a `Ball <id>:` stamp (§3.3's one compose, read
# back off disk).
goal_stamped() {
  grep -lq "^Ball $2:" "$1"/agents/*/goal.md 2>/dev/null
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
    fail "S6 stop: lernie stop dispatched" "no second root to stop"
    fail "S6 ack: seen watermark names the stopped conversation" "no second root"
    return 0
  fi
  # `lernie stop` reaps a *streaming* driver, so its row lands when the driver
  # actually dies — a 5 s sleep here read a live stop as "no stop verb" once.
  # The predicate names the conversation, not a count: the failure this beat
  # exists to catch is a stop of the WRONG one, which a count cannot see.
  stop_it() { "$drive" bare "$wid" x; }
  until_landed stop_it stopped "$flight" \
    && pass "S6 stop: lernie stop dispatched" \
    || fail "S6 stop: lernie stop dispatched" "no stop verb for $flight"
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
  read -r bpid bwid < <("$drive" launch "$data")
  sleep 4
  "$drive" shot "$bwid" "$out/s6-06-second-instance.png"
  [ "$(wc -l < "$ops")" = "$lines" ] \
    && pass "S6-T2 ack-converges: second instance adopts, spawns nothing" \
    || fail "S6-T2 ack-converges: second instance adopts, spawns nothing" "spawn at idle"
  "$drive" stop "$bpid"
}
