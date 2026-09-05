#!/bin/bash
# predicates.sh — the harness's READ tier: every true/false question a beat
# asks of the world on disk. Sourced by harness.sh (beside wall.sh and
# gesture.sh, and for the same reason: it is a tier, not a runner), spent by
# every `beats_*.sh`. Split out of harness.sh at the repo's 300-line cap
# (bl-7547).
#
# The seam is real and it restores harness.sh's own header. That file is "the
# assertion helpers, the two waiting primitives and the
# verdict"; a predicate is none of the last three. What is left there is
# MECHANISM — how a beat waits and what a verdict is — and
# what is here is the vocabulary those wait on. `await` and `until_landed`
# take one of these; none of these knows either exists.
#
# THREE DISCIPLINES HOLD ACROSS EVERY PREDICATE BELOW, and each was learned by
# a beat that passed while proving nothing:
#   - MONOTONE, never an equality. `until_landed` re-fires, so `>=` and
#     set-membership are the only safe shapes; there is deliberately no
#     equality spelling left here to reach for (bl-0e44).
#   - AN EMPTY SUBJECT IS REFUSED, never interpolated. `grep -q ""` is true of
#     every non-empty stream, and it bites hardest when the beat ABOVE failed
#     to produce the id (bl-f16e; `make beat-audit` now checks the shape).
#   - READ AS DATA, never grepped, where the fact is structured. A bare
#     `grep '"seen"'` is true of any world where anything was ever focused
#     (bl-2d45).
#
# It reads two variables from its caller, exactly as harness.sh does: `$ops`,
# the world's ops.jsonl, and `$ws_root`, the focused workspace.

ops="" ; ws_root=""

# count occurrences of a litany verb in ops.jsonl (argv[1] == verb). Before the
# first spawn there is no ops.jsonl at all and `grep -c` prints nothing, so
# normalise the missing-file and no-match cases to the same plain 0 — every
# `await`/`until_landed` predicate below reads a count before yog has written a
# single row.
verb_count() { c=$(grep -c "\"litany\",\"$1\"" "$ops" 2>/dev/null) || true; echo "${c:-0}"; }
# the 1-based ops.jsonl line of a verb's first row (empty when absent) — the
# §8.1 order assertion compares two of these.
verb_line() { grep -n "\"$1\",\"$2\"" "$ops" 2>/dev/null | head -1 | cut -d: -f1; }
# does any ops row carry this exact argv fragment *and* a clean exit?
row_ok() { grep -q '"exit":0' <<<"$(grep "$1" "$ops" 2>/dev/null)"; }
# a *gpt* reply message exists under any agent of the focused workspace
reply_exists() { [ -n "$(find "$ws_root" -path '*messages*gpt*.json' 2>/dev/null)" ]; }
# THE SECOND TURN'S REPLY (bl-bf79): some conversation carries two or more user
# turns AND its newest message is an assistant record that arrived after the last
# of them. A transcript is numbered in arrival order (`001-user.md`,
# `002-<model>.json`, …), so that is exactly "the message that had to REVIVE a
# quiescent driver was answered" — the turn the operator's `stove` and
# `procedure` conversations both stopped dead at, where every yog-side signal
# said the verb succeeded. An assertion at the argv or the exit code passed with
# that bug shipped for as long as this runner has existed, which is why this
# reads the reply itself.
#
# INDEX-FREE on purpose. "assert `004-*.json`" is the same claim only while the
# FIRST turn also replied; a first turn that errors (its step writes an error
# `response.json` and no message file) shifts every number after it, and this
# beat is about the last turn either way. Read as data, never grepped, for the
# reason `seen_kind` is.
second_reply() { python3 - "$ws_root" <<'PY'
import pathlib,sys
for msgs in pathlib.Path(sys.argv[1]).glob("agents/*/messages"):
    names=sorted(p.name for p in msgs.iterdir())
    users=[n for n in names if n.endswith("-user.md")]
    if len(users)>=2 and names[-1].endswith(".json") and names[-1]>users[-1]:
        sys.exit(0)
sys.exit(1)
PY
}
# SOME step wrote a response.json and NONE of them is ZERO-BYTE — the shape a
# driver that died before its first token leaves behind (the wall-less `bz`
# refusal wrote exactly that, beside a truncated `staging.json`). The first
# clause is not decoration: a bare `! find … -size 0` is true of a world with no
# steps at all, so this passed in every run where the message it is about never
# reached a driver (bl-f16e).
no_dead_step() {
  [ -n "$(find "$ws_root/steps" -name response.json 2>/dev/null)" ] \
    && [ -z "$(find "$ws_root/steps" -name response.json -size 0 2>/dev/null)" ]
}
agent_count() { find "$ws_root/agents" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l; }
# does this file carry this text? (`beats_s5.sh` and `beats_s6.sh`, so it lives
# in the shared tier — bl-f16e.)
file_has() { grep -q -- "$2" "$1" 2>/dev/null; }
# Every §9 "cannot land" assertion is "these bytes did not move", so hash a file
# — and every caller compares TWO of these. A missing file printed the EMPTY
# STRING, so two absences compared EQUAL and the claim was made about a file
# that was never there (bl-f16e). `absent:<path>` can never equal a hash, and it
# names the miss in the beat's own failure detail.
md5of() { [ -f "$1" ] && md5sum "$1" | cut -d' ' -f1 || printf 'absent:%s' "$1"; }
verb_ge() { [ "$(verb_count "$1")" -ge "$2" ]; }
# A count predicate is `>=`, never `=`, and there is deliberately no equality
# spelling left here to reach for. Every gesture that counts conversations
# STARTS one, and `until_landed` re-fires — so an equality pinned to `before+1`
# is destroyed by the retry that was meant to satisfy it: the slow first attempt
# lands late, the retry starts a second, the count steps straight past `before+1`
# to `before+2`, and the predicate can never be true again. All five attempts
# burn and the beat reports "no new agent" about a gesture that worked every
# time — five phantom conversations deep, with the very next beat PASSING on the
# evidence they left (bl-0e44). What each of these beats means is "at least one
# more conversation exists", which is what this says; the exactness they used to
# claim is asserted by their own no-re-mint neighbours, which count a verb that
# must NOT grow — the safe direction for an equality.
agents_ge() { [ "$(agent_count)" -ge "$1" ]; }
# a `litany stop` row naming THIS conversation (argv: `litany stop <ws> <id>`).
# Identity, not a count: `verb_ge stop 1` is satisfied by a stop of the WRONG
# conversation exactly as well as by the right one, and the S6 beat that spends
# it is about which one the selection was on (bl-2d45).
# An EMPTY id is refused, never interpolated: `grep -q '""'` matches almost any
# ops row. Every id-taking predicate in this harness carries the same guard, so
# the trap cannot be re-armed one call site at a time (bl-f16e).
stopped() { [ -n "$1" ] && grep -q "\"$1\"" <<<"$(grep '"litany","stop"' "$ops" 2>/dev/null)"; }
# the id of the one conversation under `$ws_root` that is NOT `$1` — empty when
# there is none. A world that lays two roots can then name either by identity
# instead of by its rank in a list, which is a thing yog's list no longer has.
other_root() {
  find "$ws_root/agents" -maxdepth 1 -mindepth 1 -type d ! -name "$1" -printf '%f\n' 2>/dev/null | head -1
}
# `seen[ws][agent].<kind>` in ui.json — the §4.1/§6 acknowledgement watermark,
# read BY AGENT. It is how a gesture that is invisible by nature (focus is RAM,
# §13.1) proves it landed: acknowledging IS focusing, and the watermark is on
# disk. `seen_kind <ui.json> <agent> [kind]` — a named kind asks for that
# watermark, an omitted one for any, which is the honest question wherever the
# beat is about WHICH conversation was acknowledged rather than about what it
# had to acknowledge. Read as JSON, never grepped: a bare `grep '"seen"'` is
# true of any world where anything was ever focused, so it reported PASS in runs
# where the gesture under test never happened (bl-2d45).
seen_kind() { [ -f "$1" ] && [ -n "$2" ] || return 1 ; python3 - "$1" "$2" "${3:-}" <<'PY'
import json,sys
doc=json.load(open(sys.argv[1]))
agent,kind=sys.argv[2],sys.argv[3]
for ws in doc.get("seen",{}).values():
    marks=ws.get(agent,{})
    if marks if not kind else kind in marks:
        sys.exit(0)
sys.exit(1)
PY
}

