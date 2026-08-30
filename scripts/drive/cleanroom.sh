#!/bin/bash
# cleanroom.sh — the DESIGN §16.7 W14 batteries proof, made re-runnable.
#
# It builds a room in which the ONLY substrate on `PATH` is one yog binary plus
# the system's own `git`/coreutils/X tools, points every version-fragile root at
# fresh scratch, asserts the scrub instead of assuming it, and then hands the
# room to `stories.sh` unchanged. The assertion is the point: `litany`, `bl` and
# `bz` are unresolvable inside the room, so a spawn that still reached for a host
# binary cannot silently succeed — it dies ENOENT and the beat goes red. That is
# the whole of W14's invariant ("it fails if any spawn still resolves a host
# binary"), and it is why this wrapper exists rather than a prose recipe in a
# drive log: the env IS the proof, so it lives in one executable place.
#
# Usage: cleanroom.sh <yog-binary> <scratch-root> <out-dir> [run|run-s3s4s6]
#   e.g. cleanroom.sh target/release/yog /tmp/w14 "$DRIVE_ROOT/w14-shots"
#   The out dir is evidence: it belongs under the drive root, outside the
#   checkout, with every other run's (QUALITY.md §3 step 6; bl-244f).
#
# WHAT STAYS AMBIENT, AND WHY (§16.2, deliberately, not by omission):
#   - `$HOME` alone — git's own identity, and the two FIXTURE SOURCES the room's
#     scratch world is seeded from: the world seed below, and the bootstrap
#     sphere's wall `seed_wall` lays beside it (wall.sh). A source is not a
#     destination: nothing the room drives reads the host's brazen state.
#   - NOTHING brazen-shaped is shared any more. The blast-radius
#     ruling (§16.2) moved brazen's config, credentials and model cache inside
#     the per-workspace wall, `<world>/walls/<name>/brazen/*`, which is inside
#     the room's own `XDG_DATA_HOME` — so the room's isolation is now structural
#     rather than a symlink it has to remember. The `$XDG_CONFIG_HOME/brazen`
#     link this file used to lay is gone with it: yog injects the wall's path as
#     `BRAZEN_CONFIG`, so brazen never reaches its own `$XDG_CONFIG_HOME`
#     fall-through and the link fed nothing (bl-49c6).
#   - `$XDG_CACHE_HOME` — the drive's own evidence root. Every root yog *nests*
#     (litany's home, balls' store layout, yog's own artifacts) is scratch,
#     which is what W14 asks for.
#   - stories.sh's world seed copies the ambient world's `models.yaml` +
#     `template/providers.yaml`, and `seed_wall` copies the host's brazen config
#     and credential files into the bootstrap sphere's wall. Those are config
#     files, never a binary: the model roster, the role rows, and the provider
#     row those roles name with the sign-in that answers for it. litany's own
#     in-process `prime` seeds an anthropic-only roster, so the seeded rows are a
#     *credential* choice — the same carve-out, one layer in.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
yogbin=${1:?usage: cleanroom.sh <yog-binary> <scratch-root> <out-dir> [verb]}
root=${2:?usage: cleanroom.sh <yog-binary> <scratch-root> <out-dir> [verb]}
out=${3:?usage: cleanroom.sh <yog-binary> <scratch-root> <out-dir> [verb]}
verb=${4:-run}

yogbin=$(cd "$(dirname "$yogbin")" && pwd)/$(basename "$yogbin")
rm -rf "$root"
mkdir -p "$root/bin" "$root/config" "$root/state"
ln -sfn "$yogbin" "$root/bin/yog"

# The room. `/usr/bin:/bin` carries git, sh, coreutils and the harness's
# Xvfb/xdotool/ffmpeg; the three substrate binaries live in ~/.local/bin and
# ~/.cargo/bin, which the room drops.
export PATH="$root/bin:/usr/bin:/bin"
export XDG_DATA_HOME="$root/data" XDG_STATE_HOME="$root/state"
export XDG_CONFIG_HOME="$root/config" LITANY_HOME="$root/litany-ambient-unused"

for t in litany bl bz; do
  if p=$(command -v "$t" 2>/dev/null); then
    echo "clean room BREACHED: $t resolves to $p" >&2
    exit 1
  fi
done
for t in yog git; do
  command -v "$t" >/dev/null || { echo "clean room lacks $t" >&2; exit 1; }
done
# The wall half of the same invariant, asserted in the same two directions
# (bl-49c6): the room must supply NO ambient brazen state, so a fold that still
# reached for `$XDG_CONFIG_HOME/brazen` — brazen's own fall-through, the one yog
# overrides with the wall's path — finds nothing and cannot silently succeed.
# The room's brazen state is the per-workspace wall under `XDG_DATA_HOME` and
# nowhere else, which the seeded run then proves by using it.
if [ -e "$XDG_CONFIG_HOME/brazen" ]; then
  echo "clean room BREACHED: ambient brazen config at $XDG_CONFIG_HOME/brazen" >&2
  exit 1
fi

echo "clean room:"
echo "  PATH=$PATH"
echo "  XDG_DATA_HOME=$XDG_DATA_HOME"
echo "  XDG_STATE_HOME=$XDG_STATE_HOME"
echo "  XDG_CONFIG_HOME=$XDG_CONFIG_HOME  (nothing brazen-shaped here, §16.2)"
echo "  LITANY_HOME=$LITANY_HOME  (yog overrides it per-spawn anyway, §16.2)"
echo "  brazen  -> $XDG_DATA_HOME/yog/world/walls/home/brazen  (the bootstrap wall, §16.2)"
echo "  yog=$(command -v yog) -> $yogbin"
echo "  litany/bl/bz: unresolvable"
exec "$here/stories.sh" "$verb" "$XDG_DATA_HOME" "$out"
