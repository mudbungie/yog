+++
title = "REMOTE §9.5 — the wire: mTLS listener in 'yog serve', client transport in the shell, the window becomes a seat of loopback"
created = 1786684037
updated = 1786688551
claimant = "Binnacle"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-1eb0"
on = "claim"
+++
docs/REMOTE.md §9 step 5 of the client/server split (bl-b9a2).

Operator ruling 2026-08-13: rustls is APPROVED as a direct dependency — it is already in-tree transitively via brazen/ureq and deny.toml already allows the stack (openssl-sys and native-tls stay banned). With it: NO tokio — a synchronous listener over std::net honors rule 8's no-async posture (the rule stays vacuous); NO rcgen — local-CA bootstrap is a make target shelling to existing tooling, out-of-channel per REMOTE.md §1.4. When the dep lands, amend AGENTS.md rule 6's prose to record the approval and adjust deny.toml only if the resolver actually changes the lockfile.

Scope: 'yog serve' = the current headless boot plus the TLS listener speaking the gesture codec (wire adds NO verbs — REMOTE.md §3); client transport in the shell; bare 'yog' becomes a pure client of loopback by default. The deposit inbox REMAINS for in-world callers (§3). This ball decides and records the framing (length-delimited JSON vs JSONL, and the follow-stream chunked form) in REMOTE.md §3/§10. Verify current module names against the tree before editing; this body drifts.