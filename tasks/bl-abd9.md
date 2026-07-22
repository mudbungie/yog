+++
title = "top-level rework: workspace tabs (top right) + conversation-first center"
created = 1784696431
updated = 1784696971
claimant = "Catoblepas"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Intent (operator-stated, 2026-07-21)
Workspaces are regime walls (personal / work / client): separate blast radius, almost invisible — nothing in the UI but a small tab-style selector bar under the top right. Inside a workspace: a + affordance opens the composer; Enter fires the prompt; conversations (root agents) are the first-class unit. Ops plumbing must never read as conversation content. Auth-failed latest step renders Login inline and stirs attention.

## STATE 2026-07-21: ~95% DONE, NOT CLOSED — resume here
All work is committed as **WIP commit f5cc2bf** on the claimed worktree branch:
/home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/yog/bl-abd9 (tree clean; claimed by Catoblepas).

Done: DESIGN §11 rewritten (+§5.3, §6, §12, §15 M6 Z9) and STORIES de-rostered; new tested VMs nav/tabs.rs, nav/convs.rs (+tests), opslog::Activity, login::latest_step_auth_failed (proven on the real <workspace> kind:auth shape), theme::state_badge, AppModel surface (tab_bar, conversations, conversation_members, activity, is_collapsed); old nav roster + git_tree/render.rs subtracted; shell recomposed (navigator, workspace center w/ Login banner, activity.rs, ball_bar.rs, bottom input_bar, start_pane mint→tab ＋, 2-frame acceptance harness). cargo test 708/708, make lint green, caps hold.

## Remaining (in order)
1. make coverage: tarpaulin 99.98% — cover the last line(s); suspects convs.rs/tabs.rs/focus.rs edge arms.
2. Merge main (>= dffbab1, the BRAZEN_CONFIG world change) into the worktree; re-run tests.
3. Full make check green, then: bl -C /home/u/dev/yog close bl-abd9 --as Catoblepas (retry once on store race).
4. After close: reinstall (make install), restart yog, and re-drive stories per bl-193b.