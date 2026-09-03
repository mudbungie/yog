#!/bin/sh
# **Unattended engine CD** (bl-4e3c) — the timer-driven half of the deployment.
# `seat.sh` is a human at a keyboard; this runs on the box with nobody there.
#
#   reconcile.sh          # from yog-reconcile.service, every 15 minutes
#
# **This REVERSES the ruling seat.sh recorded**, on operator instruction
# (2026-09-02): "an upgrade is this script, run by a human". That ruling rested
# on two objections and both now have answers, which is why the reversal is a
# correction and not a relapse to the retired hourly reconciler.
#
#   1. *"There is deliberately no registry it may poll."* True of dev builds
#      and still true: a dev build travels by `save | load` through `seat.sh`,
#      which stays the bootstrap, the first seat and the emergency path. It was
#      never true of RELEASES — DESIGN §10.1 rules that `ghcr.io/mudbungie/yog`
#      publishes from this repo's release workflow at tag time, one immutable
#      version tag per crate version. So there is exactly one thing a box may
#      poll, it is public, and it only ever moves forward.
#   2. *"An unattended restart kills an in-flight turn."* The retired
#      reconciler read quiescence off the unit's own cgroup, which for a
#      container unit answers about `docker run`, a client process — the wrong
#      question wearing the right word. The right one is asked **over the §8.5
#      control boundary**, of the engine itself, and is answered below.
#
# **The idle read is `{"op":"workspaces"}`, and it is not a new gesture.**
# `WsRow::running` is "whether anything in it is Live/InFlight right now"
# (`src/boundary/reply/ws_row.rs`) — the boundary's one *aggregate* liveness
# bit, and the union over the rows is precisely "no turn in flight anywhere in
# this world". Every other liveness on the roster is addressed at a named
# workspace and agent (`Query::Agent`, `Query::Conversations`), so answering
# from those would mean an enumeration, N+1 deposits and a window between them
# in which a turn can start. Adding a machine-class read would have been in
# scope; it was not needed, and a verb that restates a field is the
# near-duplicate question bl-296f refused.
#
# The reply's own `stale` note is read too, and it defers. `stale` is present
# only when the derivation behind these rows is behind the world (§7.2) — so
# the engine is saying its own `running` bits may be out of date, and acting on
# them would be acting on a photograph.
#
# **A deferral is a correct steady state, so it is unbounded and silent-ish:**
# the timer is the retry cadence, there is no sleep-and-look-again here (the
# discipline `verify.sh` states as "one bounded sleep, no polling loop"), and a
# turn is NEVER killed to make room for an upgrade. What IS bounded is the
# other direction — **a tag that failed verification here is attempted exactly
# once, ever.** The bound is an invariant rather than a counter: a rollback
# writes `YOG_REFUSED` into `deploy.env` beside the `YOG_IMAGE` it restored,
# and this refuses to re-attempt that tag. Without it a bad release restarts
# the engine every fifteen minutes forever, which is the failure mode an
# unattended upgrade has and a human one does not.
#
# **`verify.sh` gates, and it is the same file `seat.sh` calls** — `--local`,
# because this runs on the box. Five beats ending at a real TLS handshake with
# the §9.5 listener; a failure rolls the unit back to the tag it was serving.
#
# The whole product is the exit code and the journal. `systemctl --user
# list-units --failed` is where a refusal is visible, so a failure exits
# non-zero rather than printing a sad line and succeeding.
set -eu

self=${0##*/}
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

# The registry has one home (DESIGN §10.1) and it is not a parameter: a box
# that could be pointed at another registry is a box whose upgrades are not
# this project's. Nothing else here is box-specific either — every fact about
# THIS box lives in `deploy.env`, which is the leak rule and the severability
# one at once.
REGISTRY=ghcr.io
REPOSITORY=mudbungie/yog
PACKAGE=$REGISTRY/$REPOSITORY
UNIT=yog.service
env_file=$HOME/.config/yog/deploy.env

say() { printf '%s: %s\n' "$self" "$*"; }
die() { printf '%s: %s\n' "$self" "$*" >&2; exit 1; }

# The version a tag states about itself. §10.1 makes the image and the crate one
# publication carrying one version, so the tag is the authority and the box is
# never asked twice. Both spellings answer: the registry's `<package>:<version>`
# and `seat.sh`'s local `yog:<version>-<short-commit>`.
version_of() { printf '%s\n' "${1##*:}" | sed 's/-[0-9a-f]\{7,\}$//'; }

# ---------------------------------------------------------------------------
# 1. What this box is running, from the one file that says so.

[ -f "$env_file" ] || die "no $env_file — this box is not seated (scripts/deploy/seat.sh)"
current=$(sed -n 's/^YOG_IMAGE=//p' "$env_file" | tail -n 1)
[ -n "$current" ] || die "no YOG_IMAGE in $env_file"
refused=$(sed -n 's/^YOG_REFUSED=//p' "$env_file" | tail -n 1)
running_version=$(version_of "$current")

# ---------------------------------------------------------------------------
# 2. What the registry has released.

# **AN EMPTY PACKAGE IS THE STANDING STATE, not an error** (bl-6b96). DESIGN
# §10.1 names the registry, the tag convention and the publishing authority —
# "pushed only from that repo's own release workflow, at tag time" — but no job
# in `.github/workflows/` performs that push yet. So until bl-6b96 lands, every
# pass here finds nothing and must say so and exit 0: a timer that goes red
# every fifteen minutes on a condition nobody has promised to fix is a timer an
# operator disables, and it would be red on the ONE box state this reconciler
# was written to sit in quietly.
#
# That makes "nothing published" and "something is wrong" two answers rather
# than one, so the HTTP status is read rather than thrown away. `curl -f`
# collapses every failure into exit 22, which cannot tell an unpublished
# package from a private one — and those two want opposite responses.
ask() {
    # Prints the body, then the status code on its own last line. `000` is a
    # request that never got an answer at all.
    curl -sS -w '\n%{http_code}' "$@" 2>/dev/null || printf '\n000\n'
}
status_of() { printf '%s' "$1" | tail -n 1; }
body_of() { printf '%s' "$1" | sed '$d'; }

# The package is PUBLIC (DESIGN §10.1), so this box holds no registry
# credential — which is the property that makes an unattended reconciler
# possible at all rather than a place to park a long-lived token.
answer=$(ask "https://$REGISTRY/token?service=$REGISTRY&scope=repository:$REPOSITORY:pull")
token=$(body_of "$answer" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
[ -n "$token" ] || die "the registry issued no anonymous pull token for $PACKAGE
  (it answered $(status_of "$answer")). DESIGN 10.1 makes that package public,
  so this box needs no credential and holds none. Remedy: restore the package's
  public visibility, or seat this box by hand from a checkout
  (make deploy HOST=<this box>), which needs no registry at all."

answer=$(ask -H "Authorization: Bearer $token" "https://$REGISTRY/v2/$REPOSITORY/tags/list")
status=$(status_of "$answer")
case $status in
    200) ;;
    404)
        # The package does not exist yet — the standing state until bl-6b96
        # builds the push. Nothing to reconcile against, and nothing wrong.
        say "$PACKAGE publishes nothing yet — nothing to reconcile against"
        exit 0 ;;
    401|403)
        die "$PACKAGE refused an anonymous read ($status). DESIGN 10.1 makes it
  public and this box holds no credential by design. Remedy: restore the
  package's public visibility, or seat this box by hand from a checkout
  (make deploy HOST=<this box>)." ;;
    *) die "$PACKAGE answered $status listing its tags" ;;
esac

# RELEASED tags only, spelled strictly. That excludes `latest` — never
# published, §10.1 — and `seat.sh`'s local `<version>-<commit>` dev tag, which
# an unattended box must never adopt: a dev build is carried by a human who is
# watching, and this is the code that is not.
newest=$(body_of "$answer" | tr ',' '\n' \
    | sed -n 's/.*"\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)".*/\1/p' \
    | sort -V | tail -n 1)
# A package that exists but carries no released version — the same standing
# state one HTTP status over, and the same clean no-op.
[ -n "$newest" ] || { say "$PACKAGE lists no released version tag yet"; exit 0; }

# ---------------------------------------------------------------------------
# 3. Is there anything to do at all?

if [ "$newest" = "$running_version" ]; then
    say "on $running_version, the newest released version — nothing to do"
    exit 0
fi
# A box ahead of the registry is `seat.sh`'s doing and is not an error: a human
# carried an unreleased build here on purpose, and an unattended DOWNgrade onto
# a released tag would undo their deploy behind their back.
if [ "$(printf '%s\n%s\n' "$newest" "$running_version" | sort -V | head -n 1)" = "$newest" ]; then
    say "on $running_version, ahead of the newest released $newest — leaving it alone"
    exit 0
fi

target=$PACKAGE:$newest

# The bound (see the header): one unattended attempt per tag, ever.
if [ "$refused" = "$target" ]; then
    say "$target failed verification on this box and was rolled back; it is not
  retried unattended. Remedy: fix it and release again — a version is published
  once (DESIGN 10.1) — or, to re-attempt this exact tag, delete the YOG_REFUSED
  line from $env_file."
    exit 0
fi

# ---------------------------------------------------------------------------
# 4. Ask the engine, over the boundary, whether a turn is in flight.

# `docker exec` and not a wire dial: the gesture inbox is the §8.5 boundary's
# other serialization, it needs no certificate, and the engine is right there.
# An engine that does not answer is deferred to rather than upgraded past — a
# silent failure to read is not a reading of "idle".
reply=$(docker exec yog yog gesture '{"op":"workspaces"}' 2>/dev/null) || reply=
[ -n "$reply" ] || { say "the engine did not answer the boundary — deferring $target"; exit 0; }

case $reply in
    *'"stale":'*)
        say "the engine says its derivation is stale — deferring $target"
        exit 0 ;;
esac
case $reply in
    *'"running":true'*)
        say "a turn is in flight — deferring $target to the next timer fire"
        exit 0 ;;
esac

# ---------------------------------------------------------------------------
# 5. Act: pull, point, restart, prove.

# Rewrite `deploy.env`'s two generated keys, keeping every other line — the git
# identity `seat.sh` wrote lives in this file and is not ours to lose. Both keys
# are dropped and re-stated so a rollback can clear a refusal by not passing one.
point_at() {
    tmp=$env_file.tmp.$$
    {
        sed '/^YOG_IMAGE=/d;/^YOG_REFUSED=/d' "$env_file"
        printf 'YOG_IMAGE=%s\n' "$1"
        [ -n "${2:-}" ] && printf 'YOG_REFUSED=%s\n' "$2"
        true
    } > "$tmp"
    mv "$tmp" "$env_file"
}

# `reset-failed` before `restart`, for the reason `seat.sh` states: the unit's
# start limit is twenty starts in a hundred seconds (a deliberately wide boot
# race window), and a unit that tripped it refuses to start until the interval
# expires — so a restart without this is a no-op that reads as a success.
restart() {
    systemctl --user daemon-reload
    systemctl --user reset-failed "$UNIT" 2>/dev/null || true
    systemctl --user restart "$UNIT"
}

say "pulling $target"
docker pull "$target" >/dev/null 2>&1 || die "could not pull $target"

prior=$current
say "pointing the unit at $target"
point_at "$target"
restart

if "$here/verify.sh" --local "$target" "$newest"; then
    say "the engine is answering on $target"
    exit 0
fi

# ---------------------------------------------------------------------------
# 6. Rollback. verify.sh said no, so the box is not serving the new tag and the
# only destination known to have served is the one it was on.

say "verification failed on $target — rolling back to $prior"
point_at "$prior" "$target"
restart
"$here/verify.sh" --local "$prior" "$(version_of "$prior")" \
    || die "rolled back to $prior and IT does not verify either — this box needs a human"
die "$target failed verification; rolled back to $prior and recorded it refused"
