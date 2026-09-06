#!/bin/sh
# Seat the NATIVE engine deployment on THIS box (bl-8ea9): `make deploy-local`.
#
# It takes no argument and reaches no network but the registry. `seat.sh` is the
# server recipe — an ssh destination, an image carried over the channel, a
# container unit — and it cannot seat the box it is run ON: the box most likely
# to run a native engine is a workstation, which is the box least likely to be
# running an sshd, and it cannot ssh to itself. This is the same three acts
# without the hop.
#
# **It seats a timer; it does not deploy a build.** Nothing is compiled here.
# The box installs from crates.io on its own schedule from then on, which is the
# difference between this and `seat.sh`, where the image is the unit of install
# and a human carries it. A native box's unit of install is a published version,
# and the registry already serves it.
#
# **One unit name, two shapes.** The unit is seated AS `yog.service`, over
# whatever `yog.service` was, because a box runs one engine over one world and
# two managers converging the same data root against each other is the failure
# this prevents. Re-running is therefore also how a box seated on the container
# shape adopts this one, and `seat.sh` is how it goes back — that file says the
# same of the binary units it superseded.
#
# Idempotent, and the upgrade path: re-run it to move this box to this
# checkout's units and reconciler.
#
# **Its last act runs the reconciler once, synchronously, and its exit code is
# this script's** — so seating a box either ends with the newest release
# installed and serving, or says why, rather than reporting that a timer was
# enabled and leaving the first real answer an hour away.
set -eu

self=${0##*/}
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
die() { printf '%s: %s\n' "$self" "$*" >&2; exit 1; }

command -v systemctl >/dev/null 2>&1 || die 'no systemctl: this recipe seats systemd user units'
command -v cargo >/dev/null 2>&1 || die 'no cargo on PATH: the reconciler installs from crates.io'
command -v curl >/dev/null 2>&1 || die 'no curl on PATH: the reconciler reads the registry index'

# The engine's substrate commits, and git refuses against an identity-less
# process. The container unit passes this in by name out of `deploy.env`; a
# native engine reads the operator's own `~/.gitconfig`, so the check is that
# there IS one rather than a file to write.
git config --get user.name >/dev/null 2>&1 && git config --get user.email >/dev/null 2>&1 \
    || die 'no git user.name/user.email on this box; the engine commits and git refuses without one'

say 'installing the units and the reconciler'
mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
# To a temp name and then `mv` into place: the reconciler may be running right
# now (the timer is armed from a previous seating), and a copy truncates before
# it writes. rename(2) in the same directory means a running shell reads
# whole-old or whole-new and never a half file.
install -m 0755 "$here/local-reconcile.sh" "$HOME/.local/bin/.yog-reconcile.tmp"
mv -f "$HOME/.local/bin/.yog-reconcile.tmp" "$HOME/.local/bin/yog-reconcile"
install -m 0644 "$here/yog-local.service" "$HOME/.config/systemd/user/yog.service"
install -m 0644 "$here/yog-local-reconcile.service" \
    "$HOME/.config/systemd/user/yog-reconcile.service"
install -m 0644 "$here/yog-local-reconcile.timer" \
    "$HOME/.config/systemd/user/yog-reconcile.timer"

# **Lingering, or the engine stops at logout** — and a box that reboots without
# it comes back with no engine at all. `enable-linger` is the one act here that
# is not a file copy, and it is what makes the unit a deployment rather than a
# convenience.
say 'enabling lingering so the engine outlives a logout'
loginctl enable-linger "$(id -un)" >/dev/null 2>&1 \
    || say 'could not enable lingering (the engine will stop at logout)'

say 'arming the units'
# `reset-failed` before the enable, for the reason `reconcile.sh` states: a unit
# that hit its start limit refuses to start until the interval expires, and a
# start without this is a no-op that reads as a success.
systemctl --user daemon-reload
systemctl --user reset-failed yog.service yog-reconcile.service 2>/dev/null || true
systemctl --user enable yog.service yog-reconcile.timer
systemctl --user start yog-reconcile.timer

# The verification, and it is the reconciler itself rather than a probe of one.
# `systemctl --user start` blocks on a `Type=oneshot` unit and exits non-zero
# when it fails, so this is a real end-to-end run — the index reached, the
# version compared, the build done if there was one, the unit started under it —
# and not a status print. A first-ever seating builds the engine here, which is
# minutes.
#
# It is also what STARTS the engine: `enable` above arms the unit for the next
# boot and does not run it, and the reconciler's `failed`/`inactive` arms leave a
# stopped unit alone on purpose. So the first pass installs the binary and the
# start below puts the box in the state the timer will then hold.
say 'running the first reconcile (a first build is not quick)'
systemctl --user start yog-reconcile.service \
    || { journalctl --user -u yog-reconcile.service --no-pager --lines=30 2>&1 \
             | sed 's/^/  | /' >&2
         die 'the first reconcile failed (the timer is armed; it will retry)'; }
journalctl --user -u yog-reconcile.service --no-pager --lines=5 -o cat 2>/dev/null || true

say 'starting the engine'
systemctl --user restart yog.service

# One bounded sleep, no polling loop — the discipline `verify.sh` states. Longer
# than the unit's `RestartSec=5s`, so an engine that cannot start has already
# been restarted at least once by here and this reads a settled box.
sleep 8
state=$(systemctl --user is-active yog.service 2>/dev/null || true)
[ "$state" = active ] || {
    journalctl --user -u yog.service --no-pager --lines=20 2>&1 | sed 's/^/  | /' >&2
    die "yog.service is '$state', not active"
}
# `is-active` on its own is worth nothing — it says a process exists, not that
# the engine serves. The boundary answering is the fact a deploy exists to
# establish, and it needs no certificate and no seat.
"$HOME/.local/bin/yog" gesture '{"op":"workspaces"}' >/dev/null 2>&1 \
    || die 'the unit is active but the engine does not answer the boundary'

say "seated: $("$HOME/.local/bin/yog" --version) is serving, and this box tracks released versions hourly"
