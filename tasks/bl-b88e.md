+++
title = "steps view: the framing outcome (✔ ✖ ■) is glyph-only — §11 promises the words and the render omits them"
created = 1785287139
updated = 1785373497
claimant = "entrance-b88e"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["ui"]
+++
Glyph-doctrine follow-up (DESIGN §11 "Glyph doctrine", filed by bl-5013).

src/steps_view/render.rs (render_summary/framing_badge): a step row's framing —
complete/failed/killed — is painted only as ✔/✖/■ + hue. Delete the glyph and
the row is seq + attempts + tokens with no outcome at all. DESIGN §11 already
specifies the words: "Steps — steps/NNN table: framing status
(in-flight/complete/failed/stopped, …)" — so this is a doc/code divergence as
well as a doctrine violation. (The §7.3 no-response wound already gets its
sentence; only the ordinary framings are mute.)

Fix per the doctrine: state the framing in text on the row (or hover at
minimum); keep the glyph on top for the glance.