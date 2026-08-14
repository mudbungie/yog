#!/bin/bash
# harness.sh — the tier every story run shares, sourced by stories.sh: the
# assertion helpers, the two waiting primitives, the per-run seat, and the
# verdict in BOTH its halves — the human PASS/FAIL line, and the machine-keyable
# row written beside it.
#
# It is a SEAM, not a shard. stories.sh carries the S0/S1 beats and the STEERING
# doctrine that aims them; nothing below is about S0/S1 — every `run_*` verb in
# every sourced `beats_*.sh` calls `pass`/`fail`/`await`/`until_landed`/
# `claim_seat`/`verdict`, so their one home is here rather than in whichever
# runner happened to be written first. The cut was forced and is the right one:
# stories.sh sat at the repo's 300-line cap, so the verdict could not grow its
# second half where it lived (bl-56d5).
#
# Sourced, never executed. It reads two variables from its caller — `$out`, the
# run's evidence directory, and `$ops`, the world's ops.jsonl — and it defines
# no CLI verb of its own.

# --- assertion helpers ------------------------------------------------------
ops="" ; ws_root="" ; fails=0

# count occurrences of a lernie verb in ops.jsonl (argv[1] == verb). Before the
# first spawn there is no ops.jsonl at all and `grep -c` prints nothing, so
# normalise the missing-file and no-match cases to the same plain 0 — every
# `await`/`until_landed` predicate below reads a count before yog has written a
# single row.
verb_count() { c=$(grep -c "\"lernie\",\"$1\"" "$ops" 2>/dev/null) || true; echo "${c:-0}"; }
# the 1-based ops.jsonl line of a verb's first row (empty when absent) — the
# §8.1 order assertion compares two of these.
verb_line() { grep -n "\"$1\",\"$2\"" "$ops" 2>/dev/null | head -1 | cut -d: -f1; }
# does any ops row carry this exact argv fragment *and* a clean exit?
row_ok() { grep "$1" "$ops" 2>/dev/null | grep -q '"exit":0'; }
# a *gpt* reply message exists under any agent of the focused workspace
reply_exists() { find "$ws_root" -path '*messages*gpt*.json' 2>/dev/null | grep -q . ; }
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
# no step wrote a ZERO-BYTE response.json — the shape a driver that died before
# its first token leaves behind (the wall-less `bz` refusal wrote exactly that,
# beside a truncated `staging.json` and its reason in `stderr.log`).
no_dead_step() { ! find "$ws_root/steps" -name response.json -size 0 2>/dev/null | grep -q . ; }
agent_count() { find "$ws_root/agents" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | wc -l; }
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
# a `lernie stop` row naming THIS conversation (argv: `lernie stop <ws> <id>`).
# Identity, not a count: `verb_ge stop 1` is satisfied by a stop of the WRONG
# conversation exactly as well as by the right one, and the S6 beat that spends
# it is about which one the selection was on (bl-2d45).
stopped() { grep '"lernie","stop"' "$ops" 2>/dev/null | grep -q "\"$1\""; }
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
seen_kind() { [ -f "$1" ] || return 1 ; python3 - "$1" "$2" "${3:-}" <<'PY'
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

# --- the workspace wall: the harness's per-workspace brazen fixture ----------
# `seed_wall <data> <workspace-name>`.
# Since the blast-radius ruling (DESIGN §16.2) brazen's config,
# credentials and model cache are the WORKSPACE's — `<world>/walls/<name>/brazen/`
# — and nothing yog spawns reads the host's own brazen state any more. A newborn
# wall is an EMPTY DIRECTORY, so a scratch workspace has no provider rows and no
# sign-ins until something puts them there. That used to be `yogdrive.sh launch`
# symlinking `$XDG_DATA_HOME/brazen/credentials`, a path no driven process has
# read since the ruling — the run launched green and died at its first model
# call (bl-49c6).
#
# A WALL IS KEYED BY A NAME, NOT BY A WORKSPACE THAT ALREADY EXISTS (bl-1851),
# and the name of the one every wire beat runs in is a CONSTANT. DESIGN §3.1:
# "The bootstrap names without asking. The empty-world start (§3.4) creates its
# workspace under the fixed default name `home` — a constant, not a config
# (severability: there is nothing to delete), and not a mint (the wordlist names
# conversations, §3.3)." Every run verb here opens on zero workspaces, so every
# wall this harness lays is `home`'s, and it is layable before yog is launched.
#
# SO IT IS LAID BEFORE THE LAUNCH, with the world seed (stories.sh `seed`), and
# that ordering is the whole of bl-1851. yog's bare start is ONE gesture — the
# `lernie new` and the detached `lernie prompt` fire within the same second — so
# a fixture laid "after the mint" is always later than the FIRST MODEL CALL, and
# every scratch-world run's first turn died with brazen's config error,
# `unknown provider openai-chatgpt`, while the beat asserting the reply read as
# a wire outage. Two independent runs, two days latent, one phantom each.
#
# Nothing about the product moved, and §9.2's gate stays retired. §16.2's "a
# wall is born empty" is the rule that YOG never inherits a machine's brazen
# state into a newborn sphere, and yog still never does; what the birth gate
# lacked was never a knowable name but anything of its own to put in the wall.
# This is the operator's own hand, and an operator's `home` wall is exactly
# where their `home` sign-in lives.
#
# The host paths are anchored on `$HOME`, like stories.sh's `real_world`, because
# they are FIXTURE SOURCES: the clean room overrides `XDG_CONFIG_HOME` /
# `XDG_DATA_HOME` onto its own scratch, and a source that folded off those would
# vanish inside the room it is meant to supply.
#
# DESIGN §3.1's bootstrap constant, `src/names/mod.rs::DEFAULT_NAME` in the tree.
BOOTSTRAP_WS=home
wall_dir() { printf '%s/yog/world/walls/%s/brazen' "$1" "$2"; }
seed_wall() {
  [ -n "${2:-}" ] || return 0
  wall=$(wall_dir "$1" "$2")
  mkdir -p "$wall"
  if [ -f "$HOME/.config/brazen/config.toml" ]; then
    cp "$HOME/.config/brazen/config.toml" "$wall/config.toml"
  fi
  if [ -d "$HOME/.local/share/brazen/credentials" ]; then
    mkdir -p "$wall/credentials"
    cp "$HOME"/.local/share/brazen/credentials/*.json "$wall/credentials/" 2>/dev/null || true
  fi
  echo "wall: $wall"
}

# --- the verdict, machine-keyable (bl-56d5) ---------------------------------
# Every PASS/FAIL line also lands as ONE JSONL row in `$out/verdicts.jsonl`,
# beside the `gestures.jsonl` the boundary transport writes. The printf lines
# stay exactly as they were: a human reads the run scroll, a tool reads the
# rows, and neither is a summary of the other because both are emitted from the
# same call. Before this, a run's whole verdict was ~50 printf lines plus one
# exit code — nothing a report generator, a re-baseline diff or the QUALITY §3
# scorecard could key on without re-parsing prose.
#
# The LABEL stays the single source of the beat's name; `beat` is only that
# label slugged, so there is no second name to drift out of step with the line
# the operator reads. A consumer that needs exactness reads `label`, which is
# verbatim; `beat` is the convenience key (two labels differing only in
# punctuation would slug alike — none do, and `label` is the tiebreak).
#
# `evidence` is the newest screenshot in the run directory at the instant the
# verdict was taken — the frame the beat had driven to when it was judged. That
# is a measurement, not a claim: a beat that shot nothing inherits the previous
# frame, which is why the field names a file rather than asserting it proves
# anything. The shots stay what STORIES.md says they are, visual confirmation.
beat_id() {
  printf '%s' "$1" | tr 'A-Z' 'a-z' \
    | sed -e 's/[^a-z0-9][^a-z0-9]*/-/g' -e 's/^-//' -e 's/-$//'
}
json_str() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }
newest_shot() { ls -t "$out"/*.png 2>/dev/null | head -1; }

# `bin` is the BINARY THIS RUN DROVE, resolved inside the run and carried on
# every row. yogdrive.sh launches plain `yog` from PATH and `drive.sh ladder`
# prefixes this checkout's `target/release` onto it, so the run's own PATH is
# the only place the answer exists — a reader that re-asks `command -v yog`
# afterwards resolves the operator's INSTALLED yog, a different sha, into the
# one field a drive log exists to pin (bl-d1af; QUALITY.md §4, "a verdict is a
# claim about the sha it names"). Resolved once and memoized: PATH does not
# move mid-run, and an unresolvable `yog` is left EMPTY rather than filled with
# a plausible path, so logskel.sh can say so out loud.
drive_bin=""
driven_binary() {
  [ -n "$drive_bin" ] || drive_bin=$(command -v yog 2>/dev/null) || drive_bin=""
  printf '%s' "$drive_bin"
}
record() {
  [ -n "${out:-}" ] && [ -d "$out" ] || return 0
  printf '{"run":"%s","beat":"%s","label":"%s","verdict":"%s","detail":"%s","evidence":"%s","bin":"%s","at":"%s"}\n' \
    "$(json_str "${drive_run:-unknown}")" "$(beat_id "$1")" "$(json_str "$1")" \
    "$2" "$(json_str "${3:-}")" "$(json_str "$(newest_shot)")" \
    "$(json_str "$(driven_binary)")" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$out/verdicts.jsonl"
}

say() { printf '%-46s %s\n' "$1" "$2"; }
pass() { say "$1" "PASS"; record "$1" PASS ""; }
fail() { say "$1" "FAIL — $2"; fails=$((fails + 1)); record "$1" FAIL "$2"; }

# EVERY assertion that waits on the substrate waits HERE, not on a sleep: poll a
# predicate for up to ~40 s. A fixed sleep is the wrong wait for work whose
# latency belongs to the model or to a spawned tool, and it fails in both
# directions — 2026-07-26 alone, an 8 s guess went red against an agent that
# spent four tool round trips before answering, and a 5 s one went red against a
# `lernie stop` whose row landed the moment it finished reaping a *streaming*
# driver. Sleeps that remain gate only screenshots, never a verdict.
await() { for _ in $(seq 1 40); do "$@" && return 0; sleep 1; done; return 1; }

# The same rule for the *input* side: a gesture is only known to have landed when
# the substrate says so. `until_landed <gesture-fn> <predicate...>` fires the
# gesture, waits ~8 s for the predicate, and re-fires up to five times. A blind
# click at a fixed moment is the fixed-sleep gamble moved from time into space —
# yog paints the balls section only after its first project scan and a relaunched
# window only after its first frame, so under load (this runner was hardened at a
# load average of 80) the target is not there yet and the click lands on blank
# panel, silently. Every gesture passed here is a no-op when it misses, and the
# predicate is re-read before each retry, so a landed gesture never re-fires.
#
# TWO REQUIREMENTS ON THE ARGUMENTS, and the second one cost a whole re-baseline
# to learn (bl-0e44). (1) The GESTURE must be a no-op when it misses — the line
# above. (2) The PREDICATE must be MONOTONE: once true it stays true, so `>=` and
# set-membership, never `=`. A retry is not free of consequence for a gesture
# that is not idempotent — `phantom` starts a whole conversation each time — so
# an equality on a quantity the gesture ADDS to is destroyed by its own retry
# loop, and the beat then fails five times about a gesture that succeeded five
# times. `verb_ge`, `agents_ge`, `row_ok`, `stopped` and `seen_kind` are all
# monotone by construction; there is no equality predicate in this file, and a
# beat that wants exactness asserts it OUTSIDE this loop, where nothing re-fires.
until_landed() {
  g=$1 ; shift
  for _ in 1 2 3 4 5; do
    "$g"
    for _ in 1 2 3 4 5 6 7 8; do "$@" && return 0; sleep 1; done
  done
  return 1
}

# --- one name, one definition ----------------------------------------------
# Every `beats_*.sh` is sourced into ONE flat namespace, and bash lets a later
# `f() { … }` silently replace an earlier one. A second `s6_attention` added to
# `beats_s6.sh` therefore DELETED the S6-T1 stage `run_s7` calls: three beats
# stopped running, and nothing in the verdict could say so, because a beat that
# never ran leaves no row and the run reported ALL BEATS PASS for what was left
# (bl-0e44 — found by reading a ladder's rows against the ladder's own source,
# which is not a thing anyone does twice).
#
# So the check is STRUCTURAL and fires before any verb runs — stories.sh calls
# it at the one instant the namespace is complete, right after the last source.
# Column-anchored on purpose: a nested helper (`stop_it`, `phantom`,
# `bare_start`) is indented and local to its beat, and those names repeat
# legitimately. `$1` is the script directory.
one_name_one_definition() {
  dupes=$(grep -h '^[a-z0-9_]*() {' "$1"/harness.sh "$1"/stories.sh "$1"/beats_*.sh \
    | sed 's/() {.*//' | sort | uniq -d)
  [ -z "$dupes" ] && return 0
  echo "drive: one beat function, two definitions — the later silently wins," >&2
  echo "  so whatever the earlier one asserted stops running and leaves no trace:" >&2
  printf '    %s\n' $dupes >&2
  exit 1
}

# --- the seat: claimed per RUN, torn down with the verdict ------------------
# Every run opens with `claim_seat` and closes with `verdict "$out"` — the ONLY
# two places a display is named. A hardcoded seat was a singleton: two drives at
# once stole each other's window focus mid-typing and payloads landed
# doubled/truncated (bl-4132). `YOG_SEAT` is exported here, so every `$drive`
# call and every sourced beats_* file inherits this run's display.
claim_seat() { YOG_SEAT=$("$drive" seat); export YOG_SEAT; echo "seat: $YOG_SEAT"; }
# The one tail every run shares: drop the seat, then the verdict. Pairing them
# means a run cannot report PASS and leak its X server.
verdict() {
  "$drive" unseat
  echo "---"
  echo "screenshots: $1"
  echo "verdicts:    $1/verdicts.jsonl"
  [ "$fails" = 0 ] && echo "ALL BEATS PASS" || { echo "$fails BEAT(S) FAILED"; return 1; }
}
