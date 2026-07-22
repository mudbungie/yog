+++
title = "top-level rework: workspace tabs (top right) + conversation-first center"
created = 1784696431
updated = 1784696451
claimant = "Catoblepas"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Intent (operator-stated, 2026-07-21)
Workspaces are regime walls (personal / work / client): totally separate blast radius, almost invisible. UI gives them nothing but a small tab-style selector bar under the top right. Inside a workspace you just start conversations: a + affordance opens the composer, Enter fires the prompt (S0/S1 gestures unchanged). A conversation (STORIES: a root agent in a workspace) is the first-class unit on screen. yog's own plumbing (ops trail) must never read as conversation content — today four red 'lernie prime' retry rows sit mid-workspace and were mistaken for the conversation's context.

## Scope
DESIGN §11 rewrite (this ball edits the doc first, then makes code match):
- Top bar: wordmark + attention strip stay; RIGHT side: workspace tab bar — one tab per named workspace + a slim '+' (the deliberate mint, replacing the roster's 'New workspace' button). Foreign/replay workspaces behind an overflow menu, not tabs.
- Left panel: the focused workspace's CONVERSATION LIST — one row per root agent (state badge, first-line preview, age, streaming pulse) sorted attention > running > recency — headed by a '+ conversation' affordance that focuses the composer. Project/ball sections leave the default roster (they return in the ball-views ball; keep a minimal 'balls' collapsible section if trivial).
- Center: the SELECTED conversation — transcript first (streaming tail prominent), descent tree only when children exist, inspector tabs (Transcript/Steps/Inbox/Files/Config) as today. An auth-failed step must surface its Login affordance here (Z8 landed; verify it renders on a real kind:auth response.json — <workspace> step <agent-id>/001 is a live fixture shape).
- Ops pane: demoted to a collapsed bottom accessory ('activity' chip w/ error count); expands on demand; never inline between conversation content.
- Composer: docked bottom as today; targets selected conversation (message) or new conversation (prompt); dir/ball affordances unchanged.
- Attention: a conversation whose latest step failed (incl. auth) must stir the strip — 'nothing stirs' over a dead conversation is a lie. Touch src/attention predicates if needed.
STORIES.md: amend surface language where it names the old roster; gestures unchanged.
Keep view-model/shell split religion: new/changed VMs (conversation list, workspace tabs) are pure + tested; shell stays excluded glue. 300-line caps; pre-split at design time. make check green at 100%.