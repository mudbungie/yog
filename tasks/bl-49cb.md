+++
title = "sending a start goal leaves the view on the new-conversation placeholder, not the started conversation"
created = 1785646875
updated = 1785646875
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Repro: bare world, type a goal in the composer, Enter. The conversation mints and the reply streams — but the center pane stays on the 'new conversation' form ('select a conversation — or start one below') and the reply streams unwatched; the operator must press Down to see their own conversation. DESIGN §3.4: the raise focuses what it raised; STORIES S0 step 3: 'the reply streams into the focused view' — that is the Codex bar payoff. Verify against current main first (observed on 82031f1 via the drive harness, screenshot state: conversation row present + flagged, center pane placeholder). Fix: a fired start focuses the conversation it started. Acceptance: S0 drive beat asserts the transcript view renders the streaming tail with no selection gesture after Enter.