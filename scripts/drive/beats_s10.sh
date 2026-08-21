#!/bin/bash
# beats_s10.sh — S10 Historian, driven through the headless spellings (bl-faca),
# fired from `run_headless`. Sourced by stories.sh; not an entry point of its
# own.
#
# bl-bb20 ruled this rung OUT because "the rail, transcript, steps and files
# surfaces have NO headless spelling at all, so a beat could only screenshot
# them, and a screenshot proves nothing about a spine." bl-6233 then gave every
# one of them a `Query` and a line, and bl-13f9 made the WINDOW read them the
# same way — so the premise expired and nothing came back to say so.
#
# The subject is the SPINE: six surfaces that are pure functions of one
# conversation's own bytes, each asked at the boundary and each answered here
# against content this run can name — the ball id the goal was composed from,
# the marker a deposit carried, the model row the request named. A shape
# assertion would pass on an empty read; every predicate below names something
# the historian could only have got from disk.
#
# **The conversation is the fleet's** (`beats_s18.sh`): a drone the armed loop
# minted on a wall with no sign-in, so its first model call declined at the wall
# and the whole spine exists with nothing spent. That is also why `tokens.total`
# is asserted at zero — on this rung the historian's own reading is the proof
# the run stayed off the wire.

# `/step`'s drill-in and `/files`' preview both name one thing inside the reply;
# `reply_is` reads the whole tail, so these are ordinary expressions over `d`.
s10_historian() {
  data=$1 ; ws=$2 ; agent=$3 ; ball=$4
  # An EMPTY agent would make every predicate below an assertion about the
  # boundary's refusal instead of about the historian — the empty-subject rule
  # (bl-f16e). One red row saying so beats six saying nothing.
  if [ -z "$agent" ]; then
    fail "S10 fixture: the loop's conversation is the historian's subject" \
      "no conversation was minted to read"
    return 0
  fi
  pass "S10 fixture: the loop's conversation is the historian's subject"
  ask() { gesture "$data" "$1" --ws "$ws" --agent "$agent" || true; }

  # S10-T1 — THE TRANSCRIPT. The first entry is the composed goal (§3.3), so it
  # carries the ball's own id: bytes lernie wrote, read back through the query.
  ask /transcript
  reply_is "[r for r in d.get(\"rows\",[]) if r.get(\"kind\")==\"delivered\"
      and r.get(\"sender\")==\"user\" and \"$ball\" in r.get(\"body\",\"\")]" \
    && pass "S10-T1 transcript: the composed goal reads back as the first turn" \
    || fail "S10-T1 transcript: the composed goal reads back as the first turn" "no delivered goal"

  # S10-T2 — THE STEPS. One settled step, framed `failed` because its model call
  # declined at the wall, and its token figures are ZERO — which is this whole
  # run's no-wire claim, made on the surface that would show a spend.
  ask /steps
  reply_is '[r for r in d.get("rows",[]) if r.get("seq")=="001"
      and r.get("framing")=="failed" and r.get("tokens",{}).get("total")==0]' \
    && pass "S10-T2 steps: the settled step is framed, and spent nothing" \
    || fail "S10-T2 steps: the settled step is framed, and spent nothing" "no zero-token failed step"
  # …and the drill-in is the step's own records, not a summary of them: the
  # request the driver really composed, naming the model row it was aimed at.
  ask "/step 001"
  reply_is "(\"$ball\" in d.get(\"request\",{}).get(\"raw\",\"\")
      and \"model\" in d.get(\"request\",{}).get(\"raw\",\"\"))" \
    && pass "S10-T2 step: the drill-in carries the request the driver composed" \
    || fail "S10-T2 step: the drill-in carries the request the driver composed" "no request record"

  # S10-T3 — THE FILES. The agent's worktree listed, and one file previewed by
  # name: `goal.md` is the start's own product, so its bytes are the same goal
  # the transcript's first entry carries — two surfaces, one fact, which is what
  # makes this more than a directory listing.
  ask "/files goal.md"
  reply_is "(d.get(\"worktree\") and [r for r in d.get(\"rows\",[]) if r[\"path\"]==\"goal.md\"]
      and \"$ball\" in d.get(\"preview\",{}).get(\"text\",\"\"))" \
    && pass "S10-T3 files: the worktree lists, and the named file previews" \
    || fail "S10-T3 files: the worktree lists, and the named file previews" "no listing or no preview"

  # S10-T4 — THE RAIL. The spine's notch names the step AND the transcript row
  # it cuts at, which is the join the window paints and the one thing a
  # screenshot could never have proved.
  ask /rail
  reply_is '[r for r in d.get("rows",[]) if r.get("seq")=="001"
      and r.get("row","").startswith("tx/")]' \
    && pass "S10-T4 rail: the notch names its step and its cut in the chat" \
    || fail "S10-T4 rail: the notch names its step and its cut in the chat" "no seated notch"

  # S10-T5 — THE GOVERNING CONFIG (VISION V1.2's freeze): the lineage this
  # conversation is frozen on, by name and by oid, with the files that commit
  # carries. A workspace's own `config/default`, made by real `lernie config`.
  ask /governing
  reply_is '(d.get("branch")=="default" and d.get("oid")
      and "workflow.yaml" in d.get("files",[]))' \
    && pass "S10-T5 governing: the freeze names its lineage, oid and files" \
    || fail "S10-T5 governing: the freeze names its lineage, oid and files" "no frozen config"

  s10_mail "$data" "$ws" "$agent"
}

# S10-T6 — THE MAIL, and it is one beat with two clauses because that is the
# claim: a deposit is in exactly ONE place. `/message` deposits, the executor
# delivers it, and the historian then reads it as the conversation's second
# delivered turn while `/inbox` — the PENDING view — is empty again.
#
# Asserting the pending half full would need a driver holding the executor lock
# across the read, which needs a live model call; this run has none by
# construction. So the deposit is followed to where it lands, which is the
# stronger half anyway: an inbox that never drains reddens the transcript
# clause, and a delivery that never happened reddens it too.
s10_mail() {
  data=$1 ; ws=$2 ; agent=$3
  marker="s10-mail-$$"
  gesture "$data" "/message $marker" --ws "$ws" --agent "$agent" || true
  mail_landed() {
    gesture "$data" /transcript --ws "$2" --agent "$3" || true
    reply_is "[r for r in d.get(\"rows\",[]) if r.get(\"body\",\"\").strip()==\"$1\"]"
  }
  if await mail_landed "$marker" "$ws" "$agent"; then
    gesture "$data" /inbox --ws "$ws" --agent "$agent" || true
    reply_is 'd["ok"] and d.get("rows")==[]' \
      && pass "S10-T6 mail: the deposit is delivered, and the inbox is empty" \
      || fail "S10-T6 mail: the deposit is delivered, and the inbox is empty" "still pending"
  else
    fail "S10-T6 mail: the deposit is delivered, and the inbox is empty" "never delivered"
  fi
}
