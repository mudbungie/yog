+++
title = "the 'recent' conversation sort isn't recent: it keys on last committed step and is outranked by attention/running tiers — sort by last action of any kind"
created = 1785649466
updated = 1785649488
claimant = "recent-sort"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Operator complaint (2026-08-01, verbatim)

'the "recent" sort of conversations doesn't make a ton of sense to me. it should be by what agent last had any action.'

## Verified code facts (verify again before editing)

- The flat 'recent' ordering (src/nav/convs/row.rs `build`, ~line 98) sorts `attention > running > recency, then root id` — recency is the THIRD key, so an attention-flagged or running-but-stale conversation outranks one that acted a second ago.
- The recency key is `last_active = max(members' tip_timestamp_unix)` — the **commit timestamp of each agent branch's tip** (src/git_tree/enumerate.rs ~63-93), i.e. the last *committed step*. Activity that has not landed as a commit moves nothing: a streaming inference (live tail growing), a running tool, a just-delivered message file. The same value drives the row's age label.

## What 'recent' must mean (the operator's words are the spec)

The recency key becomes **the last observed action of any kind across the conversation's subtree**: max over members of
- tip commit timestamp (committed steps — today's key),
- newest `messages/` entry mtime (deliveries and results land as files),
- the live streaming tail file's mtime when the agent is in flight.

And the flat 'recent' ordering becomes literally that key, descending — the attention/running rank tiers LEAVE the sort. Attention and liveness stay visible as what they already are (the badge, the pulse, the attention count); they stop reordering the list. Note: this reverses the §11 'attention > running > recency' doctrine — that is deliberate, by operator ruling; amend DESIGN §11 accordingly, recording the old order and why it fell (a sort that pins stale rows above active ones reads as broken to the operator scanning for what just moved). Grouped-by-ball view: unchanged mechanics — it partitions the already-sorted rows, so it inherits the new order for free (verify group.rs makes no independent rank assumption).

## Discipline

Purity holds: the mtimes are gathered at enumerate/snapshot time alongside tip_ts (yog reads disk statelessly; the view stays a pure projection of the snapshot — never a per-frame stat from the render path). Age label follows the same fact — one recency, one home. Test both directions: a streaming/just-messaged conversation with an old tip sorts above a fresher-tip idle one; attention no longer reorders; deterministic tie tail (I9) stays. Concurrent work warning: another fleet's agent (naming-arch) holds bl-50f3 touching display-name plumbing near nav/convs — verify the tree before editing and fold close conflicts honestly.