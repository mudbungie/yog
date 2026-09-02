#!/bin/bash
# fixture.sh — the one-command door over `yog fixture` (bl-8741): lay a named
# world state, boot an engine on it, print the address, and tear the lot down
# on the way out.
#
# The verb is the portable half and this is the convenience: `yog fixture
# <state>` writes the world and answers one JSON object, and everything below
# is the three lines a consumer would otherwise write around it. A harness in
# another repository should spend the VERB — it owns its own engine process and
# its own teardown, which is the whole reason the verb does not park.
#
# Usage:
#   scripts/fixture.sh                 list the states
#   scripts/fixture.sh <state>         lay it, serve it, print the address
#
# `FIXTURE_ROOT` and `WIRE_HOST`/`WIRE_PORT` are the verb's own readings and
# pass straight through. `yog` is taken from PATH, so `make fixture` prefixes
# this checkout's release build onto it — the prefix is what proves the binary
# in hand rather than whatever is installed (`scripts/drive/drive.sh`'s rule,
# and for its reason).
set -eu

yog=${YOG:-yog}
state=${1:-}
if [ -z "$state" ]; then
  exec "$yog" fixture
fi

laid=$("$yog" fixture "$state")

# One reader for the whole answer: the verb's contract is a JSON object, and a
# shell that greps it would be a second, worse parser of a document that
# already has one.
read -r root address anchors chain key <<EOF
$(printf '%s' "$laid" | python3 -c '
import json, sys
d = json.load(sys.stdin)
print(d["root"], d["address"], d["anchors"], d["chain"], d["key"])')
EOF

# **Everything this script started, it stops.** The engine is a child and the
# root is a scratch tree the verb wiped to make; a run that abandoned either
# would leave a world nobody looks at again holding a port nobody can reuse.
engine=""
cleanup() {
  [ -z "$engine" ] || kill "$engine" 2>/dev/null || true
  [ -z "$engine" ] || wait "$engine" 2>/dev/null || true
  rm -rf "$root"
}
trap cleanup EXIT INT TERM

# The descriptors that make a streaming conversation read as a LIVE model call.
# Liveness is derived from an open fd and a held executor lock, so no tree on
# disk can be one by itself — the verb names the two paths and this shell is
# the process that holds them, because it is the one that outlives the lay.
# A directory is the lock; a file is the writer.
while IFS= read -r path; do
  [ -n "$path" ] || continue
  if [ -d "$path" ]; then
    exec {fd}<"$path"
  else
    exec {fd}>>"$path"
  fi
done <<EOF
$(printf '%s' "$laid" | python3 -c '
import json, sys
for p in json.load(sys.stdin)["hold"]:
    print(p)')
EOF

XDG_DATA_HOME="$root" "$yog" >"$root/engine.log" 2>&1 &
engine=$!

# The engine is up when it answers, not when the process exists. Deadlined,
# and the deadline is the engine's own liveness too: a boot that died has
# nothing to wait for.
if ! timeout 60 bash -c '
  until XDG_DATA_HOME="$1" "$2" gesture /workspaces >/dev/null 2>&1; do
    kill -0 "$3" 2>/dev/null || exit 2
    sleep 0.5
  done' _ "$root" "$yog" "$engine"; then
  echo "fixture: the engine never came up; its log:" >&2
  tail -20 "$root/engine.log" >&2
  exit 1
fi

cat <<EOF
state    $state
address  $address
anchors  $anchors
chain    $chain
key      $key
root     $root
log      $root/engine.log

dial it with those three files; Ctrl-C stops the engine and removes the root.
EOF

wait "$engine"
