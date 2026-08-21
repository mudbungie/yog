#!/bin/bash
# beats_s5.sh — the S5 Operator beats of stories.sh (which sources this; not an
# entry point of its own): the §9.1 brazen editor's two negatives (S5-T3/T4) and
# the §9.3 config-branch shim (S5-T5). Split for the repo's 300-line cap on the
# seam this file's header always drew — the world-A runner that fires these and
# the S8 beats is `beats_s5_run.sh`, its fixtures and predicates are
# `beats_s5_fixture.sh` (bl-9df2, bl-fc3f). It uses stories.sh's seat handle
# (`$drive`) and the shared assertion helpers, and every point it presses is
# derived per run by `locate.sh`, which refuses the frame outright unless the
# centre is on Config.

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
      --ws "$(basename "$ws_root")" \
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
