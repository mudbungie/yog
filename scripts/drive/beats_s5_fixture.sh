#!/bin/bash
# beats_s5_fixture.sh — world A's fixtures and predicates, split from
# beats_s5.sh at the repo's 300-line cap (bl-9df2) on the seam the file
# already declared: what the run asserts WITH (paths, predicates) against
# what it drives (the beats). Sourced by stories.sh ahead of the other two
# world-A files, `beats_s5_run.sh` (the runner) and `beats_s5.sh` (the beats),
# which are its only consumers.

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
# `md5of` and `file_has` were here and are now in predicates.sh: `beats_s8.sh`
# spends the first and `beats_s6.sh` the second, and a predicate two runners
# assert on has one home — the same rule that moved `seen_kind` (bl-2d45,
# bl-f16e). Whoever files a beat here reaches for the shared spelling.
# a `config/<name>` branch of the workspace's bare repo carries <file> with <text>
config_branch_has() {
  git --git-dir="$1/repo.git" show "config/$2:$3" 2>/dev/null | grep -q -- "$4"
}
# The collapse override is a SET in the per-seat pane document (§4.1 as landed,
# bl-8bbc: `collapsed` is a pane-of-glass fact, so it lives in
# clients/yog-window/pane.json, never ui.json — bl-9df2): expanding removes the
# key, so the expand is as landable as the collapse — the entry is simply gone.
balls_expanded() { ! grep -q '"balls"' "$pane" 2>/dev/null; }
# The run's own contract, asserted instead of described (bl-fc3f): world A binds
# a ball, drives three editors and NEVER prompts a model — "the whole run spends
# NOTHING on the wire", this file's neighbour's header. BOTH halves, because the
# negative alone is satisfied by a world where nothing ran at all: the trail must
# carry rows AND none of them may be a `litany prompt`.
no_wire_spend() { [ -s "$ops" ] && [ "$(verb_count prompt)" = 0 ]; }

# --- world-A navigation -----------------------------------------------------
# Focus the §11 Config tab and PROVE it (bl-fc3f). `yogdrive.sh key` is a
# `windowfocus` and then an XTEST press 0.2 s later, so on a loaded box the press
# can be injected before the focus has arrived and land nowhere: in one run of
# two the centre stayed on Conversation. The fixed sleep that used to follow
# proved nothing, and the frame it left was handed to `locate.sh brazen` as if it
# were the §12 Config column — the rules are the same family in both frames — so
# S5's marker was typed into the COMPOSER and clicked into a `litany prompt`,
# spending on the wire in the one run whose contract is that it spends nothing.
#
# Focus is per-instance RAM (§13.1), so no file can witness this and the frame
# must: the strip paints the selected tab FILLED, and `locate.sh centertab` reads
# back the digit that focuses it — the same 2 the key spells. That satisfies
# `until_landed`'s two requirements exactly — the predicate is monotone (nothing
# else here leaves the tab) and the gesture is a no-op when it lands twice (a tab
# focus is not a toggle since bl-1ca2). The poll shoots into the beat's own
# evidence frame, so the picture that proves the beat is the picture it was
# judged on, and a run that never got there keeps the frame that says why.
focus_config() {
  wid=$1 ; cfg_shot=$2
  press_config() { "$drive" key "$wid" ctrl+shift+2; }
  on_config() {
    "$drive" shot "$wid" "$cfg_shot"
    [ "$("$here/locate.sh" centertab "$cfg_shot" 2>/dev/null)" = 2 ]
  }
  until_landed press_config on_config
}

