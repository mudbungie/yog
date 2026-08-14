#!/bin/bash
# wall.sh — the §16.2 WALL FIXTURE, and the only tier that lays state rather
# than reading it. Sourced by harness.sh, spent by stories.sh's `seed` and by
# `beats_s5.sh`'s `wall_config`; it defines no beat, no predicate and no verb.
#
# Split out of harness.sh at the 300-line cap (bl-f16e). The seam is real and
# it restores that file's own header: harness.sh is "the assertion helpers, the
# two waiting primitives, the per-run seat, and the verdict in BOTH its halves"
# — a fixture that COPIES A HOST CREDENTIAL INTO A SCRATCH WORLD is none of
# those, and it was parked there only because bl-49c6 had nowhere else to put
# it. Everything below is unchanged; only its address moved.

# --- the workspace wall: the per-workspace brazen fixture -------------------
# `seed_wall <data> <workspace-name>`.
# Since the blast-radius ruling (DESIGN §16.2) brazen's config,
# credentials and model cache are the WORKSPACE's — `<world>/walls/<name>/brazen/`
# — and nothing yog spawns reads the host's own brazen state any more. A newborn
# wall is an EMPTY DIRECTORY, so a scratch workspace has no provider rows and no
# sign-ins until something puts them there. That used to be `yogdrive.sh launch`
# symlinking `$XDG_DATA_HOME/brazen/credentials`, a path no driven process has
# read since the ruling — the run launched green and died at its first model
# call (bl-49c6).
#
# A WALL IS KEYED BY A NAME, NOT BY A WORKSPACE THAT ALREADY EXISTS (bl-1851),
# and the name of the one every wire beat runs in is a CONSTANT. DESIGN §3.1:
# "The bootstrap names without asking. The empty-world start (§3.4) creates its
# workspace under the fixed default name `home` — a constant, not a config
# (severability: there is nothing to delete), and not a mint (the wordlist names
# conversations, §3.3)." Every run verb here opens on zero workspaces, so every
# wall this harness lays is `home`'s, and it is layable before yog is launched.
#
# SO IT IS LAID BEFORE THE LAUNCH, with the world seed (stories.sh `seed`), and
# that ordering is the whole of bl-1851. yog's bare start is ONE gesture — the
# `lernie new` and the detached `lernie prompt` fire within the same second — so
# a fixture laid "after the mint" is always later than the FIRST MODEL CALL, and
# every scratch-world run's first turn died with brazen's config error,
# `unknown provider openai-chatgpt`, while the beat asserting the reply read as
# a wire outage. Two independent runs, two days latent, one phantom each.
#
# Nothing about the product moved, and §9.2's gate stays retired. §16.2's "a
# wall is born empty" is the rule that YOG never inherits a machine's brazen
# state into a newborn sphere, and yog still never does; what the birth gate
# lacked was never a knowable name but anything of its own to put in the wall.
# This is the operator's own hand, and an operator's `home` wall is exactly
# where their `home` sign-in lives.
#
# The host paths are anchored on `$HOME`, like stories.sh's `real_world`, because
# they are FIXTURE SOURCES: the clean room overrides `XDG_CONFIG_HOME` /
# `XDG_DATA_HOME` onto its own scratch, and a source that folded off those would
# vanish inside the room it is meant to supply.
#
# DESIGN §3.1's bootstrap constant, `src/names/mod.rs::DEFAULT_NAME` in the tree.
BOOTSTRAP_WS=home
wall_dir() { printf '%s/yog/world/walls/%s/brazen' "$1" "$2"; }
seed_wall() {
  [ -n "${2:-}" ] || return 0
  wall=$(wall_dir "$1" "$2")
  mkdir -p "$wall"
  if [ -f "$HOME/.config/brazen/config.toml" ]; then
    cp "$HOME/.config/brazen/config.toml" "$wall/config.toml"
  fi
  if [ -d "$HOME/.local/share/brazen/credentials" ]; then
    mkdir -p "$wall/credentials"
    cp "$HOME"/.local/share/brazen/credentials/*.json "$wall/credentials/" 2>/dev/null || true
  fi
  echo "wall: $wall"
}
