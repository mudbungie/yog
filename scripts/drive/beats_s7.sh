#!/bin/bash
# beats_s7.sh — the S7 Forensic beats and world C's runner `run_s7`, which also
# fires the S6/S4 residual beats in `beats_s6.sh`. Sourced by stories.sh; not an
# entry point of its own.
#
# World C is one bare conversation on the live wire, then a **laid** forensic
# state around it: a child agent carrying a `refs/litany/*` mark, a second
# (newer) root, a pending inbox deposit, and a malformed step file — plus the two
# marks `s6_attention` lays for itself, one beat before it spends them, which is
# where a fixture whose beat must WATCH it land belongs. Every one of those is
# litany's own on-disk representation written by hand — a branch one well-formed
# descent segment below the root IS a child (§2.3, `CHILD_SEG` below), a ref IS a
# mark (§6) — which is what makes S7/S6's rows drivable
# without a second live agent, a real budget exhaustion or a real conflict. The
# fixture is honest about being a fixture: it writes what litany would write and
# nothing yog owns.

# --- world C fixtures --------------------------------------------------------
# The workspace, its root agent id, and its bare repo — the three handles every
# fixture below and every predicate needs.
ws_here() { find "$1/yog/workspaces" -maxdepth 1 -mindepth 1 -type d | head -1; }
root_agent() {
  git --git-dir="$1/repo.git" for-each-ref --format='%(refname:lstrip=3)' refs/heads/agents \
    | head -1
}

lay_forensics() {
  ws=$1 ; ag=$2 ; g=$ws/repo.git
  tip=$(git --git-dir="$g" rev-parse "agents/$ag")
  tree=$(git --git-dir="$g" rev-parse "agents/$ag^{tree}")
  # A child agent (§2.3): hierarchy lives in the id, so a branch whose id is the
  # root's plus one descent segment off the root's tip IS a member of the
  # conversation — nothing else to write. Its `conflicted` mark is what makes
  # selecting it PROVABLE: focus is RAM (§13.1), but acknowledging is focusing
  # (§6), so the member click leaves the child's own watermark in ui.json. A
  # child's work-product conflict is exactly what that ref is for (§2.6), so the
  # fixture is a state, not a trick.
  child=$ag-$CHILD_SEG
  git --git-dir="$g" branch "agents/$child" "$tip"
  git --git-dir="$g" update-ref "refs/litany/conflicted/$child" "$tip"
  # A second root, deliberately NEWER than the flagged one, so the §6 sort has
  # something to beat: by recency it would head the roster.
  second=$(git --git-dir="$g" commit-tree "$tree" -p "$tip" -m "yogdrive second root")
  git --git-dir="$g" update-ref "refs/heads/agents/$SECOND_ROOT" "$second"
  # §6 rules 3 and 4 are NOT laid here: `s6_attention` (beats_s6.sh) lays its own
  # two marks on this root, immediately before the walk that acknowledges them.
  # Laid here they were ~30 s and three beats early, and `s7_steps`'s ↑ walks the
  # selection back onto this agent in between — §6's ack is held on every frame a
  # conversation stays focused, so that ↑ acknowledged both, and all three S6-T1
  # rows passed on it with their own gesture deleted (bl-1061).
  # §6 rule 5: pending mail nobody is driving — a deposit file (§2.11) with the
  # lock free, the one signal that is deliberately NOT silenceable.
  # The file name is litany's own framing — `<sender-agent-id>-NNN.md` (§2.11) —
  # so the sender is an agent id, the child laid above, never a human name. It
  # has to be the child specifically: `litany scan`'s sweep reads this very
  # listing to answer "did the child ever deposit?" (§2.6 return), and a sender
  # that is not the child makes the sweep deposit a `died` message of its own.
  mkdir -p "$ws/inbox/$ag"
  printf -- '---\nfrom: %s\ndeposited_at: %s\nepitaph: final-response\n---\nmail nobody is driving\n' \
    "$child" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$ws/inbox/$ag/$child-001.md"
  # A step file yog cannot parse: it must render an ERROR ROW rather than vanish,
  # and its siblings must still build (§11 "a file yog cannot parse").
  printf '{ this is not json' > "$ws/steps/$ag/001/request.json"
}

# The second root's id: not hyphen-prefixed by the real root, so it is a root and
# not a member of its conversation.
SECOND_ROOT=20260727T090000Z-yogdrive2

# The laid child's own descent SEGMENT — `<ts>-<short>`, exactly two hyphen-free
# tokens, which is the whole of litany's id grammar (litany ARCH §2.3: an id is
# its parent's id plus one such segment). So the child branch is a four-token id
# whose derived parent is exactly the root. Until bl-c03e this fixture laid a
# one-token suffix (`$ag-c0ffee`) — a legal branch name, but an id litany would
# never mint: three tokens, so stripping the last segment derives the bare
# timestamp, a ref nobody holds, and `litany scan` died on git's 128 (litany
# bl-025b). Two tokens is not decoration here; it is what the beats assert.
CHILD_SEG=20260727T090100Z-c0ffeeba

# --- world C predicates -----------------------------------------------------
# `seen_kind` — the `seen[ws][agent].<kind>` watermark read — now lives in
# predicates.sh, the read tier every run shares: the S6 beats read the same fact off the
# same file, and a predicate two runners assert on has one home (bl-2d45).
# `seen_agent` — a bare `grep -q "\"$2\"" ui.json` — is GONE rather than kept for
# convenience: it had no caller left after that move, and an unreached vacuous
# predicate is a loaded gun on the shelf. It answered "this string appears
# somewhere in ui.json", which is true of a focused agent, of a path component,
# and (on an empty id) of every file (bl-f16e). `seen_kind` is the spelling.
alive() { kill -0 "$1" 2>/dev/null; }

# --- the run ----------------------------------------------------------------
run_s7() {
  data=$1 ; out=$2
  mkdir -p "$out" ; rm -rf "$data" ; mkdir -p "$data"
  claim_seat
  seed "$data"
  ops="$data/yog/world/state/yog/ops.jsonl"
  ui="$data/yog/world/state/yog/ui.json"
  launch_engine "$data" ; pid=$engine_pid ; wid=$engine_wid
  sleep 1
  # One live conversation — the thing every forensic surface below reads. Escape
  # then `i` is §11's deterministic "put the cursor in the composer" idiom from
  # any state: the release gesture, then the focus binding (the bootstrap box
  # opens focused, so a bare `i` would type an `i` into the goal).
  bare_start() {
    "$drive" bare "$wid" i
    "$drive" type "$wid" "Respond with exactly this text and nothing else: Forensic wire OK."
    "$drive" key "$wid" Return
  }
  until_landed bare_start verb_ge prompt 1 \
    && pass "S7 fixture: conversation on the wire" \
    || fail "S7 fixture: conversation on the wire" "no prompt"
  ws_root=$(ws_here "$data")
  # Its wall (§16.2) — brazen config and sign-ins — was laid with the world seed,
  # before the launch (bl-1851). This start is one gesture, mint and prompt
  # together, so a wall laid here would already have missed the first model call.
  await reply_exists \
    && pass "S7 fixture: wire reply on disk" \
    || fail "S7 fixture: wire reply on disk" "no gpt reply in 40s"
  # Select through the §11 binding (↓ lands via focus_agent) and photograph the
  # single-agent case FIRST: a conversation with no children has NO descent to
  # unfold, so its row grows no subagent field (S7-T5's negative half, and the
  # screenshot is its proof).
  "$drive" bare "$wid" Down ; sleep 2
  "$drive" shot "$wid" "$out/s7-01-no-descent.png"
  agent=$(root_agent "$ws_root")
  lay_forensics "$ws_root" "$agent"
  sleep 3

  s7_descent "$wid" "$out" "$agent"
  s7_steps "$wid" "$out" "$pid"
  s6_attention "$wid" "$out" "$agent"
  s7_inbox "$wid" "$out"
  # The overflow beat goes LAST of the window beats: its menu row is one click
  # away from *focusing* the foreign workspace, and a focus change would retarget
  # everything after it (the first run of this file did exactly that and the
  # uncoloured-id prompt landed in the wrong sphere).
  s4_uncoloured "$wid" "$out" "$ws_root"
  s4_overflow "$wid" "$out" "$data"
  "$drive" stop "$pid" ; sleep 1
  s6_converges "$data" "$out"

  verdict "$out"
}

# S7-T5 — a descent exists only with children, the conversation's list row
# unfolds one row per member, and selecting a member is the §6 acknowledgement
# gesture. That last clause is what makes this assertable: the member selection
# writes the CHILD's watermark into ui.json, so an invisible selection leaves a
# visible fact.
#
# KEYS, not a click (fixes bl-52c7, which is bl-20f4's defect one altitude in).
# That ball's premise — "§11 binds the roster, not a pick among a conversation's
# members" — stopped being true at bl-fa82: ↑/↓ walk the visible LIST rows, and
# a member is one of them once its parent is unfolded. So the same cure bl-20f4
# applied to the conversation roster applies here, and the stale coordinate that
# broke twice (the tree moved down under two new attention lines) is gone rather
# than re-measured. The root row is already selected by the caller's ↓.
# → unfolds it; the next ↓ steps into the first child, which is `list_step` →
# `focus_agent` — the very same call the deleted tree's click made.
s7_descent() {
  wid=$1 ; out=$2 ; agent=$3
  "$drive" bare "$wid" Right ; sleep 1
  "$drive" shot "$wid" "$out/s7-02-unfolded.png"
  step_to_child() { "$drive" bare "$wid" Down; }
  until_landed step_to_child seen_kind "$ui" "$agent-$CHILD_SEG" conflicted \
    && pass "S7-T5 descent: selecting a member retargets (child watermark)" \
    || fail "S7-T5 descent: selecting a member retargets (child watermark)" "no child seen record"
  "$drive" shot "$wid" "$out/s7-03-member.png"
}

# S7-T1/T2 — the tabs are the §11 digit keys (a bound gesture, so no clicks here),
# the Transcript Raw toggle yields verbatim bytes, and the malformed step file
# stays inspectable while its siblings still build. Every one of these is a pure
# read: the ops trail must not grow, and yog must still be alive at the end — a
# parse that panics is the failure this rules out, and it is a real one to rule
# out, since the fixture's `request.json` is deliberately not JSON.
s7_steps() {
  wid=$1 ; out=$2 ; pid=$3
  lines=$(wc -l < "$ops")
  # KEY, and it is the whole subject: the malformed `request.json` is laid under
  # the ROOT's `steps/<agent>/001/`, and `s7_descent` above left the selection on
  # the laid CHILD, which has no steps at all. So the Steps tab this beat opened
  # said `(no steps yet)` and its three clicks had nothing to land on — for
  # twelve days, silently, because the beat's assertion is a negative that a
  # click into blank panel satisfies (bl-5cce). ↑ is §11's step to the previous
  # visible row, which from the unfolded child is its parent: the agent whose
  # steps this beat is about. It also restores what `s6_attention` below states
  # it inherits — "the selection is ALREADY the in-flight root".
  "$drive" bare "$wid" Up ; sleep 1
  "$drive" bare "$wid" 2 ; sleep 2
  "$drive" shot "$wid" "$out/s7-04-steps.png"
  # CLICK (VIEWS, and picks — §11 rule 2): WHICH step to drill into, then WHICH
  # record — the `request` tab, the one holding the malformed file. Both DERIVED
  # from the frame just photographed (bl-5cce, `locate.sh inspector`) off the
  # rule the centre paints between a conversation's header and its tab strip.
  # The numbers they replace were measured on 2026-07-26 and were wrong from
  # bl-1ff1 on, which put a Raw checkbox above the step selector; bl-1ca2's
  # centre tab strip then moved the whole inspector down again. Everything both
  # balls inserted sits ABOVE that rule, so nothing below it can learn of them.
  read -r raw_x raw_y step_x step_y rec_x rec_y \
    < <("$here/locate.sh" inspector "$out/s7-04-steps.png")
  "$drive" click "$wid" "$step_x" "$step_y" ; sleep 2
  "$drive" click "$wid" "$rec_x" "$rec_y" ; sleep 2
  "$drive" shot "$wid" "$out/s7-05-malformed.png"
  # Back to Transcript (the §11 digit key) and its Raw toggle — CLICK (a VIEW):
  # it changes how the same bytes are rendered and nothing else, so it is §5.3
  # RAM and crosses no boundary by design. Re-derived off the Transcript frame
  # rather than re-used from the Steps one: the two tabs carry different control
  # rows, and a point is read from the frame it is about to drive or it is a
  # pinned number again.
  "$drive" bare "$wid" 1 ; sleep 2
  "$drive" shot "$wid" "$out/s7-05a-transcript.png"
  read -r raw_x raw_y _ _ _ _ < <("$here/locate.sh" inspector "$out/s7-05a-transcript.png")
  "$drive" click "$wid" "$raw_x" "$raw_y" ; sleep 2
  "$drive" shot "$wid" "$out/s7-06-raw.png"
  { [ "$(wc -l < "$ops")" = "$lines" ] && alive "$pid"; } \
    && pass "S7-T2 steps: malformed step stays inspectable, no crash" \
    || fail "S7-T2 steps: malformed step stays inspectable, no crash" "ops grew or yog died"
}

# S7-T4 — the Inbox tab explains `✉n` and the flush is `litany scan`. The verb is
# the assertable half; the deposit's parsed from/deposited_at/epitaph are the
# screenshot's. Note where the flush actually lives: the Inbox tab renders the
# deposits only, and the dispatcher is the composer's `Scan` (§8.2) — one verb,
# named once, not a second button inside the tab.
s7_inbox() {
  wid=$1 ; out=$2
  "$drive" bare "$wid" 3 ; sleep 2
  "$drive" shot "$wid" "$out/s7-07-inbox.png"
  # `f` is §11's Flush — the composer's `Scan` verb (§8.2) on the focused
  # workspace, the same dispatcher the button calls.
  flush() { "$drive" bare "$wid" f; }
  # The OUTCOME, not merely the dispatch: `row_ok` is the ops row with `exit:0`,
  # so this asserts that the verb yog spawned actually succeeded. It could not,
  # until this file's own `CHILD_SEG` fix: the sweep derives each `agents/*`
  # branch's parent address by stripping one two-token segment, and the old
  # one-token `$ag-c0ffee` derived an address no ref held, so git's 128 aborted
  # the whole pass before the flush ran (the mail was a bystander). litany fixed
  # its half too — c816ee8 intersects the derived address with the `agents/*`
  # registry — and that IS in the litany release yog pins and embeds (the pin
  # authority is `Cargo.toml`, never a version copied into a comment here; the
  # verb runs through the §16.7 self-multiplex, never the `litany` on PATH), so
  # both halves of the fix are now under this beat.
  until_landed flush row_ok '"litany","scan"' \
    && pass "S7-T4 inbox: Flush's litany scan exits 0" \
    || fail "S7-T4 inbox: Flush's litany scan exits 0" "no clean scan row"
  "$drive" shot "$wid" "$out/s7-08-flushed.png"
}
