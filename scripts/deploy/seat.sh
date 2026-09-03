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
# **A RECONCILER IS SEATED AGAIN, and this comment block used to say it never
# would be** (bl-4e3c, operator instruction 2026-09-02). What stood here was:
# *"the reconcile question for an image is 'is a newer image loaded', which
# nothing on the box can answer without a registry to poll, and there is
# deliberately no registry it may poll. So an upgrade is this script, run by a
# human."* That was right about the binary-era reconciler this script still
# disables below — it reconciled a cargo-installed binary against the crates.io
# index and read quiescence off the unit's own cgroup, which for a container
# unit answers about `docker run`, a client process — and it was right about
# dev builds, which still travel only by the `save | load` stream below.
#
# It was wrong about RELEASES, and became more wrong once §10.1's registry
# landed: `ghcr.io/mudbungie/yog` publishes one immutable version tag per crate
# version from this repo's release workflow at tag time. That is a registry a
# box may poll, it is public, and it only moves forward. And the second
# objection — an unattended restart killing an in-flight turn — is answered by
# asking the engine over the §8.5 control boundary instead of asking the
# cgroup. `scripts/deploy/reconcile.sh` states both answers in full and is what
# this script now seats.
#
# **So the two paths are split by what they carry, not by who is watching.**
# THIS script is the bootstrap, the first seat, the dev-build path and the
# emergency path: it builds here, carries an unreleased tag over ssh, and its
# restart is unconditional because a human is at the keyboard. The reconciler
# is the released path, and it defers.
#
# Idempotent, and the upgrade path. Re-run it to move the box to the tip;
# re-running is also how a box seated on the old binary units adopts this one.
#
# **It ends by PROVING the engine answers** (`verify.sh`, bl-0719) and exits
# non-zero when it does not — a deploy that prints success over a crash loop is
# the defect, so the exit code and not a status print carries the truth.
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
#
# **The carrier renames the image, so the box reconciles the name** (bl-0719).
# podman's save archive spells the image with the registry it invents for a
# local build — `localhost/yog:<tag>` — `docker load` faithfully restores THAT
# name, and the unit is pointed at the bare `yog:<tag>` this checkout built.
# `docker run` cannot resolve it, the unit crash-loops under `Restart=always`,
# and the deploy used to print an `active` unit over the loop.
#
# Reconciled here rather than by writing `localhost/` into `deploy.env`,
# because the two spellings are not equally true: the tag is a fact about the
# crate — a version and a commit — while `localhost/` is a fact about which
# engine happened to carry it. **The unit's name must not encode the carrier's
# quirk**, or every box's environment file records the machine that last
# deployed to it and a docker-side deploy needs a different one. So the rename
# is undone where it happens: `docker load` names what it loaded, and that name
# is retagged to the name the unit knows. Unconditional and so idempotent —
# under a docker carrier the loaded name already IS `$tag`.
say "loading $tag on $host"
loaded=$("$engine" save "$tag" | ssh "$host" 'docker load' \
    | sed -n 's/^Loaded image: *//p' | tail -n 1)
[ -n "$loaded" ] || die "docker load on $host named no image; it did not carry $tag"
say "reconciling the loaded name ($loaded) to $tag"
ssh -n "$host" "docker tag '$loaded' '$tag'"

say "seating the unit on $host"
ssh -n "$host" 'mkdir -p "$HOME/.config/systemd/user" "$HOME/.config/yog" \
    "$HOME/.local/share/yog" "$HOME/.local/bin" "$HOME/work"'
scp -q "$here/yog.service" "$host:.config/systemd/user/yog.service"

# The reconciler and the proof it gates on, seated in ONE directory because
# `reconcile.sh` resolves `verify.sh` beside itself — the same file this script
# runs over ssh at the end, never a second copy of the five beats.
say "seating the reconciler on $host"
scp -q "$here/reconcile.sh" "$host:.local/bin/yog-reconcile"
scp -q "$here/verify.sh" "$host:.local/bin/verify.sh"
scp -q "$here/yog-reconcile.service" "$here/yog-reconcile.timer" \
    "$host:.config/systemd/user/"
ssh -n "$host" 'chmod +x "$HOME/.local/bin/yog-reconcile" "$HOME/.local/bin/verify.sh"'

# The one generated file on the box: which image the unit runs, and the identity
# it commits under. Everything else about the deployment is in the unit.
#
# **Rewriting it whole is how a human clears a refusal** (bl-4e3c). The
# reconciler records a tag that failed verification as `YOG_REFUSED` here and
# never re-attempts it; this write drops that line, which is right, because a
# human seating a box by hand IS the review the refusal was waiting for.
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
# These are the OLD `yog-update.*` names and this stays true of them: what
# bl-4e3c seated above is `yog-reconcile.*`, a different unit asking a different
# question, and the two never coexist under one name.
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
# at a keyboard, which is the one condition under which killing an in-flight
# turn is somebody's decision. `reconcile.sh` cannot assume it and so defers
# instead — same `reset-failed`, different answer to "may I restart now".
say "starting the engine on $tag"
ssh -n "$host" 'systemctl --user daemon-reload; \
    systemctl --user reset-failed yog.service 2>/dev/null; \
    systemctl --user enable yog.service; \
    systemctl --user restart yog.service'

# The timer, armed last: it must not fire against a half-seated box, and its
# first pass is ten minutes out in any case. `enable --now` starts the TIMER,
# not a pass.
say "arming the reconcile timer on $host"
ssh -n "$host" 'systemctl --user enable --now yog-reconcile.timer'

# **The last act is a VERIFICATION, and it fails the deploy** (bl-0719). What
# stood here was a status print, which is exactly what could not see a
# crash-looping unit: `is-active` says `active` because a `docker run` client
# process exists, not because the engine serves. `verify.sh` is the whole of
# that question — one ssh, one bounded sleep, five beats ending at a real TLS
# handshake with the §9.5 listener — and it says why in full. It is a separate
# file because re-asking "is it answering?" must not require re-seating a box.
say "verifying the engine on $host"
"$here/verify.sh" "$host" "$tag" "$version" \
    || die "the engine is not answering on $host (still seated on $tag)"
