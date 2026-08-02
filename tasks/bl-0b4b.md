+++
title = "composer's Scan button moves to the Inbox tab — it's a per-workspace 'lernie scan' flush, not a message-send verb"
created = 1785646051
updated = 1785646052
claimant = "Hardiness"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Currently: src/shell/input_bar.rs:187, the Scan button sits in the bottom composer bar (verb_buttons), beside Message/New-prompt/Stop. It calls dispatch::scan_focused(model, lernie) -> lernie scan <focused workspace> (flush inboxes + died epitaphs, §8.2).

Operator feedback: it reads confusingly among the send/stop verbs it has nothing to do with — scan is an inbox-flush/crash-recovery action, not a conversation verb.

Move it: relocate the Scan button out of the composer's verb_buttons into the Inbox tab (InspectorTab::Inbox), alongside src/inboxview/render.rs's deposit listing — wired in shell/inspector.rs's per_tab_controls, matching the pattern of the existing Transcript-tab Raw-toggle controls. Thread lernie: &Cli through shell/inspector.rs::tabs_and_content (and its caller in shell/workspace.rs::center, which already holds lernie) so the click can still call dispatch::scan_focused(model, lernie).

Keep the 'f' keybinding (KeyAction::Scan, src/shell/keys.rs:139) as-is — unaffected by the button's location.

Update stale doc comments that assert Scan lives in the composer: src/shell/input_bar.rs module doc (line ~5-6, 'the short-verb buttons'), src/inboxview/render.rs:8 ('the Flush = lernie scan action is the composer's Scan verb, wired in the shell').

shell/* is coverage-excluded (tarpaulin.toml) — no new tests required for the glue move itself; run the full suite regardless.