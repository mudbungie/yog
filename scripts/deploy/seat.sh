#!/bin/sh
# Seat the headless deployment on a server (bl-bf35): `make deploy HOST=<host>`.
#
# HOST is an ssh destination and the ONLY parameter — no address, account or
# machine name is committed anywhere in this tree, which is both the leak
# gate's rule and the severability one: pointing this at a different box is a
# different argument, not an edit.
#
# Idempotent. Re-run it to push a changed unit or updater; re-running is also
# how you adopt a new deployment recipe on a box already running one.
#
# It installs no yog itself — it seats the reconciler and lets it do the one
# thing it exists to do. First run and every later run take the same path,
# which is the only way the first run is ever tested.
set -eu

host=${1:-}
[ -n "$host" ] || { echo "usage: ${0##*/} <ssh-host>" >&2; exit 2; }
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }

# The reconciler's own decision, checked before it is put somewhere unattended.
say "checking the restart decision"
"$here/yog-update" --self-test

say "seating the units on $host"
ssh "$host" 'mkdir -p "$HOME/.config/systemd/user" "$HOME/.local/bin"'
scp -q "$here/yog-update" "$host:.local/bin/yog-update"
ssh "$host" 'chmod +x "$HOME/.local/bin/yog-update"'
scp -q "$here/yog.service" "$here/yog-update.service" "$here/yog-update.timer" \
    "$host:.config/systemd/user/"

# Without lingering the user manager — and so the engine — dies at logout, which
# is the one failure that looks like "it just stopped overnight".
say "enabling lingering (the engine must outlive a login)"
ssh "$host" 'loginctl enable-linger "$USER"' \
    || echo "  ! could not enable lingering; run: sudo loginctl enable-linger \$USER" >&2

say "installing yog from the registry"
ssh "$host" 'systemctl --user daemon-reload && "$HOME/.local/bin/yog-update"'

# Seating the deployment is an explicit operator act and must leave the box
# running, so it clears a tripped start limit on the way in — `enable --now`
# will not, and a unit that hit its limit refuses to start until the interval
# expires.
say "starting the engine and arming the hourly check"
ssh "$host" 'systemctl --user reset-failed yog.service 2>/dev/null; \
    systemctl --user enable --now yog.service yog-update.timer'

say "state on $host"
ssh "$host" 'systemctl --user --no-pager --lines=0 status yog.service yog-update.timer 2>&1 | sed -n "1,6p;/Active:/p"'
