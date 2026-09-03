#!/bin/sh
# **Prove the deployed engine ANSWERS** (bl-0719) — `seat.sh`'s last act, and
# the only act in the deploy whose whole product is an exit code.
#
#   verify.sh <ssh-host|--local> <image-tag> <version>
#
# **`--local` runs the same beats ON the box** (bl-4e3c). `reconcile.sh`, the
# unattended reconciler, has to establish exactly this fact after it restarts
# the unit, and it runs where the unit runs — so it needs the beats without an
# ssh hop. It gets them by calling this file, never by carrying a copy: two
# statements of one verification drift, and the copy that drifts is the one
# nobody re-reads. Only the carrier below is conditional; the payload is one
# text and does not know which way it arrived.
#
# A status print stood at the end of the deploy, and a status print is exactly
# what could not see the defect this file was written for: `systemctl --user
# is-active` says `active` while the unit crash-loops, because a `docker run`
# client process existing is not the engine serving. Two consecutive real
# deploys reported success over a container that never started. A deploy that
# cannot tell success from a crash loop has reinvented the retired
# reconciler's blindness at the moment it acts — so the truth is carried here,
# in the exit code, and `seat.sh` fails with it.
#
# It is severable from the seating on purpose: re-running it re-asks the
# question without touching the box, which is what an operator wants when the
# answer was no.
#
# ONE ssh and one bounded sleep — no polling loop. Five beats, each proving
# what the one above it cannot, each naming what it found:
#
#   1. `sleep 8` — longer than the unit's `RestartSec=5s`, so a container that
#      cannot start has already been restarted at least once by here, and
#      every beat below reads a settled box rather than the instant `restart`
#      returned.
#   2. the unit is `active` — necessary, and on its own worth nothing (above).
#   3. the running container's image is EXACTLY the tag: bl-0719's defect head
#      on, and a crash loop with it — mid-loop there is no container at all.
#   4. `yog --version` inside it answers the version just built — the container
#      does not merely exist, it EXECUTES this engine.
#   5. the wire answers, and this needs NO seat and no credential: the engine's
#      own container carries `openssl` (the mint's recipe shells to it), the
#      listener is `--network host`, and a TLS server sends its certificate
#      before it ever asks the client for one. So an anonymous `s_client` gets
#      the operator chain out of the §9.5 listener and is then refused for want
#      of a client certificate — the refusal is mutual auth working, and the
#      chain is the proof the engine is bound and serving. Nothing weaker
#      separates "a process is up" from "the wire answers", which is the fact a
#      deploy exists to establish. The address is never passed in — it has one
#      home (`wire/address`, REMOTE §8) and it is read out of the container's
#      own data root, so the probe asks the engine where it bound.
set -eu

host=${1:-}
tag=${2:-}
version=${3:-}
[ -n "$host" ] && [ -n "$tag" ] && [ -n "$version" ] \
    || { echo "usage: ${0##*/} <ssh-host|--local> <image-tag> <version>" >&2; exit 2; }

# The carrier, and the whole of what `--local` changes. `sh -s --` either way,
# so the payload's argv is spelled once.
beats() {
    if [ "$host" = --local ]; then
        sh -s -- "$tag" "$version"
    else
        ssh "$host" "sh -s -- '$tag' '$version'"
    fi
}

beats <<'REMOTE'
set -eu
tag=$1
version=$2

fail() {
    printf 'deploy: %s\n' "$1" >&2
    journalctl --user -u yog.service --no-pager --lines=20 2>&1 | sed 's/^/  | /' >&2
    exit 1
}

sleep 8

state=$(systemctl --user is-active yog.service 2>/dev/null || true)
[ "$state" = active ] || fail "the unit is '$state', not active"

image=$(docker inspect -f '{{.Config.Image}}' yog 2>/dev/null || true)
[ -n "$image" ] \
    || fail 'active, but no yog container runs — active names a docker client'
[ "$image" = "$tag" ] || fail "the running container is $image, not $tag"

said=$(docker exec yog yog --version 2>&1 || true)
[ "$said" = "yog $version" ] || fail "the container answers '$said', not 'yog $version'"

address=$(docker exec yog cat /state/yog/wire/address 2>/dev/null || true)
[ -n "$address" ] || fail 'the engine bound no wire: no address in its data root'

probe=$(docker exec yog timeout 10 openssl s_client -connect "$address" 2>&1 || true)
printf '%s\n' "$probe" | grep -q 'Server certificate' \
    || fail "nothing answered TLS at $address"

printf 'engine answering: %s, wire at %s\n' "$image" "$address"
REMOTE
