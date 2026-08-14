+++
title = "design investigation: thin-client/backend split — one yog backend on the home server, multiple UI clients sharing chats, each client exposing its machine-local tools"
created = 1786682408
updated = 1786682490
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
the operator's question (2026-08-13): what would it take to keep a split in yog — a thin UI client talking to a backend — so one yog instance on the home server serves multiple clients on multiple systems, sharing chats, with each client able to expose local tools to interact with its own system/environment.

Deliverable of the first pass: an architecture assessment (what already supports the split, what has to change, rough sequencing). If the direction is adopted, the follow-on deliverable is a living design doc (docs/), not a growing task body.

---

Architecture assessment (first pass, from a full-tree survey):

ALREADY DONE (the hard part): src/boundary/ is a formal, compile-gated command/query/reply surface — Gesture{Act(Action),Ask(Query)} with 25 Actions, 12 Queries, 22 Replies, three serializations (JSON codec, slash line, GUI), strict decode, refusal semantics. 'yog headless' boots the identical engine minus the window (Engine::boot spawns worker/bridge/consumer/sentry/pilot/follow; the frame only paints snapshots). deposit.rs is already RPC: O_EXCL-minted ids, rename-claim mutual exclusion, durable replies — a directory-shaped server. boundary/, control/, world/, fleet/, monitor/ etc. are zero-egui.

MISSING, by cost:
1. DOMINANT: the transcript/inspector (the chats themselves) is NOT on the boundary — shell/inspector/vms.rs reads disk on the frame thread; there is no Query::Transcript/Steps/Rail/Files/Inbox. Promote these to Query/Reply pairs (WorkDiff/Search are the precedent).
2. Reply is encode-only and hand-rolled (no serde derive anywhere, deliberate); a thin client needs the decode side (~600 lines) or a serde ruling.
3. Shell still paints raw GitTree/Agent in places — finish the view-model promotion.
4. Addressing is absolute local PathBufs on the wire; needs a workspace-identity layer (names, not paths).
5. Transport: swap deposit/consume for a socket server at that exact seam (~4 files, bounded) + auth/identity (none exists today).
6. ui.json is one shared last-writer-wins doc; pins/panel-sizes are per-seat, watermarks shared — needs partitioning.
7. Doc ratification: DESIGN §14 rejects 'Daemon/socket/IPC between instances — disk is the bus'; VISION §4.8/V5 sanction remote teleop of the one dispatch surface. Frame the socket as a client transport for the §4.8 boundary, never instance coordination; amend DESIGN before building.

CLIENT-LOCAL TOOLS is the genuinely new element: tool execution lives in lernie's driver (bash on the engine host); yog only seeds PATH shims and adjudicates via 'yog tool-control' on stdio. Agents acting on a CLIENT machine = a reverse channel (per-client tool host the backend routes into) and an upstream lernie ask, not a yog-only change. Separate design.

SEQUENCING: (1) ratify direction in DESIGN; (2) boundary-complete the reads — valuable standalone for teleop V5 even with no network; (3) Reply decode/serde ruling; (4) identity layer; (5) socket+auth; (6) ui.json split; (7) client-local tools as its own design with lernie. Steps 2–4 are pure boundary-completion and de-risk everything later.
