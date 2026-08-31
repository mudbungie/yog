#!/bin/sh
# Seat the headless deployment on a server (bl-bf35; containerized in bl-b973,
# reconciled with the tree in bl-c6e2): `make deploy HOST=<host>`.
#
# HOST is an ssh destination and the ONLY parameter — no address, account or
# machine name is committed anywhere in this tree, which is both the leak
# gate's rule and the severability one: pointing this at a different box is a
# different argument, not an edit.
#
# **It deploys the IMAGE.** `Containerfile` is the unit of install (DESIGN
# §10.1, README "The image"): this builds it here under the pinned toolchain,
# retags it with an immutable `yog:<version>-<short-commit>`, carries it over
# the ssh channel with `save | load`, and points the unit at that exact tag.
# Nothing is pushed to any registry — the ghcr package publishes only from this
# repo's release workflow at tag time, and a push is not undoable.
#
# **The hourly reconciler is RETIRED, and this script disables it.** It
# reconciled a cargo-installed binary against the crates.io index and read
# quiescence off the unit's own cgroup; against a container unit both facts are
# wrong in the direction that ACTS — it would see the installed binary differ
# from the running container and restart the unit under it. The reconcile
# question for an image is "is a newer image loaded", which nothing on the box
# can answer without a registry to poll, and there is deliberately no registry
# it may poll. So an upgrade is this script, run by a human, which is also the
# only shape in which the restart below is an operator's decision rather than
# an unattended one.
#
# Idempotent, and the upgrade path. Re-run it to move the box to the tip;
# re-running is also how a box seated on the old binary units adopts this one.
set -eu

host=${1:-}
[ -n "$host" ] || { echo "usage: ${0##*/} <ssh-host>" >&2; exit 2; }
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo=$(CDPATH= cd -- "$here/../.." && pwd)

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
die() { printf '%s: %s\n' "${0##*/}" "$*" >&2; exit 1; }

# The tag names a commit, so a dirty tree would make it a lie — and the whole
# point of an immutable tag is that the box can say what it is running.
git -C "$repo" diff --quiet HEAD 2>/dev/null \
    || die 'the worktree is dirty; the image tag names a commit and must not lie'

version=$(sed -n '/^\[package\]/,/^\[/{s/^version *= *"\([^"]*\)".*/\1/p;}' "$repo/Cargo.toml")
[ -n "$version" ] || die 'no version in Cargo.toml'
commit=$(git -C "$repo" rev-parse --short HEAD)
tag="yog:$version-$commit"

# The identity the box commits under is the identity of whoever deploys it,
# read from this checkout's own git config. It is never committed to the tree:
# an identity in a unit file is somebody's name in a public repo.
name=$(git -C "$repo" config user.name || true)
email=$(git -C "$repo" config user.email || true)
[ -n "$name" ] && [ -n "$email" ] \
    || die 'no git user.name/user.email here; the container commits and git refuses without one'

# `make image` builds under the pinned toolchain AND runs the image-side
# disclosure gate. Building here rather than on the box keeps the server free
# of a checkout, which was the reason it installed from the registry before.
say "building $tag"
make -C "$repo" --no-print-directory image
engine=$(command -v podman 2>/dev/null || command -v docker 2>/dev/null) \
    || die 'no podman and no docker on PATH'
"$engine" tag "yog:$version" "$tag"

# `save | ssh load` and no registry. The stream is the whole transfer; nothing
# is written to a third place that could then be pulled by anyone else.
say "loading $tag on $host"
"$engine" save "$tag" | ssh "$host" 'docker load'

say "seating the unit on $host"
ssh -n "$host" 'mkdir -p "$HOME/.config/systemd/user" "$HOME/.config/yog" \
    "$HOME/.local/share/yog" "$HOME/work"'
scp -q "$here/yog.service" "$host:.config/systemd/user/yog.service"

# The one generated file on the box: which image the unit runs, and the identity
# it commits under. Everything else about the deployment is in the unit.
say "pointing the unit at $tag"
ssh "$host" "cat > \$HOME/.config/yog/deploy.env" <<EOF
YOG_IMAGE=$tag
GIT_AUTHOR_NAME=$name
GIT_AUTHOR_EMAIL=$email
GIT_COMMITTER_NAME=$name
GIT_COMMITTER_EMAIL=$email
EOF

# The binary-era units, retired. A box seated before the cutover still has the
# timer armed, and an armed reconciler would fight this unit on the next hour.
say "retiring the binary-era reconciler"
ssh -n "$host" 'systemctl --user disable --now yog-update.timer 2>/dev/null; \
    rm -f "$HOME/.config/systemd/user/yog-update.timer" \
          "$HOME/.config/systemd/user/yog-update.service" \
          "$HOME/.local/bin/yog-update"' || true

# Without lingering the user manager — and so the engine — dies at logout, which
# is the one failure that looks like "it just stopped overnight".
say "enabling lingering (the engine must outlive a login)"
ssh -n "$host" 'loginctl enable-linger "$USER"' \
    || echo "  ! could not enable lingering; run: sudo loginctl enable-linger \$USER" >&2

# Seating the deployment is an explicit operator act and must leave the box
# running the tag just loaded, so it clears a tripped start limit on the way in
# — `enable --now` will not, and a unit that hit its limit refuses to start
# until the interval expires. The restart is unconditional: a deploy is a human
# at a keyboard, which is exactly the condition the retired reconciler's
# deferral could not assume.
say "starting the engine on $tag"
ssh -n "$host" 'systemctl --user daemon-reload; \
    systemctl --user reset-failed yog.service 2>/dev/null; \
    systemctl --user enable yog.service; \
    systemctl --user restart yog.service'

say "state on $host"
ssh -n "$host" 'systemctl --user --no-pager --lines=0 status yog.service 2>&1 | sed -n "1,6p;/Active:/p"'
