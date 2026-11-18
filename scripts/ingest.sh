#!/bin/sh
# **The pins follow the registry** (bl-38dc) — `make ingest`, run hourly by
# `.github/workflows/ingest.yml`. yog links litany and brazen in-process at
# exact pins, and every move of them used to be a hand-filed ball: publishing
# was automatic and every endpoint reconciled from the registry, so the pins
# were the one link a person carried, and an upstream release reached no
# device until somebody carried it.
#
#   scripts/ingest.sh      # run at the repo root; rewrites Cargo.toml + Cargo.lock
#
# It prints ONE line on stdout — the pins the tree now states — and that is the
# whole of what a caller reads; everything else goes to stderr.
#
# THE BRAZEN PIN IS READ OFF LITANY'S OWN INDEX LINE, never off brazen's newest.
# litany pins brazen exactly, yog pins it exactly, and the lockfile must resolve
# ONE brazen: a brazen newer than the one litany names cannot be linked at all.
# So there is one question asked of the registry — the newest live litany — and
# the brazen pin is that line's answer.
#
# NOTHING NEWER IS THIS SAME PATH: the pins are rewritten to what they already
# say, `cargo update --workspace` finds the lockfile already matching, and
# the diff is empty. The caller reads `git diff`; this file has no "current"
# branch to get wrong.
#
# NEVER BACKWARDS. The candidates are the registry's live versions AND the
# version pinned now, and the newest of them wins — so a pin whose release was
# yanked after it landed stays put rather than stepping down (a rollback of
# compiled-in API is a change somebody must read, unlike `local-reconcile.sh`'s
# rollback of an installed binary).
#
# WHAT IT REFUSES, loudly, because each is a hand's: a pin line that is no
# longer the shape written here, a litany whose brazen requirement is not an
# exact `=x.y.z`, and a lockfile that did not end up holding exactly one brazen
# at the pinned version.
set -eu

INDEX_URL=https://index.crates.io/li/ta/litany

die() { printf 'ingest: %s\n' "$*" >&2; exit 1; }

# The one spelling of each pin line: what `pinned` reads is what `pin` writes.
pinned() { # <crate>
    sed -n "s/^$1 = \({ version = \)\{0,1\}\"=\([0-9][0-9.]*\)\".*/\2/p" Cargo.toml
}
pin() { # <crate> <version>
    _tmp=$(mktemp)
    sed "s/^\($1 = \({ version = \)\{0,1\}\"=\)[0-9][0-9.]*\"/\1$2\"/" Cargo.toml >"$_tmp"
    cat "$_tmp" >Cargo.toml
    rm -f "$_tmp"
    [ "$(pinned "$1")" = "$2" ] || die "Cargo.toml's $1 pin is not \"=$2\" after the rewrite"
}
# Every version the lockfile holds of one crate, one per line.
locked() { # <crate>
    awk -v want="name = \"$1\"" '$0 == want { getline; gsub(/version = |"/, ""); print }' Cargo.lock
}

have=$(pinned litany)
[ -n "$have" ] || die 'Cargo.toml has no `litany = "=x.y.z"` line'
[ -n "$(pinned brazen)" ] || die 'Cargo.toml has no `brazen = { version = "=x.y.z"` line'

index=$(curl -fsS --max-time 60 "$INDEX_URL") || die 'cannot reach the registry index'

# One sparse-index line per published version. A line is a candidate when it is
# live, or when it is the version pinned now; the newest candidate is the pin.
line=$(printf '%s\n' "$index" | while IFS= read -r _line; do
    _v=${_line#*'"vers":"'}
    _v=${_v%%'"'*}
    case $_line in
        *'"yanked":false'*) ;;
        *) [ "$_v" = "$have" ] || continue ;;
    esac
    printf '%s %s\n' "$_v" "$_line"
done | sort -V -k1,1 | tail -1)
[ -n "$line" ] || die 'the registry index named no live litany'

litany=${line%% *}
brazen=$(printf '%s\n' "$line" \
    | sed -n 's/.*"name":"brazen","req":"=\([0-9][0-9.]*\)".*/\1/p')
[ -n "$brazen" ] || die "litany $litany does not pin brazen exactly; this move is a hand's"

pin litany "$litany"
pin brazen "$brazen"

# The lockfile follows the manifest, and the manifest is the pin authority: the
# two requirements are exact, so `--workspace` (re-resolve only what no longer
# matches Cargo.toml) has exactly one answer for each. NOT `cargo update -p
# <crate> --precise <v>`: measured on this tree, that re-resolves the crate's
# own dependencies even when <v> is the version already locked (it rewrote
# which `windows-sys` one transitive crate names), so the nothing-newer path
# would have landed a lockfile change every hour.
cargo update --workspace >&2

[ "$(locked litany)" = "$litany" ] || die "Cargo.lock does not hold litany $litany"
[ "$(locked brazen)" = "$brazen" ] \
    || die "Cargo.lock does not hold exactly one brazen at $brazen"

printf 'litany =%s, brazen =%s\n' "$litany" "$brazen"
