+++
title = "conversation rows: the run-state badge (● ◐ ○ ■, + the ? sigil) is the state's only carrier — no text, no hover"
created = 1785287137
updated = 1785306171
claimant = "entrance-ae05"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["ui"]
+++
Glyph-doctrine follow-up (DESIGN §11 "Glyph doctrine", filed by bl-5013).

The doctrine's test: is the state still legible with the glyph deleted? Here it
is not. On every §11 conversation-list row (src/shell/navigator.rs, conversation_row)
and every descent-tree member row (src/shell/workspace.rs, member_row), the
run-state — Live/InFlight/Quiescent/Stopped from theme::state_badge — is carried
by the glyph alone, and §10 uncertainty by a bare sigil "?". Delete them and the
row says nothing about whether the agent is running.

Fix per the doctrine: make the state sayable — hover text at minimum
(on_hover_text naming the state, e.g. "in flight — a model call is streaming"),
and consider whether the row has space to say it outright. Keep the glyphs; they
become the glance layer over the stated state.