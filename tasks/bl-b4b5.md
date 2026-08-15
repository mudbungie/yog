+++
title = "REMOTE read path: the accessory tail, part two — the ball-and-spend family, the fire-time gates, the misc singles, and staleness's clock"
created = 1786770804
updated = 1786771043
claimant = "Scupper"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
bl-296f took the first seven of bl-48ae's accessory tail and found they needed no new query at all: the facts were already on landed answers (Query::Workspaces, Query::Ops, Query::Agent) or wanted one field on one of them. What is left does need spelling. REMOTE 9.7's bl-296f block is the authority; this is the work.

(a) The 3.2/3.5 ball-and-spend family: ws_balls, roster_ball_rows, bound_ball, focused_join, ball_spend, conversation_spend, conversation_context. Take the altitude as one question, not eight. Query::Balls already answers the join rows but answers them PATH-TYPED (JoinRow carries project and workspace as paths; 8.1's narrowing did not reach them), so a seat holding a 3.1 name cannot select its workspace's balls out of that answer without the join bl-7407 refused. So: one workspace-addressed question answering the bound balls with their figures, and the 8.1 narrowing of JoinRow rides with it. The two conversation-scoped figures are AgentView's shape - facts about one conversation's subtree, like flight and strip.

(b) The 3.6/8.4 fire-time gates: delete_confirmation, agent_delete_confirmation, move_targets, conversation_names. The rendering split is already exact and must stay so - the chokepoint re-derives every confirmation fail-closed at fire (9.8, bl-1747), so the seat's copy is a painted affordance and may land an ask period late.

(c) The misc singles: focused_pending (the 5.1 #11 queue), agent_titles (the 3.3 table a seat resolves a third party against), config_tip (the 2.2 lineage tip the 9.4 drift clause reads).

(d) staleness, which is the one item in the tail that is not scope. The 7.2 staleness line is the age of Snapshot::derived_at, an Instant; the chokepoint takes now_unix, an i64 minted at the process boundary precisely so every derivation is deterministic under test. So it cannot be answered until the snapshot carries its completion as a wall-clock stamp - a payload-and-clock change rather than a read migration. growth_note is painted in the same two-line loop and moves with it.

startable/resumable stay excluded (the acts side of bl-adcb's line). The 9 config editors' own loads and Prepared::binding stay where bl-f297 and 8.1 left them.

Standing discipline: a read is a standing question; a fact that gates a gesture is read off a LANDED answer, never awaited at click; the accessor is deleted with the migration; the wire adds no verbs - a missing capability is a boundary Query/Reply on every face, all serializations. Where a fact already rides a landed answer, fold it there rather than minting a near-duplicate. Verify every named accessor against the tree before editing; this body will drift.