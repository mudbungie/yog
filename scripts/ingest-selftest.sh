#!/bin/sh
# `make ingest-selftest` — the regression half of the unattended pin move
# (bl-38dc), in the gate. It drives the REAL `ingest.sh` under a fake `curl` and
# `cargo`, and the REAL `land-through-gate.sh` and `alarm.sh` under a fake `git`
# and `gh`, so it
# needs no network, no toolchain, no registry and no repository — the same
# discipline as `deploy/local-selftest.sh`, for the same reason: both scripts
# run unattended, are in no crate, and one of them writes to `main`.
#
# BOTH DIRECTIONS, because each failure is silent in its own way. A mover that
# never moves is green forever while the pins rot; a mover that moves when it
# should not lands a commit an hour. So every case that must move asserts the
# exact bytes it wrote, and every case that must not asserts the two files are
# byte-identical to what it was handed.
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
INGEST="$HERE/ingest.sh"
LAND="$HERE/land-through-gate.sh"
ALARM="$HERE/alarm.sh"
for s in "$INGEST" "$LAND" "$ALARM"; do
    [ -x "$s" ] || { echo "ingest-selftest: $s is not executable" >&2; exit 1; }
done

fails=0
cases=0
verdict() { # <label> <checker-fn>
    cases=$((cases + 1))
    if "$2"; then
        printf '  ok    %s\n' "$1"
    else
        printf '  FAIL  %s (exit %s)\n' "$1" "$code"
        printf '%s\n' "$out" | sed 's/^/          | /'
        sed 's/^/          log: /' "$work/log" 2>/dev/null || true
        fails=1
    fi
    rm -rf "$work"
}
shim() { # <name> — body on stdin
    { echo '#!/bin/sh'; cat; } > "$work/bin/$1"
    chmod 0755 "$work/bin/$1"
}
logged() { grep -qFx -- "$1" "$work/log" 2>/dev/null; }
said()   { case $out in *"$1"*) ;; *) return 1 ;; esac; }

# --- ingest.sh ---------------------------------------------------------------

# One sparse-index line of litany, in the registry's own key order.
line() { # <version> <yanked:true|false> <brazen-req>
    printf '{"name":"litany","vers":"%s","deps":[{"name":"serde","req":"^1"},' "$1"
    printf '{"name":"brazen","req":"%s","features":[],"kind":"normal"}],' "$3"
    printf '"features":{},"yanked":%s}\n' "$2"
}
# The last line is deliberately the OLDEST: the newest is chosen by version,
# never by position, and 0.0.9 must not outrank 0.0.15.
newer()        { line 0.0.15 false =0.0.19; line 0.0.16 false =0.0.21; line 0.0.9 false =0.0.8; }
current()      { line 0.0.14 false =0.0.19; line 0.0.15 false =0.0.19; }
tip_yanked()   { line 0.0.15 false =0.0.19; line 0.0.16 true  =0.0.21; }
pin_yanked()   { line 0.0.14 false =0.0.19; line 0.0.15 true  =0.0.19; }
loose_brazen() { line 0.0.15 false =0.0.19; line 0.0.16 false ^0.0.21; }
none()         { :; }

lock() { # <crate> <version>
    printf '[[package]]\nname = "%s"\nversion = "%s"\n\n' "$1" "$2"
}

ingest_case() { # <label> <index-fn> <litany-pin> <cargo:follows|splits> <checker-fn>
    work=$(mktemp -d "${TMPDIR:-/tmp}/yog-ingest.XXXXXX")
    mkdir -p "$work/bin" "$work/tree"
    "$2" > "$work/index.txt"
    {
        echo '[dependencies]'
        echo '# brazen = { version = "=0.0.1" }, a comment, is not the pin'
        echo 'brazen = { version = "=0.0.19", features = ["native-host"] }'
        echo "litany = \"=$3\""
        echo 'litany-like = "=9.9.9"'
    } > "$work/tree/Cargo.toml"
    { lock anyhow 1.0.0; lock brazen 0.0.19; lock litany "$3"; } > "$work/tree/Cargo.lock"
    cp "$work/tree/Cargo.toml" "$work/toml.was"
    cp "$work/tree/Cargo.lock" "$work/lock.was"

    shim curl <<EOF
for a in "\$@"; do :; done
[ "\$a" = https://index.crates.io/li/ta/litany ] \
    || { echo "fake curl: unexpected URL \$a" >&2; exit 22; }
[ -s "$work/index.txt" ] || exit 22
exec cat "$work/index.txt"
EOF
    # What `cargo update --workspace` does with two exact requirements: the
    # lockfile takes the versions the manifest states. `splits` is the resolver
    # that kept the old brazen beside the new one.
    shim cargo <<EOF
printf '%s\n' "\$*" >> "$work/log"
pin() { sed -n "s/^\$1 = .*\"=\([0-9.]*\)\".*/\1/p" Cargo.toml; }
{
    printf '[[package]]\nname = "anyhow"\nversion = "1.0.0"\n\n'
    printf '[[package]]\nname = "brazen"\nversion = "%s"\n\n' "\$(pin brazen)"
    [ "$4" = follows ] || printf '[[package]]\nname = "brazen"\nversion = "0.0.19"\n\n'
    printf '[[package]]\nname = "litany"\nversion = "%s"\n\n' "\$(pin litany)"
} > Cargo.lock
EOF

    set +e
    out=$(cd "$work/tree" && PATH="$work/bin:$PATH" "$INGEST" 2>"$work/err")
    code=$?
    set -e
    out="$out$(cat "$work/err")"
    verdict "$1" "$5"
}

untouched() {
    cmp -s "$work/tree/Cargo.toml" "$work/toml.was" \
        && cmp -s "$work/tree/Cargo.lock" "$work/lock.was"
}
toml_says() { # <litany> <brazen> — and every other line as it was
    sed -e "s/^litany = .*/litany = \"=$1\"/" \
        -e "s/^brazen = { version = \"=[^\"]*\"/brazen = { version = \"=$2\"/" \
        "$work/toml.was" | cmp -s - "$work/tree/Cargo.toml"
}
lock_says() { # <litany> <brazen>
    { lock anyhow 1.0.0; lock brazen "$2"; lock litany "$1"; } | cmp -s - "$work/tree/Cargo.lock"
}
moved_to() { # <litany> <brazen>
    [ "$code" = 0 ] && toml_says "$1" "$2" && lock_says "$1" "$2" \
        && logged 'update --workspace' && said "litany =$1, brazen =$2"
}
stayed()   { [ "$code" = 0 ] && untouched && said "litany =$1, brazen =0.0.19"; }
refused()  { [ "$code" != 0 ] && cmp -s "$work/tree/Cargo.toml" "$work/toml.was" \
                 && [ ! -e "$work/log" ] && said "$1"; }

both_step()       { moved_to 0.0.16 0.0.21; }
skips_yanked()    { moved_to 0.0.15 0.0.19; }
stays_15()        { stayed 0.0.15; }
never_backwards() { stayed 0.0.15; }
not_exact()       { refused 'does not pin brazen exactly'; }
no_index()        { refused 'cannot reach the registry index'; }
two_brazens()     { [ "$code" != 0 ] && said 'exactly one brazen'; }

echo 'ingest-selftest: the pin move'
ingest_case 'a newer litany -> both pins step, brazen read off litany'"'"'s line' \
    newer 0.0.15 follows both_step
ingest_case 'nothing newer -> the same path, and not one byte moves' \
    current 0.0.15 follows stays_15
ingest_case 'the newest is yanked -> it is skipped; the pin stays' \
    tip_yanked 0.0.15 follows stays_15
ingest_case 'the newest is yanked, a live one is still ahead -> steps to that' \
    tip_yanked 0.0.14 follows skips_yanked
ingest_case 'the pinned release itself was yanked -> never backwards' \
    pin_yanked 0.0.15 follows never_backwards
ingest_case 'litany does not pin brazen exactly -> refuses before writing' \
    loose_brazen 0.0.15 follows not_exact
ingest_case 'the index is unreachable -> refuses, cargo never runs' \
    none 0.0.15 follows no_index
ingest_case 'the lockfile kept a second brazen -> refuses' \
    newer 0.0.15 splits two_brazens

# --- land-through-gate.sh ----------------------------------------------------

B=ingest/20260101-0000

land_case() { # <label> <mode:green|red|moved|refused|absent> <pat> <checker-fn> [args...]
    _label=$1 _mode=$2 _pat=$3 _check=$4
    shift 4
    work=$(mktemp -d "${TMPDIR:-/tmp}/yog-land.XXXXXX")
    mkdir -p "$work/bin"
    shim date  <<EOF
echo 20260101-0000
EOF
    shim sleep <<EOF
:
EOF
    shim git <<EOF
printf 'git %s\n' "\$*" >> "$work/log"
case "$_mode \$*" in
  "moved push origin $B:main"|"refused push origin $B:main") exit 1 ;;
  "moved merge-base --is-ancestor FETCH_HEAD $B") exit 1 ;;
esac
EOF
    shim gh <<EOF
printf 'gh %s\n' "\$1 \$2 \$3" >> "$work/log"
case "$_mode \$1 \$2" in
  'absent run list') ;;
  *' run list')      echo 77 ;;
  'red run watch')   exit 1 ;;
esac
EOF
    set +e
    out=$(PATH="$work/bin:$PATH" PAT="$_pat" GH_TOKEN=notreal "$LAND" "$@" 2>&1)
    code=$?
    set -e
    verdict "$_label" "$_check"
}

before() { # <earlier line> <later line>
    _a=$(grep -nFx -- "$1" "$work/log" | head -1 | cut -d: -f1)
    _b=$(grep -nFx -- "$2" "$work/log" | head -1 | cut -d: -f1)
    [ -n "$_a" ] && [ -n "$_b" ] && [ "$_a" -lt "$_b" ]
}
gated() {
    before "git commit -qm the message -- Cargo.toml Cargo.lock" "git push -u origin $B" \
        && before "git push -u origin $B" "gh workflow run ci.yml" \
        && before "gh workflow run ci.yml" "gh run watch 77"
}
landed()    { logged "git push origin $B:main" && before 'gh run watch 77' "git push origin $B:main"; }
swept()     { [ "$(tail -1 "$work/log")" = "git push origin --delete $B" ]; }

green()        { [ "$code" = 0 ] && gated && landed && swept; }
no_token()     { [ "$code" != 0 ] && [ ! -e "$work/log" ] && said 'RELEASE_PLZ_TOKEN is not set'; }
red()          { [ "$code" != 0 ] && gated && ! logged "git push origin $B:main" \
                     && swept; }
main_moved()   { [ "$code" = 0 ] && gated && swept \
                     && said 'main moved while the gate ran'; }
refused_push() { [ "$code" != 0 ] && gated && swept && logged 'git fetch -q origin main' \
                     && said 'main did not move'; }
never_ran()    { [ "$code" != 0 ] && ! logged "git push origin $B:main" && swept \
                     && said 'ci.yml never started'; }
no_paths()     { [ "$code" = 2 ] && [ ! -e "$work/log" ]; }

echo 'ingest-selftest: the landing'
set -- ingest 'the message' Cargo.toml Cargo.lock
land_case 'green gate -> main fast-forwarded after the watch, branch deleted' \
    green notreal green "$@"
land_case 'no release token -> refuses, and git is never run' \
    green '' no_token "$@"
land_case 'red gate -> main is NOT pushed; branch deleted' \
    red notreal red "$@"
land_case 'main moved while the gate ran -> says so and exits GREEN; branch deleted' \
    moved notreal main_moved "$@"
land_case 'the push is refused and main did NOT move -> red, a hand'"'"'s; branch deleted' \
    refused notreal refused_push "$@"
land_case 'ci.yml never starts -> main is NOT pushed; branch deleted' \
    absent notreal never_ran "$@"
land_case 'no path to commit -> usage, and git is never run' \
    green notreal no_paths ingest 'the message'

# --- alarm.sh ----------------------------------------------------------------

T='unattended run red: Ingest'
RUN=https://github.example/o/r/actions/runs/7

# What `gh issue list --jq` answers: number, tab, title. The decoys are another
# workflow's alarm and a title this one is only the head of.
decoys()   { printf '3\tunattended run red: Advisories\n4\t%s (old)\n' "$T"; }
one_open() { decoys; printf '9\t%s\n' "$T"; }

alarm_case() { # <label> <red|green|''> <open-issues-fn> <checker-fn>
    work=$(mktemp -d "${TMPDIR:-/tmp}/yog-alarm.XXXXXX")
    mkdir -p "$work/bin"
    "$3" > "$work/issues.txt"
    shim gh <<EOF
[ "\$1 \$2" != 'issue list' ] || exec cat "$work/issues.txt"
printf 'gh %s\n' "\$*" | tr '\n' ' ' >> "$work/log"
echo >> "$work/log"
EOF
    set +e
    out=$(PATH="$work/bin:$PATH" GH_TOKEN=notreal GITHUB_WORKFLOW=Ingest GITHUB_RUN_ID=7 \
        GITHUB_SERVER_URL=https://github.example GITHUB_REPOSITORY=o/r \
        GITHUB_REPOSITORY_OWNER=o "$ALARM" $2 2>&1)
    code=$?
    set -e
    verdict "$1" "$4"
}
wrote()    { [ "$(grep -c . "$work/log" 2>/dev/null)" = "$1" ]; }
silent()   { [ "$code" = 0 ] && [ ! -e "$work/log" ]; }
opened()   { [ "$code" = 0 ] && wrote 1 \
                 && grep -q "^gh issue create --title $T --body @o .*$RUN" "$work/log"; }
closed()   { [ "$code" = 0 ] && wrote 1 \
                 && logged "gh issue close 9 --comment $RUN is green "; }
no_word()  { [ "$code" = 2 ] && [ ! -e "$work/log" ]; }

echo 'ingest-selftest: the alarm'
alarm_case 'red, none open -> opens exactly one issue, mentioning the owner and the run' \
    red decoys opened
alarm_case 'red, one open -> says nothing more' red one_open silent
alarm_case 'green, one open -> closes that one, and only that one' green one_open closed
alarm_case 'green, none open -> does nothing' green decoys silent
alarm_case 'neither word -> usage, and gh is never run' '' one_open no_word

[ "$cases" -gt 0 ] || { echo 'ingest-selftest: ran 0 cases: the harness is broken' >&2; exit 1; }
[ "$fails" = 0 ] || { echo 'ingest-selftest: FAILED' >&2; exit 1; }
echo "ingest-selftest: passed ($cases cases)"
