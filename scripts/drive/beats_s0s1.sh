# beats_s0s1.sh — the S0/S1 beat body, sourced by stories.sh (same dir).
#
# The first world's beats, cut out of `stories.sh` at that file's 300-line cap
# on the seam this directory already runs on: one file per world, `stories.sh`
# keeping only the world seed, the tier sourcing and the verb dispatch. S0 is
# the first launch (the seeded-skip, the bootstrap composer, the mint); S1 is
# the operator's second and third turns against the same world — a follow-up
# into the live conversation, and a prompt that starts another one without
# re-minting the workspace.
#
# It calls `seed` and every assertion helper from `predicates.sh` exactly as the
# other `beats_*.sh` do, and it is sourced before `one_name_one_definition`
# runs, so a name collided with any other beat file is refused there.


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
    && pass "S0 bare-start: litany new fired" \
    || fail "S0 bare-start: litany new fired" "no new"
  "$drive" shot "$wid" "$out/s0-03-fired.png"

  # Locate the workspace the fire minted (single dir under yog/workspaces) — a
  # READ of what happened, which is why it is a `find` and not `$BOOTSTRAP_WS`:
  # the wall the seed laid is a path this harness chose, the workspace is a fact
  # yog produced, and a beat must never assert the second by restating the first.
  # Its wall was seeded before the launch (§16.2, bl-1851 — see `seed` above): the
  # mint and the first `litany prompt` are one gesture, so nothing laid here could
  # beat the first model call, and the payoff beat below is what that cost.
  ws_root=$(find "$data/yog/workspaces" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | head -1)

  [ "$(verb_count prime)" = 0 ] \
    && pass "S0 seeded-skip: no litany prime" \
    || fail "S0 seeded-skip: no litany prime" "prime spawned"
  [ "$(verb_count prompt)" -ge 1 ] \
    && pass "S0 bare-start: detached litany prompt" \
    || fail "S0 bare-start: detached litany prompt" "no prompt"
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
    && pass "S1 message-to-agent: litany message" \
    || fail "S1 message-to-agent: litany message" "no message verb"
  # THE PAYOFF, and the beat that was missing (bl-bf79). The line above asserts
  # that yog SPAWNED the verb; it is true of a `litany message` whose revived
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
    || fail "S1 prompt-existing: no re-mint" "litany new re-fired"

  "$drive" stop "$pid2"
  verdict "$out"
}
