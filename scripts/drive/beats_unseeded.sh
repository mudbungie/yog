#!/bin/bash
# beats_unseeded.sh — THE UNSEEDED FIRST TURN (bl-9e10): STORIES **S0 step 5**,
# driven on the real substrate for the first time. Sourced by stories.sh; not an
# entry point of its own.
#
# WHY IT DID NOT EXIST. Every wire beat in this directory seeds the workspace
# wall before the launch (bl-1851, `wall.sh`) — it had to, or its payoff beat
# read a config fault as a wire outage. But a seeded wall's first turn SUCCEEDS,
# so the harness has only ever driven S0 step 3, "the reply streams into the
# focused view". Step 5 — *"If the model call needs credentials, the failure
# surfaces as derived agent state (§13.3): the auth-failed step renders with a
# Login affordance one click away"* — has never been driven at all. S0-T6 pins
# it against a FIXTURE workspace laid by hand; nothing has ever asked the real
# substrate whether it produces that shape.
#
# So this verb seeds the world and deliberately does NOT finish the wall, in the
# two degrees `wall.sh` now names — and each degree is a different ruled remedy:
#
#   PHASE A, no wall at all. The seeded template names `openai-chatgpt`, a row
#   that reaches a wall only through `config.toml` (preflight says so outright),
#   so brazen declines CONFIG-kind: `provider error (Config): unknown provider
#   `openai-chatgpt``. That line is the §7.3 banner's own input, and since
#   bl-dd7f the banner pairs it with the §9.1 raw-TOML editor.
#
#   PHASE B, the row table and no sign-ins. The row resolves and has no
#   credential, so the decline is AUTH-kind — S0 step 5 exactly, the shape
#   §8.3's `AuthFailure` classifies into a Login affordance.
#
# NOTHING HERE SPENDS ON THE WIRE. Both phases die *before* the request leaves
# the process: one at row resolution, one at credential resolution. That is what
# makes an unseeded run cheap enough to keep in the ladder.
#
# WHAT IT ASSERTS, AND WHAT IT DELIBERATELY LEAVES TO THE PAINT LAYER. The
# remedies are painted strings, and no assertion in this harness can read the
# glass: the screenshots are visual confirmation, never the transport (the
# STEERING RULE), and the inspector surfaces still have no headless spelling
# (bl-6233). So the two halves are split exactly the way bl-55d8's were, which
# is this repo's own precedent: THE DRIVE proves the real substrate produces the
# failure and captures its real words; an ACCEPTANCE BEAT pins those words
# verbatim and reads the remedy out of the paint output
# (`src/shell/acceptance/wound.rs` for bl-55d8, `.../remedies.rs` for these
# two). Neither half is worth much alone — a paint beat over words no substrate
# emits proves nothing, and a substrate assertion with no painted consequence is
# the same — so each names the other, and the words are the seam.

# --- the two degrees, as world seeds ----------------------------------------
# stories.sh's own `seed` minus the wall's halves — the marker file and the
# template are what make this a yog world at all, and they are not what is
# being withheld.
seed_wall_less() {
  mkdir -p "$1/yog/world/lernie/template"
  cp "$real_world/models.yaml" "$1/yog/world/lernie/models.yaml"
  cp "$real_world/template/providers.yaml" "$1/yog/world/lernie/template/providers.yaml"
}

# --- what a dead first turn leaves on disk ----------------------------------
# THE TWO REMEDIES READ TWO DIFFERENT FILES, so this asserts on two, and each
# phase names the one its own remedy is derived from:
#
#   the TRAIL — `<state>/yog/` : `ops.jsonl` and the §8.1 detached sinks beside
#               it. What the §7.3 banner renders, and what `config_edit::fault`
#               classifies into the §9.1 route (bl-dd7f).
#   the STEP  — `<ws>/steps/*/response.json` : the §13.3 derived agent state,
#               which is what `login::auth::classify` reads for its Login flag.
#
# NAMING THE SURFACE IS THE POINT, not tidiness. The first cut of this beat
# swept the whole scratch world for its needle and phase B's `credential`
# matched the WALL'S OWN config.toml — the fixture the phase had just laid — so
# it would have passed with no conversation, no step and no decline anywhere.
# That is `beat-audit.sh`'s vacuity class reached at the filesystem instead of
# at a grep pattern, and the answer is the same one: assert on the subject.
#
# `<dir> <needle>`; an empty needle is refused, never interpolated into a
# pattern that matches everything (harness.sh's empty-subject discipline).
says_under() {
  [ -n "${1:-}" ] && [ -n "${2:-}" ] || return 1
  python3 - "$1" "$2" <<'PY'
import pathlib, sys
root, needle = pathlib.Path(sys.argv[1]), sys.argv[2]
if not root.is_dir():
    sys.exit(1)
for p in root.rglob("*"):
    if not p.is_file():
        continue
    try:
        if needle in p.read_text(errors="replace"):
            print(p)
            sys.exit(0)
    except OSError:
        continue
sys.exit(1)
PY
}
trail_says() { says_under "${data_root:-}/yog/world/state/yog" "${1:-}"; }
step_says() { says_under "${ws_root:-}/steps" "${1:-}"; }

# A step SETTLED AS A FAILURE: some `steps/*/response.json` carries an error
# segment, and no reply message was ever committed.
#
# THE ERROR SEGMENT IS THE CLAIM, and "no reply yet" is only its companion.
# The first cut asserted the pair `a response.json exists` + `no reply` — and
# under this beat's own mutation proof (seed the wall fully, so the turn
# SUCCEEDS) it still passed, because a healthy turn also has a response.json
# open and no reply committed *for the seconds the poll happens to look*. That
# is a race dressed as an assertion. `"type":"error"` is what only a dead turn
# writes, and it is the same segment `git_tree::error_text` reads for the §13.3
# derived state — so the beat and the surface it is about ask one question.
first_turn_failed() {
  step_says '"type":"error"' >/dev/null && ! reply_exists
}

# --- the run ----------------------------------------------------------------
run_unseeded() {
  data=$1 ; out=$2
  mkdir -p "$out" ; rm -rf "$data" ; mkdir -p "$data"
  claim_seat
  # Each phase's needle is the exact byte-string the shipped classifier gates
  # its remedy on, on the surface that classifier reads: brazen's own
  # `unknown provider` on the trail (`config_edit::fault::CONFIG_MARKERS`), and
  # brazen's `kind` field in the settled step (`login::auth`, through
  # `git_tree::error_text`).
  unseeded_phase "$data/nowall" a config trail_says 'unknown provider `openai-chatgpt`'
  unseeded_phase "$data/nocred" b auth step_says '"kind":"auth"' 
  verdict "$out"
}

# One phase: `<data-root> <shot-prefix> <kind> <where> <needle>`. The two differ
# only in how far the wall is laid, which surface carries the decline, and what
# it says there — so they are one body; the alternative is two near-identical
# fifty-line runs whose assertions drift apart.
unseeded_phase() {
  data_root=$1 ; tag=$2 ; kind=$3 ; where=$4 ; needle=$5
  mkdir -p "$data_root"
  seed_wall_less "$data_root"
  # Phase B lays the ROW TABLE and stops (`wall.sh`'s middle degree); phase A
  # lays nothing at all. This is the whole difference between the two runs.
  [ "$kind" = auth ] && seed_wall_config "$data_root" "$BOOTSTRAP_WS"
  ops="$data_root/yog/world/state/yog/ops.jsonl"

  launch_engine "$data_root" ; wid=$engine_wid
  sleep 1
  "$drive" shot "$wid" "$out/s0-$tag-01-launch.png"
  bare_start() {
    "$drive" bare "$wid" i
    "$drive" type "$wid" "Respond with exactly this text and nothing else: Wire check OK."
    "$drive" key "$wid" Return
  }
  until_landed bare_start verb_ge prompt 1 \
    && pass "S0-$kind unseeded: the start fires anyway" \
    || fail "S0-$kind unseeded: the start fires anyway" "no detached prompt"

  # The workspace is a fact yog produced, read back rather than restated.
  ws_root=$(find "$data_root/yog/workspaces" -maxdepth 1 -mindepth 1 -type d 2>/dev/null | head -1)
  # §9.2's birth gate stays retired (bl-00ee): an unresolvable row is not a
  # refusal to create anything, it is a failure at the first dispatch. Asserting
  # the conversation EXISTS is what would go red if that gate ever came back —
  # which is the whole reason this beat drives an unseeded wall rather than
  # asserting the decline in a unit test.
  { [ -n "$ws_root" ] && agents_ge 1; } \
    && pass "S0-$kind unseeded: the conversation is born, not refused" \
    || fail "S0-$kind unseeded: the conversation is born, not refused" "no conversation"

  await first_turn_failed \
    && pass "S0-$kind unseeded: the first turn dies, no reply" \
    || fail "S0-$kind unseeded: the first turn dies, no reply" "a reply landed, or no step settled"
  "$drive" shot "$wid" "$out/s0-$tag-02-failed.png"

  # THE SEAM. The decline's own word is on disk, and it is the word the shipped
  # classifier gates its remedy on — `config_edit::fault::CONFIG_MARKERS` for
  # the §9.1 route, `login::auth::AUTH_MARKERS` for Login. The acceptance beat
  # named in this file's header pins the same word and reads the painted remedy
  # out of the frame; this end proves the substrate really says it.
  "$where" "$needle" \
    && pass "S0-$kind unseeded: the decline names its $kind fault" \
    || fail "S0-$kind unseeded: the decline names its $kind fault" "no $kind decline on that surface"

  # And the conversation is still there to be nudged once the wall is fixed
  # (bl-9bef): a dead first turn leaves a branch, not a hole.
  agents_ge 1 \
    && pass "S0-$kind unseeded: the dead turn leaves a conversation to resume" \
    || fail "S0-$kind unseeded: the dead turn leaves a conversation to resume" "conversation gone"

  "$drive" stop "$engine_pid" 2>/dev/null || true
  engine_pid=""
}
