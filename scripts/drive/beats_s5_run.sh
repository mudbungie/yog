#!/bin/bash
# beats_s5_run.sh — world A's RUNNER, `run_s5s8`: the world it lays, the order
# it drives, and the fixture beats that prove each step landed before the next
# one spends a coordinate. Split from beats_s5.sh at the 300-line cap (bl-fc3f)
# on the seam that file's own header already declared — the beats it fires are
# still `beats_s5.sh`'s (S5-T3/T4/T5) and `beats_s8.sh`'s (S8), and the
# fixtures and predicates it spends are `beats_s5_fixture.sh`'s. Sourced by
# stories.sh with those two; not an entry point of its own.
#
# World A holds what S5/S8/the residual ball rows need and neither S0/S1's world
# nor the ball world has: a workspace with a real `config/default` lineage bound to
# a project (S5-T5's and S8-T4's target, reached by a ▶ Start whose goal is never
# sent — so the whole run spends NOTHING on the wire).
#
# **The §9.1 fixture is the workspace's own wall, not a scratch `BRAZEN_CONFIG`.**
# Since the blast-radius ruling (§16.2) brazen's config lives at
# `<data>/yog/world/walls/<workspace>/brazen/config.toml`, inside the run's own
# `$XDG_DATA_HOME` — so isolation from the operator's own `~/.config/brazen` is
# structural rather than a var this script has to remember to export, and an
# exported `BRAZEN_CONFIG` is now inert (yog injects the wall's path and brazen
# never reaches its own fall-through). A fresh wall's config is an empty file,
# which keeps the draft inside `desired_rows(6)` and the editor unscrolled.

# --- the run ----------------------------------------------------------------
run_s5s8() {
  data=$1 ; out=$2
  mkdir -p "$out" ; rm -rf "$data" ; mkdir -p "$data"
  claim_seat
  ball=$(seed_balls "$data")
  ops="$data/yog/world/state/yog/ops.jsonl"
  ui="$data/yog/world/state/yog/ui.json"
  # The window's per-seat pane document (REMOTE §7, bl-8bbc): where the
  # collapse below actually lands. `ui` above keeps the WORLD keys' path.
  pane="$data/yog/world/state/yog/clients/yog-window/pane.json"
  # Trim the world's seeded `models.yaml` to the one model row it names. The seed
  # ships a 40-line comment banner, and that banner is the whole reason config
  # mode SCROLLS: with it, the §9.2 editor's box pushes the marks pane and the
  # staged-edit form off the bottom and every coordinate below has to be reached
  # by a wheel — which is a coordinate in disguise unless it hits a limit, and a
  # limit moves as the panes above grow. Short seed, one screen, fixed
  # coordinates. World A never prompts, so nothing here needs the other rows;
  # the file's PRESENCE is what the §16.6 W3 marker means (S0-T2).
  printf 'models:\n  gpt-5.4:\n    provider: codex\n    model_id: gpt-5.4\n    capabilities: [streaming]\n    context_window: 400000\n' \
    > "$data/yog/world/lernie/models.yaml"
  # The ambient world's own seed file, hashed: this run reads it (the §16.6 W3
  # seed copy) and must never write it — S8-T1's "untouched", asserted on the one
  # file the run actually touches on that side of the wall.
  ambient_before=$(md5of "$HOME/.local/share/yog/world/lernie/models.yaml")

  launch_engine "$data" ; pid=$engine_pid ; wid=$engine_wid
  sleep 2

  # Bind the ball with NO wire spend: `s` is §11's ▶ Start on the balls section's
  # first ready row — this world's only one — and it claims the ball and opens the
  # editable goal, which the relaunch below drops unsent (RAM until sent, §8.1).
  # `bare` releases first: the composer opens focused at launch (§11 focus
  # discipline) and a bare binding is suppressed while a text box holds it.
  start_ball() { "$drive" bare "$wid" s; }
  until_landed start_ball row_ok "\"bl\",\"claim\",\"$ball\",\"--as\"" \
    && pass "S5 fixture: ball bound, no wire spend" \
    || fail "S5 fixture: ball bound, no wire spend" "no clean claim row"
  # A second, READY ball — laid AFTER the claim above so the ▶ Start click had
  # exactly one startable row to hit. It is the residual rows' fixture (S4-T2's
  # assign target) and it fixes the balls section's shape for every coordinate in
  # `beats_s3res.sh`: one ready row, then one ▶ Continue row.
  READY_BALL=$(in_world "$data" bl create "assign me" --body "no tools" --as yogdrive)
  # Collapse the balls section *before* the restart, and the reason is a property
  # of egui panels worth recording: a `SidePanel` widens to fit its widest label
  # and never shrinks back, and the balls section's `+ new ball · <project>` row
  # carries an absolute PATH — so with the section open the side panel is ~680
  # wide instead of its 260 default and every centre-panel coordinate below moves
  # with the length of the scratch directory's name. Collapsed, that label is
  # never painted. The collapse is the one persisted (§4.1) piece of this layout,
  # so it is a landable gesture: the pane document records it (bl-8bbc) and the
  # next launch starts narrow. `b` is §11's balls-section fold — one key,
  # both directions, so the re-expand below is the same key and neither has
  # a y to drift.
  collapse_balls() { "$drive" bare "$wid" b; }
  until_landed collapse_balls file_has "$pane" '"balls"' \
    && pass "S5 fixture: balls collapsed (pane.json)" \
    || fail "S5 fixture: balls collapsed (pane.json)" "no collapse record"
  # Then RELAUNCH rather than click a tab. Three things fall out of one restart:
  # the un-sent goal draft dies with the instance (RAM until sent, §8.1 — so the
  # ball is bound with nothing spent on the wire), startup DERIVES the focus onto
  # the only workspace (§4.1), which every beat below needs, and the fresh process
  # lays the side panel out at its default width. A tab click would be a
  # coordinate whose landing nothing on disk can confirm (focus is per-instance
  # RAM, §13.1) and it drifts with the minted name's width; a restart cannot miss.
  "$drive" stop "$pid" ; sleep 2
  launch_engine "$data" ; pid=$engine_pid ; wid=$engine_wid
  sleep 2

  # The §9.1 fixture is laid HERE, before the pane is ever opened, and the
  # ordering is the whole of it: §9's freshness rule is the open gesture, so the
  # brazen editor's draft and its load-time snapshot are whatever the file was at
  # Ctrl+Shift+2. Seeded after that, the pane holds a draft that loaded "file
  # absent" while a file now exists — which is exactly what the §9 guard refuses,
  # so EVERY Apply below was a conflict and the bracket beat could not land
  # whatever it clicked (bl-f8dc). It overwrites the config the world seed laid
  # in the same wall (bl-1851) — world A never opens the Config tab before this
  # line, so the pane has read nothing yet and the freshness rule is satisfied by
  # the same ordering it always was.
  brazen_scratch "$(wall_config "$data")"

  # Order is load-bearing: the Config tab stacks its editors top-to-bottom in one
  # column, and every status line a beat adds pushes what is BELOW it down. So the
  # editors are driven BOTTOM-UP — staged edit, then marks, then brazen — and no
  # beat ever moves a surface a later beat still has to click. (The phase-1
  # toolchain pane used to go last for a sibling reason — its one-line disclaimer
  # set the side panel's width — but W13 deleted the gate and its pane with it.)
  # KEY: Ctrl+Shift+2 focuses the §11 Config tab. Since bl-1ca2 Config is a tab
  # focus rather than a toggled overlay, and a tab focus has a keyboard spelling
  # — so the steering rule prefers it and this beat spends no coordinate at all.
  # (It also survives the roster growing a row, which the old 37,124 did not.)
  # WAITED ON, never slept at, and it is a BEAT rather than a step because the
  # press missing is what everything below mis-drove: `focus_config` carries the
  # whole account (bl-fc3f). It also GATES the three Config-tab stages, and that
  # is the other half of the same ruling — a stage that cannot be driven must not
  # be driven anyway. `s5_editors` clicks four derived points into that column;
  # with the centre elsewhere `locate.sh` refuses them (so no click is aimed at
  # another surface) but the refusal is an exit, and stories.sh runs under
  # `set -e` — which would take the run out mid-stage, before the verdict, with
  # the seat still claimed and the engine still up. The gate keeps the tail: the
  # run goes red on this row, says which stages did not run, and still reports.
  if focus_config "$wid" "$out/s5-03-config.png"; then
    pass "S5 fixture: the Config tab is focused"
    s5_config_branch "$wid" "$out" "$data"
    s8_marks "$wid" "$out" "$data"
    s5_editors "$wid" "$out" "$data"
  else
    fail "S5 fixture: the Config tab is focused" \
      "centre elsewhere; S5-T5, S8-T4, S5-T3/T4 not driven"
  fi
  # Leave the Config tab — Ctrl+Shift+1 is the Conversation tab, and pressing the
  # roster entry a second time no longer leaves anything (a tab focus is not a
  # toggle, bl-1ca2) — and re-expand the balls section: the residual rows no
  # longer *click* into it (they cross the §8.5 boundary), but their screenshots
  # are the visual half of what those gestures did, and a collapsed section
  # paints none of it. The collapse is persisted, so the expand is landable too.
  "$drive" key "$wid" ctrl+shift+1 ; sleep 2
  expand_balls() { "$drive" bare "$wid" b; }
  until_landed expand_balls balls_expanded \
    && pass "S3 fixture: balls re-expanded" \
    || fail "S3 fixture: balls re-expanded" "still collapsed"
  s3_residuals "$wid" "$out" "$data"
  "$drive" stop "$pid" ; sleep 1
  # Over the window's finished trail, and HERE rather than at the tail: the last
  # S8 beat is a severability proof that `rm -rf`s the world, so after it there
  # is no `$ops` left to read and the assertion would fail for the one reason
  # that says nothing about spend. The engine is stopped, so the trail is final;
  # the two S8 stages below drive `yog env`/`yog exec` and no window at all.
  no_wire_spend \
    && pass "S5 fixture: the run spent nothing on the wire" \
    || fail "S5 fixture: the run spent nothing on the wire" "a lernie prompt ran"
  s8_hatches "$data"
  s8_nesting "$data" "$ambient_before"

  verdict "$out"
}
