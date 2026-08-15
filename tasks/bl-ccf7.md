+++
title = "REMOTE §9.5 residual — the window becomes a seat: the frame's read path rides the wire"
created = 1786756326
updated = 1786757176
claimant = "Sheave"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-8bbc"
on = "claim"
+++
The second half of REMOTE §9 step 5, recorded as a residual in §9.5 when bl-b6fa landed the channel: the wire, the mTLS server, the framing (u32-length-delimited JSON, zero-length terminator) and a terminal seat ('yog seat') all exist and are proven end to end over loopback — but the window still holds the engine it serves. Server-and-seat in one process, not seat-over-wire.

Deliverable: the window's read path rides the wire. A frame paints from decoded replies obtained over the seat transport (src/wire/client, boundary::reply::decode) instead of adopting the in-process snapshot; the seat polls at human cadence per REMOTE §3, with the UI/backend isolation principle intact (the frame never blocks on the transport — an off-frame asker lands answers, the frame renders whatever has landed, the search precedent). Also in scope, now that a transport makes the distance real: narrow the path-typed reply residuals REMOTE §8 lists (Applied{file}, Marks{space}, worktree paths, Prepared::binding). Decide whether a held connection replaces connection-per-gesture, and whether any read graduates to the follow-stream form the framing already carries (§10 opens) — record the rulings in REMOTE §8/§10.

Blocked on bl-8bbc (boundary-surface serialization; and the per-seat ui.json split changes what a seat reads for its pane facts). Verify the tree before editing; this body drifts.