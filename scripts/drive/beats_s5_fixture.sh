#!/bin/bash
# beats_s5_fixture.sh — world A's fixtures and predicates, split from
# beats_s5.sh at the repo's 300-line cap (bl-9df2) on the seam the file
# already declared: what the run asserts WITH (paths, predicates) against
# what it drives (the beats). Sourced by stories.sh immediately before
# beats_s5.sh, which is the only consumer.

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
# The collapse override is a SET in the per-seat pane document (§4.1 as landed,
# bl-8bbc: `collapsed` is a pane-of-glass fact, so it lives in
# clients/yog-window/pane.json, never ui.json — bl-9df2): expanding removes the
# key, so the expand is as landable as the collapse — the entry is simply gone.
balls_expanded() { ! grep -q '"balls"' "$pane" 2>/dev/null; }

