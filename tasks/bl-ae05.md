+++
title = "REMOTE 1.2 vs 4.1 RULED: the local window is a wire client of localhost — implement the read path over loopback mTLS"
created = 1786757471
updated = 1786763150
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
REMOTE 9.7 (landed under bl-ccf7) records a collision inside REMOTE that no builder may settle, and the read path is stuck behind it.

REMOTE 1.2 rules that the window is a client of the boundary over the same wire a remote seat uses — no in-process face, no fallback. REMOTE 1.3/3 make that wire mTLS only. REMOTE 1.4/8 make the certificates the operator's out-of-channel act: yog mints nothing, links no certificate library, and absence of material is the off switch. REMOTE 4.1, as landed by bl-8bbc, reserves the identity `local` for the window and every other in-world caller, which hold no certificate, are not scoped, and own a pane document — and the identity parser refuses `local` outright, so no certificate can ever claim it.

A window that dialled the wire would therefore present the operator's one client leaf, be identified by that name, be scoped like any remote client and read a different pane document — it would stop being `local`, and 4.1 would be wrong. On a box with nothing provisioned it would have no read path at all: a window that paints nothing. REMOTE 8 has already rejected both ways around that second half — refusing with a remedy puts a terminal instruction in front of a desktop launch that has no terminal, and minting is forbidden by 1.4.

The ruling is one of three (REMOTE 9.7 states each with its cost):

1. Amend 1.2 and the 11 rejection list: one boundary, N transports. The window keeps `local` and its certificate-less in-world standing, and its read path becomes envelope-shaped anyway — a Query encoded with the one codec, handed to the one Answerer, decoded with reply::decode. Byte-identical to the remote path minus the socket, so putting a local seat on the real wire later is a constructor change. Costs the letter of 1.2; the defence is that a transport is not a face, and REMOTE 3 already blesses two intakes into one boundary.
2. Amend 4.1: the window is provisioned like any client. `local` stops being the window's identity and an unprovisioned box gets a window that cannot read.
3. Leave it, and restate 1.2 as aspiration rather than rule.

bl-ccf7 recommends 1 and declined to take it, because a 11 rejection exists so it is not relitigated by whoever holds the keyboard.

Deliverable once the ruling lands: the read path itself. An off-frame asker at human cadence per REMOTE 3, the frame rendering whatever has landed and never blocking on the transport (the search precedent), and the frame staying at 60fps over RAM-resident state. Two rulings already made are inputs to it: connection-per-gesture stands until a seat's ask rate exceeds human cadence and the server's connection loop already supports a held connection (REMOTE 10, bl-ccf7); and the one follow-class candidate is the live model-call tail, everything else being a projection of a snapshot republished on its own schedule.

Also unblocked by the same ruling: the last path-typed reply residual, Prepared::binding (REMOTE 8.1). It stays a path today because an opaque handle needs a mint-resolve table (durable state for a computed fact) and re-deriving at fire means a second derivation of the work target beside the executor's cross-checked one. The shape that dissolves it — Prepared becoming opaque to the seat entirely — is only affordable once one seat's read path is the only read path.