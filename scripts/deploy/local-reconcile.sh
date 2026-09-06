#!/bin/sh
# **Unattended engine CD on a NATIVE box** (bl-8ea9) — the timer-driven half of
# `local.sh`'s deployment, and the sibling of `reconcile.sh`. That one
# reconciles a box that runs the image against ghcr's released tags; this one
# reconciles a box that runs the BINARY against crates.io's released versions.
# Same operator ruling behind both (2026-09-05): any new publication should
# result in an upgrade of the running versions.
#
#   local-reconcile.sh                # from yog-reconcile.service, hourly
#   local-reconcile.sh --self-test    # checks the decision; touches no machine
#
# TWO RECONCILIATIONS, NOT ONE PROCEDURE. They are separated because they have
# different safety conditions and must be free to happen at different times:
#
#   1. the INSTALLED binary against the registry's newest live version
#   2. the RUNNING engine against the installed binary
#
# (1) is always safe while the engine runs: `cargo install` replaces the file by
# rename, so the running process keeps its own inode and never sees a
# half-written binary. (2) is NOT always safe — a restart kills whatever turn is
# in flight — and that is the whole reason this file exists rather than a
# `cargo install && systemctl restart` line.
#
# **What a restart actually destroys is the WORK IN FLIGHT and the SPEND that
# bought it**, not the conversation. The engine survives a SIGTERM (§4.1 state
# is write-through, bl-b54e) and the pinned litany settles an unanswered tool
# window at the next drive boundary, so the branch revives on an ordinary
# deposit. What does not come back is the tools that died mid-execution, their
# partial side effects, and the model call that has to be paid again to
# re-derive from a window of error results. That is reason enough to read
# quiescence rather than assume it — and the reason has to be the TRUE one,
# because a mechanism defended by a false claim is one audit away from being
# deleted by whoever checks the claim (bl-286b).
#
# **The idle read is the §8.5 boundary's, not the cgroup's.** `reconcile.sh`
# states why the question belongs to the engine — `WsRow::running` is the
# boundary's one aggregate liveness bit and the union over the rows is exactly
# "no turn in flight anywhere in this world" — and the reply's own `stale` note
# defers too, because a stale derivation means the engine is saying its own
# `running` bits may be a photograph. A native unit's cgroup WOULD answer
# honestly here (the objection that retired the old process-count predicate is
# about a container unit's cgroup holding a `docker run` client), but two
# statements of one idle question drift, and the one that drifts is the one
# nobody re-reads. There is one idle question and the engine answers it.
#
# **The question is put to the RUNNING engine's own binary**, `/proc/<pid>/exe`,
# never to the one just installed: the gesture inbox is a serialization, and the
# process that reads it should be asked in its own words.
#
# Nothing here is stored. "Which version is installed" is the binary's own
# answer, "is a restart pending" is a kernel fact, and "is it safe" is the
# engine's answer. There is no state file to drift, and nothing to reconcile if
# somebody installs or restarts by hand.
#
# Exit 0 whenever the box is in a lawful state, including "newer version
# installed, restart deferred" — a deferral is a correct steady state, the timer
# is the retry cadence, and a unit that reddened on one would cry wolf through
# every long turn.
set -eu

CRATE=yog
UNIT=yog.service
# The install root, and it is not a preference: it is the Makefile's
# `INSTALL_PREFIX` default and the path `yog-local.service` execs. Installing to
# cargo's default root instead would leave the new build at a path the unit
# never reads, and the CD would be silently dead — a timer updating a binary
# nothing runs, with no error anywhere.
ROOT="$HOME/.local"
BIN="$ROOT/bin/$CRATE"
# The sparse index — the same source `cargo install` resolves from, so the two
# can never disagree about what exists. Three-letter names live under `3/<a>`.
INDEX_URL="https://index.crates.io/3/y/$CRATE"

say() { printf '%s: %s\n' "$CRATE-reconcile" "$*"; }
die() { say "$*" >&2; exit 1; }

# ------------------------------------------------------------- the decision
#
# ONE RULE: the engine should be running the newest live version, unless a turn
# is in flight or the operator stopped it. Every case below falls out of that
# sentence. It is a pure function of four facts so the branches that matter can
# be tested without a server, a release or an agent (`--self-test`). Prints
# exactly one of `current`, `defer`, `restart`.
#
# WHY `failed` IS ITS OWN ARM. A unit that could not start has no turn to
# protect, so nothing has to be deferred for — but neither is there any point
# retrying the version that just failed. Trying a version it has NOT run is the
# one useful act, and together with the yank lever below that closes the loop: a
# release that crashes on boot is recovered by yanking it, with nobody logging
# in. Without this arm a bad release strands the box until a human comes.
#
# `unknown` for any input defers: a fact we could not read is never grounds for
# killing a turn we cannot see.
decide() {
    _state=$1 _changed=$2 _pending=$3 _idle=$4
    if [ "$_state" = failed ]; then
        if [ "$_changed" = yes ]; then echo restart; else echo defer; fi
        return
    fi
    # Stopped on purpose, or mid-transition: not ours to move.
    if [ "$_state" != active ]; then echo current; return; fi
    # **The pending read is answered BEFORE the unknown guard, and the order is
    # load-bearing.** The idle question is only put to the engine when there is
    # something to defer FOR, so on a box that is already current `idle` is
    # `unknown` by construction — and an unknown-first guard would answer
    # `defer` on every tick of the steady state, reporting "a turn is in flight
    # or a fact is unreadable" about a box with nothing to do. Measured on a
    # live box: the second pass after an upgrade said exactly that.
    [ "$_pending" != no ] || { echo current; return; }
    case "$_pending$_idle" in
        *unknown*) echo defer; return ;;
    esac
    [ "$_idle" = yes ] || { echo defer; return; }
    echo restart
}

self_test() {
    _fail=0 _cases=0
    # state changed pending idle -> expected
    for _case in \
        'active   no  no      yes     current' \
        'active   no  no      no      current' \
        'active   no  no      unknown current' \
        'active   no  yes     yes     restart' \
        'active   yes yes     yes     restart' \
        'active   no  yes     no      defer' \
        'active   no  unknown yes     defer' \
        'active   no  yes     unknown defer' \
        'active   no  unknown unknown defer' \
        'failed   yes yes     unknown restart' \
        'failed   yes no      unknown restart' \
        'failed   no  yes     unknown defer' \
        'failed   no  no      unknown defer' \
        'inactive yes yes     unknown current' \
        'activating no yes    unknown current' \
        'unknown  no  yes     yes     current'
    do
        # shellcheck disable=SC2086
        set -- $_case
        _cases=$((_cases + 1))
        _got=$(decide "$1" "$2" "$3" "$4")
        if [ "$_got" = "$5" ]; then
            printf '  ok    %-10s %-3s %-7s %-7s -> %s\n' "$1" "$2" "$3" "$4" "$_got"
        else
            printf '  FAIL  %-10s %-3s %-7s %-7s -> %s (want %s)\n' \
                "$1" "$2" "$3" "$4" "$_got" "$5"
            _fail=1
        fi
    done
    # The table must not silently shrink to nothing — the same two-direction
    # discipline `make line-cap` and `leak-scan --self-test` hold.
    [ "$_cases" -gt 0 ] || die 'the decision table ran 0 cases: the harness is broken'
    [ "$_fail" = 0 ] || die 'the restart decision is wrong'
    say "self-test passed ($_cases cases)"
}

[ "${1:-}" != --self-test ] || { self_test; exit 0; }

# ---------------------------------------------------------------- 1. install
#
# The newest version the registry will serve. **Yanked releases are filtered
# HERE rather than left to cargo, and that is what makes a yank the rollback
# lever**: yank a bad release and the next tick resolves the previous one, sees
# it differ from what is installed, and puts it back — with nobody logging in.
# `cargo install` alone cannot do this, because it refuses to go backwards; that
# is why the install below passes an explicit `--version` and `--force`.
#
# **The fetch is a separate statement from the parse, deliberately.** As one
# pipeline the exit status would be the parser's, so a registry that answered
# 503 would parse to an empty string and look like an empty index — two
# different failures reported as one, and the wrong one.
latest_live() {
    _index=$(curl -fsS --max-time 60 "$INDEX_URL") \
        || die 'cannot reach the registry index'
    printf '%s\n' "$_index" | while IFS= read -r _line; do
        # One JSON object per line. Read it with parameter expansion rather than
        # a regex: `"vers"` occurs exactly once per line (a dependency entry
        # carries `"req"`, never `"vers"`), so there is nothing for a greedy
        # match to run past.
        case $_line in
            *'"yanked":false'*) ;;
            *) continue ;;
        esac
        _v=${_line#*'"vers":"'}
        _v=${_v%%'"'*}
        [ -n "$_v" ] && printf '%s\n' "$_v"
    done | sort -V | tail -1
}

# What is on disk. Asking the binary is the one authority: cargo's own
# bookkeeping can disagree with the file after a hand install, and the file is
# what the unit will exec. Absent is the empty string, which is a lawful answer:
# a box being seated for the first time has no binary yet.
installed() {
    [ -x "$BIN" ] || return 0
    "$BIN" --version 2>/dev/null | awk 'NR==1 {print $NF}'
}

want=$(latest_live)
[ -n "$want" ] || die 'the registry index named no live version'
have=$(installed)

changed=no
if [ "$want" != "$have" ]; then
    changed=yes
    say "installing $want (was ${have:-absent})"
    # `--locked` builds against the lockfile the crate publishes, which is the
    # parity check that yog and its embedded substrate resolve one brazen
    # (§16.7). `--version` is explicit and `--force` is set because the move may
    # be a DOWNGRADE — the yank lever above — and cargo refuses to move
    # backwards otherwise.
    cargo install "$CRATE" --root "$ROOT" --locked --version "$want" --force
    say "installed $(installed)"
else
    say "installed $have is current"
fi

# ---------------------------------------------------------------- 2. restart

state=$(systemctl --user show -P ActiveState "$UNIT" 2>/dev/null || echo unknown)
[ -n "$state" ] || state=unknown
pid=$(systemctl --user show -P MainPID "$UNIT" 2>/dev/null || echo 0)

# "A restart is pending" is not a flag anybody writes — it is the running
# process executing a different file than the one installed. The kernel holds
# that fact: `/proc/<pid>/exe` still resolves to the replaced inode after an
# install renames a new file over the path.
pending=unknown
running_inode=
[ "${pid:-0}" = 0 ] || running_inode=$(stat -Lc %i "/proc/$pid/exe" 2>/dev/null || true)
installed_inode=$(stat -Lc %i "$BIN" 2>/dev/null || true)
if [ -n "$running_inode" ] && [ -n "$installed_inode" ]; then
    if [ "$running_inode" = "$installed_inode" ]; then pending=no; else pending=yes; fi
fi

# The boundary read, and only when there is something to defer FOR: an engine
# that is already the installed binary is not going to be restarted whatever it
# answers, and asking anyway spends a deposit on every tick forever.
idle=unknown
if [ "$state" = active ] && [ "$pending" = yes ] && [ "${pid:-0}" != 0 ]; then
    # The running engine's own binary (see the header), and bounded: a wedged
    # engine must not hold the box when the next tick fires.
    reply=$(timeout 30 "/proc/$pid/exe" gesture '{"op":"workspaces"}' 2>/dev/null) || reply=
    case ${reply:-} in
        '')                 idle=unknown ;;
        *'"stale":'*)       idle=no ;;
        *'"running":true'*) idle=no ;;
        *'"ok":true'*)      idle=yes ;;
        *)                  idle=unknown ;;
    esac
fi

case "$(decide "$state" "$changed" "$pending" "$idle")" in
    current)
        if [ "$state" = active ]; then
            say 'the running engine is the installed binary; nothing to do'
        else
            say "$UNIT is $state; leaving it alone"
        fi ;;
    defer)
        if [ "$state" = failed ]; then
            say "$UNIT is failed and $want is all the registry offers;" \
                "leaving it (journalctl --user -u $UNIT)"
        else
            say "a turn is in flight or a fact is unreadable (pending: $pending," \
                "idle: $idle); deferring the restart to the next tick"
        fi ;;
    restart)
        say "starting $UNIT on $(installed) (was $state)"
        # `reset-failed` FIRST, always. A unit that tripped its start limit is
        # REFUSED a restart until the limit's interval expires — so without this
        # the recovery arm above cannot actually recover anything, which is
        # exactly how it failed the first time it was tried. On a healthy unit it
        # is a no-op beyond clearing the restart counter, which is the right
        # thing to do when putting a new version on anyway.
        systemctl --user reset-failed "$UNIT" 2>/dev/null || true
        systemctl --user restart "$UNIT" ;;
esac
