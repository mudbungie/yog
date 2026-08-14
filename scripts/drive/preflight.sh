#!/bin/bash
# preflight.sh — name EVERY missing prerequisite at once, before anything is
# driven (bl-56d5).
#
# The harness used to discover its host the hard way: a run claimed a seat, went
# quiet for ten seconds, and died on the first `xdotool` or the first `ffmpeg`
# with a bare "command not found" — one missing tool per attempt, a full seat
# claim spent to learn each. This checks the whole contract in under a second and
# reports the whole of it, so a box is made ready in one pass instead of four.
#
# It reports rather than guesses: every subject prints its resolved path or
# version, which is also the "Host tool tuple" line every drive log carries —
# `logskel.sh` reads the same facts from the same probes.
#
# THE SUBJECTS ARE WHAT THE SCRIPTS ACTUALLY CALL, verified against them, not
# what an X11 harness is assumed to want. Capture is `ffmpeg -f x11grab`
# (yogdrive.sh `shot`), so ImageMagick `import`/`convert`, `scrot` and `xwd` are
# NOT subjects — the harness has never invoked one, and a box without them
# drives the full ladder green.
#
# A HOST TOOL IS NOT THE ONLY WAY A RUN CAN BE UNREADY (bl-49c6). Since the
# blast-radius ruling (DESIGN §16.2) brazen's config, credentials and
# model cache live inside the **per-workspace wall**,
# `<world>/walls/<name>/brazen/*`, and a newborn workspace's wall is an EMPTY
# DIRECTORY. So a scratch world's provider table is brazen's shipped defaults
# until `seed_wall` (wall.sh) copies the host's config and credentials in —
# which it does with the world seed, before the launch, keyed by §3.1's bootstrap
# constant `home` (bl-1851). That is a *fixture* fact this file checks in under a
# second and reports below.
#
# It is ADVISORY, not required (bl-00ee). It was required while the §9.2 birth
# gate judged the seeded birth template against the unborn workspace's wall —
# one missing row reddened every beat of every run with nothing ever created.
# That gate is retired: birth judges nothing about providers, because the wall
# it would judge against does not exist until the birth creates it. A row the
# wall lacks now costs the WIRE beats and nothing else, so it is reported beside
# the credentials, in the same tier, with the same consequence.
#
# Usage: preflight.sh          — report, exit non-zero if a REQUIRED subject is missing
set -u

req_missing=0

# A required tool: absent means no run of any verb can finish.
tool() { # tool <bin> <what uses it>
  if p=$(command -v "$1" 2>/dev/null); then
    printf '  %-14s OK       %s\n' "$1" "$p"
  else
    printf '  %-14s MISSING  %s\n' "$1" "$2"
    req_missing=$((req_missing + 1))
  fi
}

# A required file: the world seed stories.sh copies into every scratch world.
seedfile() { # seedfile <path> <what it carries>
  if [ -f "$1" ]; then
    printf '  %-14s OK       %s\n' "$(basename "$1")" "$1"
  else
    printf '  %-14s MISSING  %s\n     want: %s\n' "$(basename "$1")" "$2" "$1"
    req_missing=$((req_missing + 1))
  fi
}

real_world="$HOME/.local/share/yog/world/lernie"
# The host's own brazen state, which is a FIXTURE SOURCE now and not a
# destination: nothing yog spawns reads it since §16.2's ruling. The harness
# copies out of it into each scratch workspace's wall (`seed_wall`, wall.sh),
# so both paths are anchored on `$HOME` exactly as `real_world` is — the clean
# room overrides the XDG roots onto its own scratch, and a source that folded
# off those would vanish inside the room it is meant to supply.
host_creds="$HOME/.local/share/brazen/credentials"
host_config="$HOME/.config/brazen/config.toml"

# The provider rows a workspace born in a scratch world actually has. ASKED,
# never assumed: the answer comes from the same `yog` the drive is about to
# launch, through a throwaway wall that is exactly the shape a newborn
# workspace's is (an empty dir), so this reports brazen's real shipped table
# rather than a copy of it kept here to drift.
wall_rows() {
  d=$(mktemp -d -t yog-preflight-wall.XXXXXX) || return 1
  YOG_WALL="$d" yog bz --list-providers --json 2>/dev/null \
    | tr ',' '\n' | sed -n 's/.*"name":"\([^"]*\)".*/\1/p'
  rm -rf "$d"
}

# The role→provider names the world seed's birth template declares — the rows
# every conversation in a scratch workspace dispatches through, so the rows its
# wall has to end up carrying for a wire beat to land.
template_rows() {
  sed -n 's/^ *provider: *\([^ ]*\) *$/\1/p' \
    "$real_world/template/providers.yaml" 2>/dev/null | sort -u
}

echo "preflight — host prerequisites for scripts/drive/"
echo
echo "required — every run verb dies without these:"
tool Xvfb    "the isolated per-run seat (yogdrive.sh seat) — install xvfb"
tool xdotool "every windowed gesture: focus, type, key, click — install xdotool"
tool ffmpeg  "screen capture (yogdrive.sh shot, -f x11grab) — install ffmpeg"
tool ffprobe "the shot's own width and height (locate.sh) — ships with ffmpeg"
tool python3 "the step-kind reader (beats_s7.sh), locate.sh's rule scan, and the log skeleton's beat table"
tool git     "the scratch project fixture, and the clean room's floor"
tool yog     "the binary under drive — build it (make release) and put target/release first on PATH"
seedfile "$real_world/models.yaml" \
  "the model roster; also lernie's seeded marker, so its absence changes what S0 asserts"
seedfile "$real_world/template/providers.yaml" \
  "the workspace-birth template — the role→provider rows the wall section below reports"

echo
echo "wire — advisory: a beat that SPENDS needs each row below to reach the"
echo "       workspace's wall AND to be signed in there; run-s5s8 (zero model"
echo "       calls) stands without either. Nothing here blocks a run: since"
echo "       bl-00ee retired the §9.2 birth gate a workspace is born whatever its"
echo "       template names, and a row its wall lacks surfaces at the first"
echo "       dispatch (§8.3), not as a refusal to create anything. Both files are"
echo "       FIXTURE SOURCES — seed_wall copies them into"
echo "       <world>/walls/home/brazen/ with the world seed, before the launch"
echo "       (§16.2; §3.1 fixes the bootstrap leaf at 'home'):"
declared=$(template_rows)
if [ -e "$host_config" ]; then
  printf '  %-14s OK       %s\n' config.toml "$host_config"
else
  printf '  %-14s ABSENT   %s\n' config.toml "$host_config"
  echo "     a seeded wall then carries brazen's shipped rows only"
fi
if [ -z "$declared" ]; then
  echo "  (no rows declared — the seed template above is absent, reported there)"
elif ! command -v yog >/dev/null 2>&1; then
  echo "  (rows not judged — no yog above to ask for a fresh wall's table)"
else
  # ASKED, never assumed: the shipped table comes from the same `yog` the drive
  # is about to launch, through a throwaway wall exactly the shape a newborn
  # workspace's is, rather than a copy kept here to drift.
  rows=$(wall_rows)
  for p in $declared; do
    if printf '%s\n' "$rows" | grep -qx -- "$p"; then
      printf '  %-14s OK       %s\n' "$p" "a fresh wall's table ships this row"
    else
      printf '  %-14s VIA CFG  %s\n' "$p" \
        "not a shipped row — it reaches the wall only through config.toml above"
    fi
    f="$host_creds/$p.json"
    if [ -e "$f" ]; then
      printf '  %-14s OK       %s\n' "  sign-in" "$f"
    else
      printf '  %-14s ABSENT   %s\n' "  sign-in" "$f"
      echo "     no live model call through this row can succeed; a run still drives"
      echo "     every wire-free beat"
    fi
  done
fi

echo
if [ "$req_missing" -gt 0 ]; then
  echo "preflight: $req_missing required prerequisite(s) missing — nothing was driven." >&2
  exit 1
fi
echo "preflight: every required prerequisite present."
