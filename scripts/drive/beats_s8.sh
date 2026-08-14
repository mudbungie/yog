#!/bin/bash
# beats_s8.sh — the S8 Neighbour beats, fired from `run_s5s8` in beats_s5.sh
# (which owns world A). Sourced by stories.sh; not an entry point of its own.
#
# S8 is the only rung whose subject is the *seam* between yog's world and the
# operator's own tools, so two of its three beats need no window at all: `yog env`
# and `yog exec` are pure entrypoints of the yog binary (§8.4), and the nesting
# claim is a statement about paths on disk. The third — the per-agent
# task-branch knob (§16.3) — is a config-mode pane over the focused WORKSPACE,
# so since the per-agent ruling it needs only a focused sphere, not a
# project binding. It still runs in world A, where one is focused anyway.

# --- S8-T4: the task-branch knob -------------------------------------------
# Each agent tracks on a balls SPACE of its own (DESIGN §16.3, the
# per-agent ruling): the knob's subject is the focused WORKSPACE, not a project,
# and its value is a branch name. So the assertions are the reply the boundary
# hands back (`yog gesture` is its own receipt) plus the one file the write
# lands in — balls' own `tasks_branch`, inside that agent's space — plus
# `ui.json` byte-identical across both applications: the policy lives in balls'
# own config key, which is exactly why removing the space would delete config
# and not code (§16.3).
#
# The knob no longer runs `bl` at all, so there is no ops row of `bl conf` to
# look for. That is the point of the re-key: `bl conf set task-branch` writes a
# LANDING, which belongs to a clone and therefore to a project, and could never
# bind an agent (bl-e47b).
s8_marks() {
  wid=$1 ; out=$2 ; data=$3
  ws=$(ls -d "$data"/yog/workspaces/*/ | head -1)
  ws=${ws%/}
  branch_file="$data/yog/world/walls/$(basename "$ws")/marks/balls/config.toml"
  ui_hash=$(md5of "$ui")

  # NO CLICK. `Read current` used to be pressed here at a measured (308,511),
  # tagged "NO SPELLING AT ALL — kept only as the visual half". That tag expired
  # when bl-0164 landed `/marks` bare, and the steering rule's second rung takes
  # the site outright (bl-5cce): the button's own body constructs
  # `Query::Marks`, which is the variant this line reaches — one implementation,
  # two serializations, and the line is the one that answers. The pixel was also
  # already wrong: the marks pane sits BELOW brazen's in the §12 Config column,
  # so bl-5410's seven wrapped provider rows pushed it ~119 px down, the same
  # move bl-b9f2 caught one pane higher. Nothing noticed, because the click's
  # outcome was never asserted — only the reply below ever was.
  #
  # A bare `/marks` reads and changes nothing; balls' default answers a space
  # nothing has been written into.
  gesture "$data" "/marks" --ws "$(basename "$ws")" \
    && grep -q '"branch":"balls/tasks"' "$out/gestures.jsonl" \
    && pass "S8-T4 marks: a bare line reads balls' default branch" \
    || fail "S8-T4 marks: a bare line reads balls' default branch" "no branch in the reply"
  # The window as it stands the moment the boundary answered, with nothing driven
  # by pointer to get there. It shows the §12 Config column from its top — and
  # the marks pane is not in it, being three panes further down than one screen
  # holds, which is the whole demonstration: the retired pixel was landing inside
  # the *lernie* editor.
  "$drive" shot "$wid" "$out/s8-01-marks-read.png"

  # The amendment: one word, and it is the branch.
  gesture "$data" "/marks balls/agents/drive" --ws "$(basename "$ws")" \
    && grep -q 'tasks_branch = "balls/agents/drive"' "$branch_file" \
    && pass "S8-T4 marks: an amendment writes balls' own tasks_branch key" \
    || fail "S8-T4 marks: an amendment writes balls' own tasks_branch key" "no key in $branch_file"
  "$drive" shot "$wid" "$out/s8-02-marks-own-branch.png"

  # Non-vacuous in both directions: the gesture returned a verdict, so this
  # negative is a statement about a write that certainly happened — and the file
  # it is about must EXIST for "unchanged" to mean anything, since two absences
  # compare equal (bl-f16e).
  { [ -f "$ui" ] && [ "$(md5of "$ui")" = "$ui_hash" ]; } \
    && pass "S8-T4 marks: no yog-owned file written" \
    || fail "S8-T4 marks: no yog-owned file written" "ui.json moved or absent"

  # An unlawful branch refuses in the space's own words rather than writing one.
  # BOTH arms are spelled: as `… || pass …` this beat could only ever emit a PASS
  # row, so the one outcome it exists to catch — the landing branch accepted —
  # deleted the beat from the verdict instead of reddening it, and a ladder
  # counts rows it has, never rows it should have had (bl-f16e).
  if gesture "$data" "/marks balls/config" --ws "$(basename "$ws")"; then
    fail "S8-T4 marks: balls' landing branch is refused, not written" "the boundary accepted it"
  else
    pass "S8-T4 marks: balls' landing branch is refused, not written"
  fi
  grep -q 'tasks_branch = "balls/agents/drive"' "$branch_file" \
    && pass "S8-T4 marks: the refusal left the standing branch alone" \
    || fail "S8-T4 marks: the refusal left the standing branch alone" "the key moved"

  # Back to the project's board, so the world is left as it was found.
  gesture "$data" "/marks balls/tasks" --ws "$(basename "$ws")" \
    && grep -q 'tasks_branch = "balls/tasks"' "$branch_file" \
    && pass "S8-T4 marks: pointing at the project's board is the same one verb" \
    || fail "S8-T4 marks: pointing at the project's board is the same one verb" "no key"
  "$drive" shot "$wid" "$out/s8-03-marks-shared.png"
}

# --- S8-T3: the hatches -----------------------------------------------------
# `eval "$(yog env)"` drops a shell into the world and `yog exec <cmd…>` runs one
# command there — both pure entrypoints, neither a substrate spawn. The composed
# env overrides EXACTLY `LERNIE_HOME` and `XDG_STATE_HOME` (and, since W9, the
# `PATH` head) and names no sphere — the hatches hand out the WORLD, and the
# wall is one layer further in (§16.2), so `YOG_WALL` never appears either.
# Assertable line by line on the printed script.
s8_hatches() {
  data=$1
  script=$(XDG_DATA_HOME="$data" yog env)
  # Exactly the §16.2 override set, whatever that set currently is: `LERNIE_HOME`,
  # `XDG_STATE_HOME` and — since the batteries landed (§16.7 W9) — a `PATH` whose
  # HEAD is the world's own tools shim dir, so an agent's bare `bl` is yog's. What
  # must never appear is the anchor (`XDG_DATA_HOME` — nesting it would recurse)
  # nor anything workspace-shaped: no `YOG_WALL`, and no `BRAZEN_CONFIG`, since
  # brazen's config is the wall's and the world names no wall. That absence is
  # the assertion with teeth, because it is the one a leak would break.
  { [ "$(printf '%s\n' "$script" | grep -c '^export ')" = 3 ] \
    && printf '%s\n' "$script" | grep -q "^export LERNIE_HOME='$data/yog/world/lernie'$" \
    && printf '%s\n' "$script" | grep -q "^export XDG_STATE_HOME='$data/yog/world/state'$" \
    && printf '%s\n' "$script" | grep -q "^export PATH='$data/yog/world/tools:" \
    && ! printf '%s\n' "$script" | grep -q 'XDG_DATA_HOME\|BRAZEN_CONFIG\|YOG_WALL'; } \
    && pass "S8-T3 yog env: exactly the world override set" \
    || fail "S8-T3 yog env: exactly the world override set" "$script"
  seen=$(XDG_DATA_HOME="$data" yog exec --cwd "$data/proj" sh -c 'echo "$LERNIE_HOME|$PWD"')
  [ "$seen" = "$data/yog/world/lernie|$data/proj" ] \
    && pass "S8-T3 yog exec: argv runs in the world, at --cwd" \
    || fail "S8-T3 yog exec: argv runs in the world, at --cwd" "$seen"
}

# --- S8-T1/T2: one nested world, severable ----------------------------------
# The claim is about paths, so read them: the nested lernie home and the nested
# balls state both live under `$XDG_DATA_HOME/yog`, the project's balls clone is
# **there and not in the operator's ambient store** (the dir yog watches and the
# dir a spawned `bl` writes are one path), the ambient world is untouched, and
# `rm -rf` the nested dir takes the whole world with it and nothing else.
s8_nesting() {
  data=$1 ; ambient_before=$2
  # This beat deletes the world, so keep its ops trail beside the screenshots:
  # the run log quotes it, and a deleted trail cannot be read back.
  cp "$ops" "$out/ops.jsonl" 2>/dev/null || true
  enc=$(printf '%s' "$data/proj" | sed 's|/|%2F|g')
  { [ -f "$data/yog/world/lernie/models.yaml" ] \
    && [ -d "$data/yog/world/state/balls/clones/$enc" ] \
    && [ ! -d "$HOME/.local/state/balls/clones/$enc" ]; } \
    && pass "S8-T2 nesting: the project's balls clone is nested only" \
    || fail "S8-T2 nesting: the project's balls clone is nested only" "clone misplaced"
  # The ambient seed must BE there for "never moved" to be a claim about yog
  # rather than about an empty box: absent-then, absent-now compared equal, so
  # this passed on a host with no ambient world at all (bl-f16e). The
  # severability beat below already requires the ambient dirs, so demanding the
  # seed itself is the same host contract, stated where it is spent.
  { [ -f "$HOME/.local/share/yog/world/lernie/models.yaml" ] \
    && [ "$(md5of "$HOME/.local/share/yog/world/lernie/models.yaml")" = "$ambient_before" ]; } \
    && pass "S8-T1 nesting: the ambient world's own seed never moved" \
    || fail "S8-T1 nesting: the ambient world's own seed never moved" "ambient seed changed or absent"
  rm -rf "$data/yog"
  { [ ! -e "$data/yog" ] && [ -d "$HOME/.local/state/balls" ] \
    && [ -d "$HOME/.local/share/yog" ]; } \
    && pass "S8-T1 severability: rm -rf the world, ambient intact" \
    || fail "S8-T1 severability: rm -rf the world, ambient intact" "ambient damaged"
}
