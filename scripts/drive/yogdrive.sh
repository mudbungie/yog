#!/bin/bash
# yogdrive — drive a real yog instance on an ISOLATED, PER-RUN Xvfb seat.
# Never touches the user's seat. XTEST works natively (no compositor).
# Usage: yogdrive.sh seat                        -> claims a display, prints ":N"
#        yogdrive.sh launch <scratch-data-dir>   -> prints "PID WID"
#        yogdrive.sh shot <wid> <out.png>
#        yogdrive.sh type <wid> <text...>   | key <wid> <keysym...>
#        yogdrive.sh click <wid> <x> <y>    | stop <pid>
#        yogdrive.sh unseat                      -> tears this run's seat down
#
# THE SEAT IS PER RUN, NOT PER BOX. A hardcoded `:99` is a singleton seat: two
# drives running at once shared it and stole each other's window focus
# mid-typing, so message payloads landed doubled and truncated (bl-4132, the W14
# clean-room run). A run therefore *claims* a seat once and every later
# invocation inherits it through `YOG_SEAT` in the environment — one variable,
# no state file of ours. No verb here falls back to a default display: a silent
# default is exactly the singleton this replaces.
#
# `click` is the only coordinate-bearing verb and it is the last resort: drive a
# gesture through its DESIGN §11 keyboard binding with `key` wherever one exists
# (see stories.sh's STEERING RULE for the table), because coordinates regress on
# every layout change. Clicks are for PICKS — which row, member, or form field —
# where the binding table deliberately stops (§11 rule 2).
set -eu
cmd=$1; shift

if [ "$cmd" = seat ]; then
  # Xvfb picks the free display ITSELF (`-displayfd`) and reports it back. A
  # probe-then-start races — two runners claiming in the same instant both see
  # the same :N free, and the loser's server never starts — and the server's own
  # allocation cannot.
  fd=$(mktemp -t yogdrive-seat.XXXXXX)
  Xvfb -displayfd 3 -screen 0 2560x1700x24 3>"$fd" >"$fd.log" 2>&1 &
  server=$!
  n=""
  for _ in $(seq 1 40); do
    n=$(cat "$fd" 2>/dev/null) || true
    [ -n "$n" ] && break
    sleep 0.25
  done
  [ -n "$n" ] || { echo "Xvfb claimed no display in 10s (log: $fd.log)" >&2; exit 1; }
  for _ in $(seq 1 40); do
    DISPLAY=":$n" xdotool getdisplaygeometry >/dev/null 2>&1 && break
    sleep 0.25
  done
  DISPLAY=":$n" xdotool getdisplaygeometry >/dev/null 2>&1 \
    || { echo "Xvfb :$n never answered (log: $fd.log)" >&2; exit 1; }
  rm -f "$fd"
  # A self-allocating Xvfb writes no `/tmp/.X<n>-lock` (skipping it is part of
  # how it avoids the race), so the seat records its own pid under a name
  # derived from the display it got — one file, addressed by `YOG_SEAT` alone,
  # removed by `unseat`.
  echo "$server" > "/tmp/.yogdrive-X$n.pid"
  echo ":$n"
  exit 0
fi

: "${YOG_SEAT:?no seat — claim one first: export YOG_SEAT=\$(yogdrive.sh seat). The display is per run (bl-4132), never a default.}"
export DISPLAY="$YOG_SEAT"
# Never start a server here: a verb that silently conjured its own seat would
# drive an empty display and fail invisibly. A dead seat is a loud error.
if [ "$cmd" != unseat ] && ! xdotool getdisplaygeometry >/dev/null 2>&1; then
  echo "seat $DISPLAY is not reachable — re-claim with 'yogdrive.sh seat'" >&2
  exit 1
fi

case $cmd in
# `launch` hands over the scratch data root and NOTHING brazen-shaped. It used to
# symlink `$XDG_DATA_HOME/brazen/credentials` back at the host's; since the
# blast-radius ruling (§16.2) brazen's credentials are the
# WORKSPACE's — `<world>/walls/<name>/brazen/credentials` — so that link was a
# path no driven process read, and its presence made a credential-less run look
# ready (bl-49c6). The wall is laid by harness.sh's `seed_wall`, with the world
# seed and BEFORE this launch — §3.1 fixes the empty-world start's leaf at the
# constant `home`, and the mint and the first model call are one gesture, so
# there is no "after the mint" early enough to be a fixture (bl-1851).
launch)
  data=$1
  env -u WAYLAND_DISPLAY DISPLAY="$DISPLAY" XDG_DATA_HOME="$data" yog >"$data/yog.stdout" 2>"$data/yog.stderr" &
  pid=$!
  wid=""
  # 60 s, not 30: on a loaded box yog's first frame is minutes-scale slow to
  # arrive and a short window search reports "window not found" for a process
  # that is merely still starting.
  for i in $(seq 1 120); do
    for w in $(xdotool search --name '^yog$' 2>/dev/null); do
      if [ "$(xdotool getwindowpid "$w" 2>/dev/null)" = "$pid" ]; then wid=$w; break 2; fi
    done
    sleep 0.5
  done
  [ -n "$wid" ] || { echo "window not found for pid $pid" >&2; exit 1; }
  xdotool windowfocus "$wid" 2>/dev/null || true
  echo "$pid $wid"
  ;;
shot)  ffmpeg -y -loglevel error -f x11grab -window_id "$(printf 0x%x "$1")" -i "$DISPLAY" -frames:v 1 "$2" ;;
type)  wid=$1; shift; xdotool windowfocus "$wid"; sleep 0.2; xdotool type --delay 30 -- "$*" ;;
key)   wid=$1; shift; xdotool windowfocus "$wid"; sleep 0.2; xdotool key -- "$@" ;;
# `bare` = release, then press. A BARE §11 key is suppressed while a text box
# holds the keyboard, and DESIGN §11's focus discipline parks the keyboard in
# the composer after every operation — launch, opening a conversation, a click,
# a send, a dismissed modal — so holding it is the resting state, not the
# exception. One verb spells the idiom so no beat has to remember it. Escape is
# what egui spends surrendering that focus, and with nothing pending it does
# nothing else. Return and the combos never come through here: Escape would
# cancel a pending start goal, and a combo is not suppressed in the first place.
bare)  wid=$1; shift; xdotool windowfocus "$wid"; sleep 0.2
       xdotool key -- Escape; sleep 0.2; xdotool key -- "$@" ;;
click) xdotool mousemove --window "$1" "$2" "$3"; sleep 0.1; xdotool click 1 ;;
stop)  kill "$1" 2>/dev/null || true ;;
# Teardown addresses the seat by its display alone: the pid file `seat` left.
unseat) f="/tmp/.yogdrive-X${DISPLAY#:}.pid"
        [ -f "$f" ] && { kill "$(cat "$f")" 2>/dev/null || true; rm -f "$f"; } || true ;;
*) echo "unknown cmd" >&2; exit 1 ;;
esac
