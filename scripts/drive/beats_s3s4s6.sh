#!/bin/bash
# beats_s3s4s6.sh — the S3/S4/S6 beat body of stories.sh (which sources this;
# it is not an entry point of its own). Split out for the repo's 300-line cap;
# it uses stories.sh's seat handle (`$drive`), world seed and assertion helpers.

# --- the ball world (S3/S4/S6) ----------------------------------------------
# One command inside the world (§8.4 `yog exec`) — the same hatch the empty
# balls section itself prints as the paved way to enter a project (S3-T5), so
# this fixture is the story's own gesture, not new harness machinery.
in_world() { d=$1; shift; XDG_DATA_HOME="$d" yog exec --cwd "$d/proj" "$@"; }

# Ball-world predicates, shaped for `await` (stories.sh): each is a bare
# true/false read of a real surface, never a sleep-and-hope.
ball_gone() { [ "$(in_world "$1" bl list --json)" = "[]" ]; }
closed_claims() { in_world "$1" bl list -s closed --json | grep -q "\"claimant\": \"$2\""; }
spheres_are() { [ "$(find "$1/yog/workspaces" -maxdepth 1 -mindepth 1 -type d | wc -l)" = "$2" ]; }
# `stopped` and `other_root` — the two predicates this run shares with the S6
# stage it hands off to — are in `harness.sh`, the tier every run shares: they
# read `$ops` and `$ws_root` exactly as `verb_ge` and `agent_count` do.
# the ball rung's whole point (§3.3): the detached driver runs *in the worktree*
# `bl claim` cut, not in the project and not in the operator's cwd.
prompt_cwd_is_worktree() {
  grep '"lernie","prompt"' "$ops" 2>/dev/null | grep -q "\"cwd\":\"[^\"]*/$1\""
}

# The world seed plus a real git project primed INTO the world carrying one
# ready ball; prints the ball id. A **separate** world from `run` on purpose:
# with zero workspaces the ball rung mints one (`lernie new` *then* `bl claim` —
# the §8.1 order S3-T1 asserts) and the conversation list is empty, which is
# what keeps the balls-section coordinates below deterministic. Reusing `run`'s
# world would give a focused workspace (no mint) and rows above every click.
seed_balls() {
  data=$1 ; p="$data/proj"
  seed "$data"
  mkdir -p "$p"
  git -C "$p" init -q -b main
  git -C "$p" config user.email drive@yog.invalid
  git -C "$p" config user.name yogdrive
  : > "$p/README.md" ; git -C "$p" add -A ; git -C "$p" commit -qm init
  in_world "$data" bl prime --as yogdrive >/dev/null 2>&1
  # The body is the goal's payload verbatim (§3.3), and the worktree preamble
  # rides beside it — which reads as an invitation to work the repo, so the body
  # forbids tools outright: the beat is about yog's argv and surfaces, not about
  # an agent doing a job, and every tool round trip is wire spend.
  in_world "$data" bl create "drive ball" \
    --body "Respond with exactly this text and nothing else: Ball wire OK. Run no commands and no tools." \
    --as yogdrive
}

# S3 (the close flow), S4 (a second conversation + the by-ball toggle) and S6
# (the attention strip + the activity chip) — STORIES.md's "Drivable next" set.
# Steering is §11 keys throughout (see stories.sh's STEERING RULE for the table):
# ↓ selects, `s` starts the top ready ball, Return sends the goal, `c` closes the
# bound ball, `n` opens a conversation, `x` stops, `g` toggles the organizing
# view, `w` mints a workspace, Escape cancels the goal draft, `a` opens the
# activity accessory. The one click left is a *pick* (which ops row to expand)
# and is tagged where it fires.
run_s3s4s6() {
  data=$1 ; out=$2
  mkdir -p "$out" ; rm -rf "$data" ; mkdir -p "$data"
  claim_seat
  ball=$(seed_balls "$data")
  ops="$data/yog/world/state/yog/ops.jsonl"
  ui="$data/yog/world/state/yog/ui.json"
  read -r pid wid < <("$drive" launch "$data")
  sleep 2
  "$drive" shot "$wid" "$out/s3-01-ready-ball.png"

  # S3 ball rung: `s` is §11's ▶ Start on the balls section's FIRST ready row,
  # which is this world's only one. `bare` releases first — the composer opens
  # focused at launch (§11 focus discipline) and a bare binding is suppressed
  # while a text box holds the keyboard.
  # The row only appears after yog's first project scan, and `bl claim` then cuts
  # a git worktree — so fire until the claim row says it landed rather than
  # pressing once into whatever is painted at 2 s.
  start_ball() { "$drive" bare "$wid" s; }
  until_landed start_ball row_ok "\"bl\",\"claim\",\"$ball\",\"--as\"" \
    && pass "S3 ball-rung: claim --as workspace name" \
    || fail "S3 ball-rung: claim --as workspace name" "no clean claim row"
  "$drive" shot "$wid" "$out/s3-02-claimed-composer.png"
  new_at=$(verb_line lernie new) ; claim_at=$(verb_line bl claim)
  { [ -n "$claim_at" ] && [ -n "$new_at" ] && [ "$new_at" -lt "$claim_at" ]; } \
    && pass "S3 ball-rung: lernie new then bl claim" \
    || fail "S3 ball-rung: lernie new then bl claim" "order/rows missing"

  # Return is §11's Send (detached prompt) for the editable-goal composer (§8.1).
  # The goal box is not auto-focused, so the key reaches the table rather than
  # the text; a re-fire after the send is a no-op (nothing pending).
  ws_root=$(find "$data/yog/workspaces" -maxdepth 1 -mindepth 1 -type d | head -1)
  # Its wall — the providers and sign-ins the goal below dispatches through — was
  # laid with the world seed, before the launch (§16.2, bl-1851: the bootstrap
  # leaf is §3.1's constant, so the fixture never has to chase a mint).
  send_goal() { "$drive" key "$wid" Return; }
  until_landed send_goal prompt_cwd_is_worktree "$ball" \
    && pass "S3 ball-rung: prompt cwd is the ball worktree" \
    || fail "S3 ball-rung: prompt cwd is the ball worktree" "no prompt from the worktree"
  await reply_exists \
    && pass "S3 ball-rung: wire reply on disk" \
    || fail "S3 ball-rung: wire reply on disk" "no gpt reply in 40s"
  sleep 2
  "$drive" shot "$wid" "$out/s3-03-sent.png"

  # Select through the §11 binding (↓ lands via focus_agent), never a row click.
  "$drive" bare "$wid" Down ; sleep 2
  "$drive" shot "$wid" "$out/s3-04-bound-badges.png"

  # S3 close (S3-T7): `c` is §11's Close on the FOCUSED conversation's bound ball
  # (§8.2) — the ↓ above is what names the target, and the binding is refused
  # exactly where the button is disabled.
  # A close folds main in, runs the repo's hook and tears the worktree down; its
  # row lands when that finishes, which is nothing like a fixed interval.
  close_ball() { "$drive" bare "$wid" c; }
  until_landed close_ball row_ok "\"bl\",\"close\",\"$ball\",\"--as\"" \
    && pass "S3 close: bl close --as workspace name" \
    || fail "S3 close: bl close --as workspace name" "no clean close row"
  sleep 3
  "$drive" shot "$wid" "$out/s3-05-delivered.png"
  await ball_gone "$data" \
    && pass "S3 close: ball gone from the live set" \
    || fail "S3 close: ball gone from the live set" "still listed"
  # The other half of S3-T7: the row's *delivered* reading is re-derived from the
  # closed listing's claimant, so assert that claimant is the workspace name —
  # the same string the badge and the balls section paint from (§3.4/§3.5).
  await closed_claims "$data" "$(basename "$ws_root")" \
    && pass "S3 close: closed listing claims the sphere" \
    || fail "S3 close: closed listing claims the sphere" "claimant not the workspace"

  # S4 second conversation: `n` is §11's new conversation — it clears the agent
  # selection AND hands the composer the keyboard, so nothing else focuses the
  # box. `bare` releases first so a retry types into the box, not into the draft.
  # A long reply keeps the agent in flight for S6.
  #
  # The root already here is the ball rung's; naming it now is what lets the S6
  # beats below name the OTHER one — the in-flight conversation this start is
  # about — by its id rather than by its rank in a list (bl-2d45).
  settled_root=$(find "$ws_root/agents" -maxdepth 1 -mindepth 1 -type d -printf '%f\n' | head -1)
  second_conversation() {
    "$drive" bare "$wid" n ; sleep 1
    "$drive" type "$wid" "Print the numbers 1 through 2000, one per line."
    "$drive" key "$wid" Return
  }
  # `>=`, not `=`: this gesture STARTS a conversation, so a retry adds one and an
  # equality would be destroyed by its own loop (bl-0e44, harness.sh's
  # `until_landed` contract). The label says what is now asserted — a second root
  # under this one workspace — and the exactness that matters, that no second
  # WORKSPACE was minted, is the neighbour below, which counts a verb that must
  # not grow.
  until_landed second_conversation agents_ge 2 \
    && pass "S4 second conversation: a second root, same workspace" \
    || fail "S4 second conversation: a second root, same workspace" "agents=$(agent_count)"
  [ "$(verb_count new)" = 1 ] \
    && pass "S4 second conversation: no re-mint" \
    || fail "S4 second conversation: no re-mint" "lernie new re-fired"
  sleep 5
  "$drive" shot "$wid" "$out/s4-06-second-conversation.png"

  # S6 attention — the stop and its acknowledgement, in `beats_s6.sh` with the
  # other S6 stages (the same seam `run_s7` already reaches across for
  # `s6_converges`). It is handed the two roots by id, because naming them is
  # the whole of what bl-2d45 fixed.
  s6_stop_ack "$wid" "$out" "$(other_root "$settled_root")"

  # S4 by-ball toggle (S4-T5/§4.6): a pure re-ordering of rows already on
  # screen. `g` is §11's organizing view — one key, recent ⇄ by ball, so the
  # return trip below is the same key again.
  ui_hash=$(md5sum "$ui" | cut -d' ' -f1) ; ops_lines=$(wc -l < "$ops")
  "$drive" bare "$wid" g ; sleep 2
  "$drive" shot "$wid" "$out/s4-10-by-ball.png"
  { [ "$(md5sum "$ui" | cut -d' ' -f1)" = "$ui_hash" ] \
    && [ "$(wc -l < "$ops")" = "$ops_lines" ]; } \
    && pass "S4 by-ball: ephemera, no write no spawn" \
    || fail "S4 by-ball: ephemera, no write no spawn" "ui.json/ops moved"
  "$drive" bare "$wid" g ; sleep 1

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

  # S6 activity chip (S6-T5): `a` is §11's activity accessory, collapsed ⇄
  # expanded. CLICK (a VIEW, and a pick — §11 rule 2): WHICH ops row to open —
  # the newest, DERIVED off the window's own bottom edge from the frame the
  # line above photographed (bl-5cce, `locate.sh activity`). The trail is docked
  # to that edge and §11's tail idiom seats the newest op on it, so nothing
  # above the row — the chip heading, the trail controls, the §7.2 staleness
  # notes — can move it, and a still-collapsed accessory is refused rather than
  # clicked back shut.
  ops_lines=$(wc -l < "$ops")
  "$drive" bare "$wid" a ; sleep 2
  "$drive" shot "$wid" "$out/s6-12-activity-open.png"
  read -r row_x row_y < <("$here/locate.sh" activity "$out/s6-12-activity-open.png")
  "$drive" click "$wid" "$row_x" "$row_y" ; sleep 2
  "$drive" shot "$wid" "$out/s6-13-ops-row.png"
  [ "$(wc -l < "$ops")" = "$ops_lines" ] \
    && pass "S6 activity: expansion is a pure read" \
    || fail "S6 activity: expansion is a pure read" "ops grew"

  "$drive" stop "$pid"
  verdict "$out"
}

