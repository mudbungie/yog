+++
title = "chat scroll is sluggish and sticky — find the frame-time cost and make scrolling free"
created = 1785733752
updated = 1785734191
claimant = "scroll-surgeon"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-03, verbatim: 'the scroll in the chat window is weirdly sluggish, sticky.'

MEASURED (release build, temporary timing harness, removed before commit):

The conviction is per-frame disk rebuild + reparse of snapshot-derivable view-models, dominated by the steps tree:

- steps_view::build re-reads and re-parses EVERY step's whole response.json (framing + segment_count + spend_from_bytes each walk every JSONL line; spend full-parses each line with serde). The center panel ran it TWICE per painted frame regardless of tab (auth banner at shell/workspace.rs, wound predicate for WoundGrace):
  - 20 steps x 200 lines (~230 KB): 3.5 ms/build -> 7 ms/frame
  - 50 steps x 500 lines (~1.4 MB): 17.7 ms/build -> 35 ms/frame
  - 100 steps x 1000 lines (~5.9 MB): 72 ms/build -> 144 ms/frame
- transcript::build (whole messages/ dir re-read + parse, per frame while the Transcript tab is open): 0.4 / 1.8 / 5.4 ms per frame at 60 / 200 / 600 entries (mixed md + model json + tool results incl. 100 KB ones).
- rows()+crossings() derivation per frame: 0.2 / 0.7 / 3.3 ms at the same sizes.
- paint of the unvirtualized rows (galley cache warm): 0.4 / 1.3 / 6.1 ms.

With any agent in flight the pulse repaints at ~30 Hz, so the whole stack above was paid continuously; every scroll frame contended with it. That is the sluggish AND the sticky: egui's stick_to_bottom release-on-scroll-up logic was verified sound (read egui 0.29.1 scroll_area.rs; consumption + release + re-engage all correct), so the anchor was never fighting the operator — the frame time was.

FIX (landed):

- SnapMemo (src/app/memo.rs, pub(crate), tested): a one-slot per-snapshot memo. Sound because every disk fact these builds read is change-tracked by the snapshot (Agent::last_action_unix folds the newest messages/ mtime; streaming_text the live tail) — a change that matters forces a publish, so snapshot Arc identity is the invalidation signal.
- InspectorState gains steps_memo + tx_memo. The center builds steps once per snapshot and both banner predicates (login::auth::latest_step_auth_failed, steps_view::latest_step_no_response) now take &StepsView instead of rebuilding from disk. The inspector reads the same memo slot (also feeding the transcript's crossing rules real commit ids) and memoizes transcript::build behind an Arc<Transcript> in TabData, so nothing copies payload bytes per frame.
- Steady-state frame cost for the chat pane drops to rows-derive + paint only (~2 ms typical, ~9 ms at the 600-entry extreme); disk I/O off the frame path entirely, rebuilds at snapshot cadence (debounce-bounded <=10/s).
- NOT fixed because not convicted: row painting is unvirtualized (6 ms at the 600-entry extreme, under budget once the disk work went) — no show_rows/show_viewport rework, no galley cache, per the no-speculation rule.

Stick-yield rule pinned: new wheel-driven interaction test (transcript/tests/tail.rs scrolling_up_releases_the_tail_and_growth_does_not_recapture_it) proves scroll-up releases the tail at once, growth never recaptures a released view, and return-to-bottom re-engages. DESIGN Sec 11 tail paragraph records the rule; Sec 7.2 records the SnapMemo rule with the measured numbers; the Sec 7.3 wound row now says the disk half rides the snapshot too (grace-window reasoning unchanged).