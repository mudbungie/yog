#!/bin/bash
# beats_s3res.sh — the S3/S4/S6 rows bl-84f3 left undriven, fired from `run_s5s8`
# in beats_s5.sh (world A: a primed project, one bound ball, one ready ball, and
# no conversation — so every verb here is a `bl` spawn and nothing reaches a
# model). Sourced by stories.sh; not an entry point of its own.
#
# Rows: S3-T2 (new-ball converges: create then exactly one claim), S4-T2
# (assign / release), S3-T4 (close-gate verbatim — the fixture is a project whose
# `pre-commit` hook FAILS), and S6-T5's `M ⚠` half, which the failed close hands
# over for free: the chip's error count needs a live failure and this is one.

# The S3-T4 fixture, in two halves — and the second half is the one this drive
# had to learn. First, a project whose `pre-commit` hook fails loudly and
# identifiably: `bl close` folds main in and runs the repo's own hook, so that
# hook IS the gate, and the marker is what the ops row's stderr must carry
# verbatim.
HOOK_MARKER="yogdrive-gate-refused"
fail_hook() {
  hooks="$1/.git/hooks" ; mkdir -p "$hooks"
  printf '#!/bin/sh\necho "%s" >&2\nexit 1\n' "$HOOK_MARKER" > "$hooks/pre-commit"
  chmod +x "$hooks/pre-commit"
}

# Second half: **work in the ball's worktree**. A ball whose work branch is
# identical to main has nothing to squash, so its close never reaches a commit and
# a failing hook never runs — the first attempt at this beat watched two closes
# exit 0 with the hook in place. So lay one commit per delivery worktree (what the
# agent would have written), `--no-verify` because the fixture's own commit is not
# the gesture under test.
lay_work() {
  # The delivery territory mirrors the project path, so its depth follows the
  # scratch dir's — find the leaves by their name (`bl-<id>`), never by depth.
  find "$1/yog/world/state/balls/plugins/bl-delivery" -type d -name 'bl-*' 2>/dev/null \
  | while read -r wt; do
    # `-name 'bl-*'` matches a NAME, and the delivery territory mirrors the
    # project path — so it also matches any *path component* of that mirror that
    # happens to be a ball id, which is exactly what the scratch world of a drive
    # run from a `work/bl-xxxx` worktree contains. `git -C` then walks UP from
    # that non-repo directory to the nearest enclosing repo and commits there:
    # a drive once committed the whole of an agent's staged work onto its own
    # branch as "yogdrive work", silently. The invariant that dissolves it is
    # exact rather than defensive — a real delivery worktree is its OWN git
    # toplevel, an interior mirror directory never is — so no `find` pattern, no
    # scratch-root placement and no ball id can bring the fixture back within
    # reach of a repo it does not own.
    top=$(git -C "$wt" rev-parse --show-toplevel 2>/dev/null) || continue
    [ "$top" = "$(cd "$wt" && pwd -P)" ] || continue
    printf 'work by yogdrive\n' > "$wt/yogdrive.txt"
    git -C "$wt" add -A >/dev/null 2>&1 || continue
    git -C "$wt" commit -qm "yogdrive work" --no-verify >/dev/null 2>&1 || true
  done
}

# An ops row matching an argv fragment that FAILED, carrying `text` in stderr —
# the S3-T4 assertion (a gate failure surfaced verbatim), the mirror of `row_ok`.
row_failed_with() {
  grep "$1" "$ops" 2>/dev/null | grep -v '"exit":0' | grep -q "$2"
}
# the ball is still in the live set (a refused close leaves it claimed). The id
# is REQUIRED and matched as a quoted JSON token: an empty `$2` made this
# `grep -q ""`, true of any non-empty listing, so the beat below passed
# unconditionally in exactly the runs where its own predecessor found no refused
# close to name (bl-f16e).
ball_listed() { [ -n "$2" ] && in_world "$1" bl list --json | grep -q "\"$2\""; }
claim_rows() { c=$(grep -c '"bl","claim"' "$ops" 2>/dev/null) || true; echo "${c:-0}"; }

# The residual ball rows, driven in world A's window (the Config tab already left).
# Both spellings are here, and the split is the STEERING RULE's: the verbs §11
# binds a key to ride that key on the FOCUSED target (`r` Release, `c` Close,
# `a` activity), and the two it binds none to — the new-ball rung and `assign` —
# ride the §8.5 boundary, which names what a coordinate could only aim at. The
# one click left is a VIEW (which ops row to expand), and it is the only thing
# in this file still measured against the 1150x760 default window.
s3_residuals() {
  wid=$1 ; out=$2 ; data=$3
  ws=$(find "$data/yog/workspaces" -maxdepth 1 -mindepth 1 -type d | head -1)
  # S3-T2 — the new-ball rung: `bl create` then the existing-ball path, so the
  # whole point is that it claims **once** (§8.1's new→existing convergence).
  # It is `Action::Prepare` with a `BallSpec::New` — the same variant the
  # `+ new ball` form's `Create & Start` constructs — so the line spells it
  # outright and the four coordinates the form cost (a fold, two text boxes and
  # a per-project button, all of them measurements of a side panel whose width
  # follows the scratch dir's name) are gone. Nothing is prompted: the reply is
  # a `Prepared`, the goal is never said, and there is no draft to cancel.
  before=$(claim_rows)
  gesture "$data" "/prepare ball --new converge ball --body no tools" \
      --ws "$(basename "$ws")" --project proj \
    && row_ok '"bl","create"' \
    && pass "S3-T2 new ball: bl create from the boundary" \
    || fail "S3-T2 new ball: bl create from the boundary" "no clean create row"
  sleep 2
  # The screenshot is now the visual half of a claim it did not make: the window
  # fired nothing, and it must still be painting the ball the deposit created.
  "$drive" shot "$wid" "$out/s3r-01-created.png"
  [ "$(claim_rows)" = "$((before + 1))" ] \
    && pass "S3-T2 new ball: converges to exactly one claim" \
    || fail "S3-T2 new ball: converges to exactly one claim" "claims=$(claim_rows)"

  # S4-T2 — Assign: `bl claim <id> --as <workspace>` on a READY ball without
  # starting a conversation at all (no mint, no prompt). §11 rule 2 called this
  # a pick — WHICH ready ball — and at the pointer it is one; at the line the
  # ball has an address, so `/assign <id>` names it and (235,145), which rode on
  # the balls section holding exactly one ready row above one ▶ Continue row,
  # is gone. The terminal holds no selection, so the §3.2 stamp is stated: the
  # workspace's own name, which is its directory's. That the WINDOW infers that
  # same stamp off its focus stays asserted where only the window can say it —
  # S3's ▶ Start and S4's mint-focus beat.
  mints=$(verb_count new)
  gesture "$data" "/assign $READY_BALL" \
      --project proj --as "$(basename "$ws")" \
    && row_ok "\"bl\",\"claim\",\"$READY_BALL\",\"--as\"" \
    && pass "S4-T2 assign: claim --as the workspace name" \
    || fail "S4-T2 assign: claim --as the workspace name" "no clean claim row"
  [ "$(verb_count new)" = "$mints" ] \
    && pass "S4-T2 assign: no conversation started" \
    || fail "S4-T2 assign: no conversation started" "a lernie new fired"
  sleep 2
  "$drive" shot "$wid" "$out/s3r-02-assigned.png"

  # S4-T2 — Release: `r` is §11's Release on the FOCUSED conversation's bound
  # ball — the composer's ball row without its coordinate — stamping the ball's
  # own claimant (§8.2's identity rider).
  release() { "$drive" bare "$wid" r; }
  until_landed release row_ok '"bl","unclaim"' \
    && pass "S4-T2 release: bl unclaim --as the claimant" \
    || fail "S4-T2 release: bl unclaim --as the claimant" "no clean unclaim row"
  sleep 2
  "$drive" shot "$wid" "$out/s3r-03-released.png"

  s3_close_gate "$wid" "$out" "$data"
}

# S3-T4 — the close gate, verbatim. With a failing `pre-commit` hook in the
# project, Close must surface the hook's own bytes and leave the ball claimed:
# the ops row is non-zero and carries the marker, and `bl list` still holds the
# ball. Then S6-T5's other half: the activity chip counts that live failure.
s3_close_gate() {
  wid=$1 ; out=$2 ; data=$3
  fail_hook "$data/proj"
  lay_work "$data"
  # `c` is §11's Close on the focused conversation's bound ball — the same
  # dispatcher the ball row's button calls, with no x fixed by an id's width.
  close() { "$drive" bare "$wid" c; }
  until_landed close row_failed_with '"bl","close"' "$HOOK_MARKER" \
    && pass "S3-T4 close gate: hook stderr verbatim in the ops row" \
    || fail "S3-T4 close gate: hook stderr verbatim in the ops row" "no failed close row"
  "$drive" shot "$wid" "$out/s3r-04-gate-refused.png"
  # A refused close leaves the ball CLAIMED, not delivered — asserted on the very
  # ball that failed, read back out of the failed row's own argv.
  # `$refused` is the SUBJECT, so its absence is a failure of this beat and not a
  # licence to assert nothing: when the beat above found no failed close row the
  # extraction yielded the empty string, and `ball_listed` was then asked whether
  # the listing contains "" — which it always does (bl-f16e, the same shape as
  # bl-afa7's `$minted`).
  refused=$(grep '"bl","close"' "$ops" | grep -v '"exit":0' | grep -o 'bl-[0-9a-f]*' | head -1)
  ball_listed "$data" "$refused" \
    && pass "S3-T4 close gate: the refused ball is still claimed" \
    || fail "S3-T4 close gate: the refused ball is still claimed" \
           "${refused:-no refused close row to name} not in the live set"
  # S6-T5's `M ⚠` half: the chip's error count is a projection over the tail at
  # read time (§6), and there is now exactly one live failure to project. The
  # count itself is the screenshot's half — nothing on disk stores it — so the
  # assertion here is the one this surface CAN make: expanding it spawns nothing.
  # `a` opens the accessory (§11); CLICK (a VIEW, and a pick): WHICH row to open,
  # DERIVED off the trail's own bottom edge from the frame just photographed
  # (bl-5cce) — the §11 tail idiom seats the newest op there whatever the chip's
  # heading, the Dismiss/Clear controls or the §7.2 staleness notes do above it,
  # and `locate.sh` refuses outright if `a` left the accessory collapsed.
  lines=$(wc -l < "$ops")
  "$drive" bare "$wid" a ; sleep 2
  "$drive" shot "$wid" "$out/s3r-05-chip-warned.png"
  read -r row_x row_y < <("$here/locate.sh" activity "$out/s3r-05-chip-warned.png")
  "$drive" click "$wid" "$row_x" "$row_y" ; sleep 2
  "$drive" shot "$wid" "$out/s3r-06-failed-row.png"
  [ "$(wc -l < "$ops")" = "$lines" ] \
    && pass "S6-T5 activity: the ⚠ tail is a pure read" \
    || fail "S6-T5 activity: the ⚠ tail is a pure read" "ops grew"
}
