#!/bin/bash
# beats_s4res.sh — the **S4 board residual stages**: the rows S4 needs a world
# to be shaped a particular way for, so each one is a stage a run verb reaches
# for rather than a body sitting inside one. Sourced by stories.sh; not an
# entry point of its own.
#
# Two runs fire them, and which one can afford which is the whole reason they
# are here. `run_s3s4s6` (beats_s3s4s6.sh) owns a world with a project and a
# focused sphere, so it fires `s4_new_workspace` — the deliberate mint and the
# focus that follows it. `run_s7` (beats_s7.sh) owns world C, which holds no
# project at all, so it fires the two rows that need exactly that: a foreign
# workspace to fall to the ⋯ menu, and a ball id no join on this machine can
# colour.
#
# Cut here at the repo's 300-line cap (bl-7547), on the seam `beats_s6.sh`
# already ran on and its own header already named: a run verb owns its world
# and its fixtures and reaches out for the stages that world can support, so a
# stage never sits in the middle of a run's body. `beats_s6.sh` is the same
# tier for the S6 rows, and the two files split what one used to carry.

# S4-T1 — the deliberate sphere-wall mint, and the focus that follows it.
# `$1` is the window, `$2` the evidence dir, `$3` the world's data root; `$ops`
# is the run's, as it is for every stage here.
s4_new_workspace() {
  wid=$1 ; out=$2 ; data=$3
  # S4 New workspace (S4-T1): `w` is §11's deliberate sphere-wall mint — and it
  # is DELIBERATE in the literal sense, which is the whole of this beat's shape.
  # `w` opens the name form and nothing else (`src/shell/new_ws.rs`): the sphere
  # is raised by a name the OPERATOR types and §3.1 validates, so a `w` on its
  # own can never fire `lernie new` and no `await` window changes that. Both
  # authorities say so outright — DESIGN §11's table ("new workspace: the
  # deliberate sphere-wall raise, name typed by the operator") and STORIES S4-T1
  # ("the operator's typed, validated name") — and the beat that pressed `w`
  # alone was written three days before the name form landed (bl-afa7). So the
  # gesture is the whole form: open it, type a valid name, Return, which
  # `new_ws.rs` submits exactly as Create does.
  #
  # Retried like every other gesture: `bare` spends an Escape first, which is
  # the form's own dismissal (§11), so a retry re-opens an EMPTY form rather
  # than typing twice into a half-filled one — and after the mint has landed the
  # same name is refused as taken, so a late retry cannot raise a third sphere.
  first_ws=$(find "$data/yog/workspaces" -maxdepth 1 -mindepth 1 -type d | head -1)
  raise_sphere() {
    "$drive" bare "$wid" w
    "$drive" type "$wid" "ops"
    "$drive" key "$wid" Return
  }
  until_landed raise_sphere verb_ge new 2 \
    && pass "S4 new workspace: second lernie new" \
    || fail "S4 new workspace: second lernie new" "no mint"
  sleep 2
  "$drive" shot "$wid" "$out/s4-11-second-workspace.png"
  spheres_are "$data" 2 \
    && pass "S4 new workspace: two named spheres" \
    || fail "S4 new workspace: two named spheres" "not two"

  # S4-T1 focus follows the mint (bl-2826, §3.4 "a start focuses the workspace
  # it resolved"). The failure this pins was silent: the mint left the tab bar,
  # the conversation list and the BOTTOM composer on the workspace the operator
  # had just walked away from, so the goal typed into the box that *looks* like
  # the place you type fired into the OLD sphere and the fresh one stayed a husk
  # (`repo.git` only — never prompted). Focus is per-instance RAM (§13.1), so it
  # is not readable from any file: the argv of the prompt the bottom composer
  # fires IS the observable, and it is the one the operator was burned by.
  #
  # Escape drops the start pane's draft (§11 Cancel), `i` puts the cursor in the
  # bottom composer, Return fires its bare rung — which resolves the FOCUSED
  # workspace (§3.4). The reply is not awaited; only the dispatch is the claim.
  #
  # `$minted` is the SUBJECT of both assertions below, so it is judged before it
  # is used: when the mint above failed it was the empty string, the grep became
  # `grep -q '""'` — which matches nearly any ops row — and both beats reported
  # PASS while asserting nothing, beside a red mint (bl-afa7). A beat that
  # cannot fail is worse than a red one.
  minted=$(find "$data/yog/workspaces" -maxdepth 1 -mindepth 1 -type d ! -path "$first_ws" | head -1)
  mints=$(verb_count new) ; prompts=$(verb_count prompt)
  bottom_send() {
    "$drive" bare "$wid" i
    "$drive" type "$wid" "Respond with exactly this text and nothing else: focus OK"
    "$drive" key "$wid" Return
  }
  until_landed bottom_send verb_ge prompt $((prompts + 1)) \
    && pass "S4 mint focus: the bottom composer fired" \
    || fail "S4 mint focus: the bottom composer fired" "no prompt row"
  "$drive" shot "$wid" "$out/s4-12-mint-focused.png"
  if [ -z "$minted" ]; then
    fail "S4 mint focus: Enter prompts the MINTED sphere" "no second sphere to prompt"
    fail "S4 mint focus: focused ⇒ no re-mint" "no second sphere to prompt"
  else
    grep '"lernie","prompt"' "$ops" | tail -1 | grep -q "\"$minted\"" \
      && pass "S4 mint focus: Enter prompts the MINTED sphere" \
      || fail "S4 mint focus: Enter prompts the MINTED sphere" "wrong workspace"
    [ "$(verb_count new)" = "$mints" ] \
      && pass "S4 mint focus: focused ⇒ no re-mint" \
      || fail "S4 mint focus: focused ⇒ no re-mint" "lernie new re-fired"
  fi
  # Escape is §11's Cancel — leave no draft behind for the beats that follow.
  "$drive" key "$wid" Escape ; sleep 1
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
