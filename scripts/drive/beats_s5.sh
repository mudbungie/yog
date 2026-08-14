#!/bin/bash
# beats_s5.sh — the S5 Operator beats of stories.sh (which sources this; not an
# entry point of its own), plus the world-A runner `run_s5s8` that fires them and
# the S8 beats in `beats_s8.sh`. Split for the repo's 300-line cap; it uses
# stories.sh's seat handle (`$drive`), world seed and assertion helpers, and
# beats_s3s4s6.sh's `in_world` / `seed_balls` project fixture.
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

# --- world A fixtures --------------------------------------------------------
# The focused workspace's brazen config (§9.1, §16.2's wall layout), over
# wall.sh's `wall_dir` so the layout has one spelling — and over its
# `BOOTSTRAP_WS`, because this world opens on zero workspaces and §3.1 fixes that
# start's leaf at the constant `home` (bl-1851: it was a lazy `find` on the
# premise that the leaf is minted, which is true of a §11 `w` sphere and of
# nothing this harness drives). A tiny, VALID body is seeded because
# `bz --config <tmp> --dump-config` is the Apply gate and the beats below need it
# to say yes once and no once — which is why world A OVERWRITES what the world
# seed laid: the host's real config is ~90 lines, and a draft that long makes the
# raw editor scroll, which is a wheel gesture nothing here wants to own.
wall_config() { printf '%s/config.toml' "$(wall_dir "$1" "$BOOTSTRAP_WS")"; }
brazen_scratch() {
  mkdir -p "$(dirname "$1")"
  printf '# yogdrive scratch brazen config (the workspace wall)\n' > "$1"
}

# --- world-A predicates -----------------------------------------------------
# `md5of` and `file_has` were here and are now in harness.sh: `beats_s8.sh`
# spends the first and `beats_s6.sh` the second, and a predicate two runners
# assert on has one home — the same rule that moved `seen_kind` (bl-2d45,
# bl-f16e). Whoever files a beat here reaches for the shared spelling.
# a `config/<name>` branch of the workspace's bare repo carries <file> with <text>
config_branch_has() {
  git --git-dir="$1/repo.git" show "config/$2:$3" 2>/dev/null | grep -q -- "$4"
}
# The collapse override is a SET in ui.json (§4.1): expanding removes the key, so
# the expand is as landable as the collapse — the entry is simply gone.
balls_expanded() { ! grep -q '"balls"' "$ui" 2>/dev/null; }

# --- the run ----------------------------------------------------------------
run_s5s8() {
  data=$1 ; out=$2
  mkdir -p "$out" ; rm -rf "$data" ; mkdir -p "$data"
  claim_seat
  ball=$(seed_balls "$data")
  ops="$data/yog/world/state/yog/ops.jsonl"
  ui="$data/yog/world/state/yog/ui.json"
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

  read -r pid wid < <("$drive" launch "$data")
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
  # so it is a landable gesture: `ui.json` records it and the next launch starts
  # narrow. `b` is §11's balls-section fold — one key, both directions, so the
  # re-expand below is the same key and neither has a y to drift.
  collapse_balls() { "$drive" bare "$wid" b; }
  until_landed collapse_balls file_has "$ui" '"balls"' \
    && pass "S5 fixture: balls collapsed (ui.json)" \
    || fail "S5 fixture: balls collapsed (ui.json)" "no collapse record"
  # Then RELAUNCH rather than click a tab. Three things fall out of one restart:
  # the un-sent goal draft dies with the instance (RAM until sent, §8.1 — so the
  # ball is bound with nothing spent on the wire), startup DERIVES the focus onto
  # the only workspace (§4.1), which every beat below needs, and the fresh process
  # lays the side panel out at its default width. A tab click would be a
  # coordinate whose landing nothing on disk can confirm (focus is per-instance
  # RAM, §13.1) and it drifts with the minted name's width; a restart cannot miss.
  "$drive" stop "$pid" ; sleep 2
  read -r pid wid < <("$drive" launch "$data")
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
  "$drive" key "$wid" ctrl+shift+2 ; sleep 2
  "$drive" shot "$wid" "$out/s5-03-config.png"
  s5_config_branch "$wid" "$out" "$data"
  s8_marks "$wid" "$out" "$data"
  s5_editors "$wid" "$out" "$data"
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
  s8_hatches "$data"
  s8_nesting "$data" "$ambient_before"

  verdict "$out"
}

# S5-T4 (hash-guard) and S5-T3 (brazen-validate-rejects), on the §9.1 editor —
# the only one of the three with a validator. The caller focuses the Config tab
# (Ctrl+Shift+2) before this runs. Every beat here is bracketed by an Apply that
# DOES land, and that is deliberate: "the destination did not move" is a
# negative, and a click that missed satisfies it vacuously. So the order is land
# / refuse / land / refuse — each refusal sits between two proofs that the very
# same point works.
#
# **Nothing below is a measured pixel any more (bl-b9f2).** Three balls in three
# weeks re-baselined the same three numbers (bl-2622 → bl-f8dc → this one), each
# time because deliberate surface work moved a row and a number measured against
# an older screenshot went on pointing where the row used to be — last bl-5410,
# which gave brazen's seven provider rows a second wrapped line apiece and pushed
# the fold 119 px down. So the four points pressed here are DERIVED, per run,
# from the frame about to be driven: `locate.sh` carries the whole argument.
s5_editors() {
  wid=$1 ; out=$2 ; data=$3
  # The wall's config (§16.2), laid by the runner before the pane read it.
  bzcfg=$(wall_config "$data")
  # The pane as it stands right now, with the fold still shut — the only frame
  # from which the fold's own row can be read (opened, the rule below it moves).
  "$drive" shot "$wid" "$out/s5-03a-pane.png"
  read -r fold_x fold_y box_x box_y apply_x apply_y reload_x reload_y \
    < <("$here/locate.sh" brazen "$out/s5-03a-pane.png")
  # CLICK (a VIEW — a FOLD): the "raw config.toml" header. A fold is the one
  # thing the steering rule names outright as having no boundary spelling to
  # prefer, so the pointer is lawful here. Opened ONCE: a fold is a toggle, so a
  # re-fire inside `until_landed` would shut what the first one opened. A miss
  # stays loud — with the fold shut there is no Apply to press, and the bracket
  # beat below goes red to say exactly that.
  "$drive" click "$wid" "$fold_x" "$fold_y" ; sleep 1
  "$drive" shot "$wid" "$out/s5-03b-raw-fold.png"
  # CLICK (a FRAME-ONLY entry, and deliberately so): Apply / Reload inside that
  # fold — derived like the two above, but for a control that is not a view.
  # NEITHER named spelling reaches it. `/config brazen` exists (bl-3f46) and is
  # the WRONG one: §8.5 keeps the file editors' Apply on `BrazenEditor` because
  # the pane holds a long-lived RAM draft with a load-time snapshot and the §9
  # hash guard is over THAT draft — "a deposit has no such draft… the guard
  # degenerates to the must-not-exist check a new file wants" — and S5-T4/S5-T3
  # are assertions about the draft. Nor does the §11 focus floor (bl-478d) carry
  # it, and bl-b9f2 re-asked that at the source rather than trusting the claim:
  # `form_ui::raw_editor` ends in `.code_editor()`, which is
  # `.font(Monospace).lock_focus(true)` in egui 0.29, and `lock_focus` sets the
  # box's `EventFilter { tab: true }` — so Tab AND Shift+Tab are both eaten as
  # indentation and the walk cannot step off the draft in either direction. It
  # was driven, not assumed: replacing these clicks with Tab-then-Space put a
  # literal tab in the draft and left Apply unpressed.
  # The retry owns BOTH steps — retype *and* Apply — since the predicate depends
  # on both, and a retype that missed leaves an Apply with nothing new to write
  # (bl-84f3: `until_landed` owns every step its predicate depends on).
  apply_a() { brazen_draft "$wid" "# yogdrive-marker-A" ; "$drive" click "$wid" "$apply_x" "$apply_y"; }
  until_landed apply_a file_has "$bzcfg" 'yogdrive-marker-A' \
    && pass "S5 brazen: Apply lands (the bracket for the negatives)" \
    || fail "S5 brazen: Apply lands (the bracket for the negatives)" "marker not on disk"
  "$drive" shot "$wid" "$out/s5-04-applied.png"
  # A file changed underneath (the other instance, or vi) must REFUSE the Apply
  # and say so rather than overwrite (§9.4/§5.4). Fired three times: this beat's
  # assertion is a NEGATIVE and one miss would satisfy it vacuously, and the
  # positives on either side prove the point itself works.
  printf '# changed underneath the editor\n' >> "$bzcfg"
  underneath=$(md5of "$bzcfg")
  for _ in 1 2 3; do
    brazen_draft "$wid" "# yogdrive-marker-B" ; "$drive" click "$wid" "$apply_x" "$apply_y" ; sleep 2
  done
  "$drive" shot "$wid" "$out/s5-05-hash-guard.png"
  { [ "$(md5of "$bzcfg")" = "$underneath" ] \
    && ! file_has "$bzcfg" 'marker-B'; } \
    && pass "S5-T4 hash-guard: stale Apply wrote nothing" \
    || fail "S5-T4 hash-guard: stale Apply wrote nothing" "file moved"
  # Reload re-diffs and the same Apply then LANDS (S5-T4's second half) — which is
  # also the second bracket around the refusal above.
  reload_apply() {
    "$drive" click "$wid" "$reload_x" "$reload_y" ; sleep 1
    brazen_draft "$wid" "# yogdrive-marker-C" ; "$drive" click "$wid" "$apply_x" "$apply_y"
  }
  until_landed reload_apply file_has "$bzcfg" 'yogdrive-marker-C' \
    && pass "S5-T4 hash-guard: reload then the same Apply lands" \
    || fail "S5-T4 hash-guard: reload then the same Apply lands" "marker not on disk"
  # A malformed draft CANNOT land: `bz` itself rejects the staged file, the
  # destination is byte-identical, and the draft stays in the box (§9.1). Fired
  # three times for the same vacuity reason as the hash-guard above.
  landed=$(md5of "$bzcfg")
  for _ in 1 2 3; do
    brazen_draft "$wid" "@@@ not toml" ; "$drive" click "$wid" "$apply_x" "$apply_y" ; sleep 2
  done
  "$drive" shot "$wid" "$out/s5-06-rejected.png"
  { [ "$(md5of "$bzcfg")" = "$landed" ] \
    && ! file_has "$bzcfg" '@@@'; } \
    && pass "S5-T3 brazen: malformed draft cannot land" \
    || fail "S5-T3 brazen: malformed draft cannot land" "destination moved"
}

# Replace the brazen draft wholesale. CLICK (a VIEW): the §9.1 text box inside
# the opened raw fold, at the point `s5_editors` derived — focusing an input box
# is §8.5's own example of what does not cross the boundary, and a draft is §5.3
# RAM, so neither has a spelling to prefer. `ctrl+a` then a retype keeps the
# draft one line, so the box stays at its `desired_rows(6)` height and the
# buttons under it never move.
brazen_draft() {
  "$drive" click "$1" "$box_x" "$box_y" ; sleep 1
  "$drive" key "$1" ctrl+a ; sleep 1
  "$drive" type "$1" "$2"
}

# S5-T5 — the config-branch shim (§9.3): the drafted file is staged and `lernie
# config <ws> <name>` is driven with yog itself as `$EDITOR`, so the only lawful
# writer of `config/*` does the write. Both halves asserted: the ops row and the
# branch that now carries the file.
s5_config_branch() {
  wid=$1 ; out=$2 ; data=$3
  ws_root=$(find "$data/yog/workspaces" -maxdepth 1 -mindepth 1 -type d | head -1)
  # The lineage Send is a boundary gesture since bl-3f46 (§8.5: "the lineage
  # Send, the marks buttons, the picker's selection — now construct a variant
  # and call `AppModel::dispatch`"), so the form's four coordinates are gone:
  # three text-box focuses and `Send (stage + lernie config)`, all measured
  # against a column whose every pane above them could push them down. The line
  # spells the destination as LEADING WORDS and takes the file's text verbatim
  # after them — which is the only way a config file is sayable at all, since
  # its whitespace is semantic. `default` is the branch the workspace already
  # has, so this is `lernie config`'s everyday ADVANCE of an existing lineage.
  "$drive" shot "$wid" "$out/s5-07-branch-before.png"
  gesture "$data" "/config branch default notes.md driven by yogdrive" \
      --ws "$ws_root" \
    && row_ok '"lernie","config"' \
    && pass "S5-T5 config-branch: lernie config exit 0" \
    || fail "S5-T5 config-branch: lernie config exit 0" "no clean config row"
  await config_branch_has "$ws_root" default notes.md "driven by yogdrive" \
    && pass "S5-T5 config-branch: staged file lands on config/default" \
    || fail "S5-T5 config-branch: staged file lands on config/default" "branch/file absent"
  # The shim copies ONLY the drafted files: the `descriptions/` the checkout
  # already carried is still there, untouched by the edit (S5-T5's other clause).
  git --git-dir="$ws_root/repo.git" ls-tree -r --name-only config/default \
    | grep -q '^descriptions/' \
    && pass "S5-T5 config-branch: descriptions/ survived the staged copy" \
    || fail "S5-T5 config-branch: descriptions/ survived the staged copy" "checkout files lost"
  "$drive" shot "$wid" "$out/s5-08-branch-listed.png"
}
