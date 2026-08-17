#!/bin/bash
# beats_s19.sh — S19 Adjudicator's real-substrate half (VISION V3, bl-77bc),
# sourced by stories.sh and fired from `run_headless` (beats_headless.sh, whose
# head records why this rung is IN there). No seat, no window, and NO MODEL
# CALL: the fan's spread, the delivery law, the staleness refusal and the
# retirement are real balls + real git, and every one of them can be wrong in
# ways a fake substrate cannot. What a beat here deliberately does NOT reach is
# cohort membership on the read surfaces (`/work-diff`'s candidate rows,
# `/science`'s) — membership is derived from real FIRE rows, a fire is a
# detached `lernie prompt`, and a prompt is a model call this verb refuses to
# spend. That join is proved by the in-crate tests over fixture trails
# (`workdiff`/`science` tests); what only the real substrate can falsify is
# below.
#
# Runs after `s11_workdiff`, which committed real work on `work/<ball>` — the
# target every candidate here forks from and delivers into.

# The last reply's one field, printed — the extraction sibling of `reply_is`,
# and like it a reader of the tail so it is always THIS gesture's answer.
s19_field() {
  python3 - "$out/gestures.jsonl" "$1" <<'PY'
import json,sys
line=[l for l in open(sys.argv[1]).read().split("\n") if l.strip()][-1]
print(eval(sys.argv[2], {"d": json.loads(line), "json": json}))
PY
}

s19_adjudicator() {
  data=$1 ; ws=$2 ; claim=$3 ; proj=$4
  proj_name=$(basename "$proj")

  # A prepared start to spread: `/prepare`'s own reply object, handed straight
  # back — the same composition the `--prepared` flag documents (bl-44d8).
  gesture "$data" /prepare --ws "$ws" --project "$proj_name" || true
  prep=$(s19_field 'json.dumps(d.get("prepared",{}))')

  # S19-D spread — one gesture, N rebound starts, each bound to its own
  # attempt worktree and all off one base (DESIGN §3.8: one fan, one base).
  gesture "$data" "{\"op\":\"fan\",\"project\":\"$proj_name\",\"ball\":\"$claim\",\"n\":2,\"prepared\":$prep}" || true
  reply_is '(d.get("ok") and len(d.get("rows",[]))==2
      and len({r.get("binding") for r in d["rows"]})==2
      and all(r.get("binding") for r in d["rows"]))' \
    && pass "S19-D spread: two rebound starts, each bound to its own worktree" \
    || fail "S19-D spread: two rebound starts, each bound to its own worktree" "no 2-row fanned reply"
  wt_a=$(s19_field '(d.get("rows") or [{}])[0].get("binding","")')
  wt_b=$(s19_field '(d.get("rows") or [{},{}])[1].get("binding","")')
  h_a=$(basename "${wt_a:-missing}") ; h_b=$(basename "${wt_b:-missing}")
  # The worktree leaf IS balls' handle (`fan::cohort::handle_of`), and each
  # candidate's branch resolves in the real repo at the target's own tip.
  tip=$(git -C "$proj" rev-parse "work/$claim" 2>/dev/null || echo no-tip)
  [ -d "$wt_a" ] && [ -d "$wt_b" ] \
      && [ "$(git -C "$proj" rev-parse "attempt/$h_a" 2>/dev/null)" = "$tip" ] \
      && [ "$(git -C "$proj" rev-parse "attempt/$h_b" 2>/dev/null)" = "$tip" ] \
    && pass "S19-D spread: real attempt branches share the target's tip as base" \
    || fail "S19-D spread: real attempt branches share the target's tip as base" "worktrees or refs wrong"

  # Everything below writes into the candidates' worktrees, so a failed
  # spread fails the rest by name here instead of dying on a redirect
  # (`set -e`) with no verdict rows at all — s11_workdiff's own guard shape.
  if [ ! -d "$wt_a" ] || [ ! -d "$wt_b" ]; then
    fail "S19-D deliver: the acceptance answers the delivery identities" "no candidate worktrees"
    fail "S19-D deliver: the target's history carries the tagged squash" "no candidate worktrees"
    fail "S19-D staleness: a stale sibling's delivery is refused" "no candidate worktrees"
    fail "S19-D retire: the worktree releases and the ref survives" "no candidate worktrees"
    fail "S19-D science: the projection answers over the real store" "no candidate worktrees"
    return 0
  fi

  # S19-D deliver — real work in candidate A, accepted by the one delivery
  # law: the reply is the four identities, and the target's own history now
  # carries the [handle]-tagged squash (the derived mark's authority).
  printf 'candidate A wrote this\n' > "$wt_a/candidate-a.txt"
  git -C "$wt_a" add -A
  git -C "$wt_a" commit -qm "candidate a work" --no-verify
  gesture "$data" "{\"op\":\"deliver\",\"project\":\"$proj_name\",\"ball\":\"$claim\",\"handle\":\"$h_a\",\"summary\":\"first candidate lands\"}" || true
  reply_is 'd.get("ok") and d.get("kind")=="delivered" and d.get("commit") and d.get("target")' \
    && pass "S19-D deliver: the acceptance answers the delivery identities" \
    || fail "S19-D deliver: the acceptance answers the delivery identities" "no delivered reply"
  git -C "$proj" log --format=%s "work/$claim" 2>/dev/null | grep -F "[$h_a]" >/dev/null \
    && pass "S19-D deliver: the target's history carries the tagged squash" \
    || fail "S19-D deliver: the target's history carries the tagged squash" "no [handle] subject on work/$claim"

  # S19-D staleness — the sibling is stale BY CONSTRUCTION now (§4.10 item 5),
  # and balls refuses it before anything merges: not ok, and the target tip
  # does not move. Both arms spelled out, like every refusal beat (bl-f16e).
  after_a=$(git -C "$proj" rev-parse "work/$claim" 2>/dev/null || echo no-tip)
  printf 'candidate B wrote this\n' > "$wt_b/candidate-b.txt"
  git -C "$wt_b" add -A
  git -C "$wt_b" commit -qm "candidate b work" --no-verify
  if gesture "$data" "{\"op\":\"deliver\",\"project\":\"$proj_name\",\"ball\":\"$claim\",\"handle\":\"$h_b\",\"summary\":\"stale sibling\"}"; then
    fail "S19-D staleness: a stale sibling's delivery is refused" "the delivery was accepted"
  else
    reply_is 'not d.get("ok")' \
        && [ "$(git -C "$proj" rev-parse "work/$claim" 2>/dev/null || echo moved)" = "$after_a" ] \
      && pass "S19-D staleness: a stale sibling's delivery is refused" \
      || fail "S19-D staleness: a stale sibling's delivery is refused" "refused wrongly, or the target moved"
  fi

  # S19-D retire — the loser's worktree releases; its ref survives (retention
  # keeps the record, absence-of-policy is never discard — DESIGN §3.8).
  gesture "$data" "{\"op\":\"retire\",\"project\":\"$proj_name\",\"ball\":\"$claim\",\"handle\":\"$h_b\"}" || true
  reply_is '(d.get("ok") and d.get("kind")=="retired" and d.get("discarded")==False)' \
      && [ ! -d "$wt_b" ] \
      && git -C "$proj" rev-parse "attempt/$h_b" >/dev/null 2>&1 \
    && pass "S19-D retire: the worktree releases and the ref survives" \
    || fail "S19-D retire: the worktree releases and the ref survives" "worktree or ref wrong"

  # S19-D science — the projection walks the real store: the claim attempt is
  # a row (its diff row is the work-diff's own), and its outcome derives from
  # the real refs just exercised. Cheap, and the one headless read of §3.9.
  gesture "$data" /science --ws "$ws" || true
  reply_is 'd.get("ok") and [r for r in d.get("rows",[])
      if r.get("diff",{}).get("ball_id")=="'"$claim"'"
      and r["diff"].get("source")=="work/'"$claim"'"
      and r.get("outcome",{}).get("state")]' \
    && pass "S19-D science: the projection answers over the real store" \
    || fail "S19-D science: the projection answers over the real store" "no claim row in /science"
}
