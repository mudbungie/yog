+++
title = "the follow lane calls a HELD conversation 'at rest (quiescent)' and exits: the live view is blind at the one moment the operator is the blocker"
created = 1788746087
updated = 1788746619
claimant = "Cantaloups-Y10"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r2"]
+++
Round-2 devadmin lane. bl-5305 landed and the live view is transformed — a
routed call now streams its name, its input and its exit code:

    {"kind":"follow","ok":true,"stream":{},"tools":[{"input":"{\"command\":\"uptime\"}","tool":"alpha2_Bash","tool_use":"toolu_01…"}]}
    {"kind":"follow","ok":true,"stream":{},"tools":[{"exit_code":0,"tool_use":"toolu_01…"}]}

That is the `codex exec` bar round 1 asked for and it is met.

## What is still missing is the one frame that matters

A conversation whose next act is HELD is reported by `follow` as at rest, and
the follow exits:

    $ lernie --json follow ops KhakiArchway
    KhakiArchway is at rest (quiescent) — nothing more will arrive until it is
    nudged or messaged

At that same instant the conversation's own row said:

    "held": {
      "reason": "beta2_service_status {} classified opaque (beta2_service_status
        is not a tool this control implements and its input carries no command
        line, so what it reaches cannot be read — held rather than passed)",
      "tool": "beta2_service_status",
      "tool_use_id": "toolu_01…"
    }

and `lernie attention` rendered it correctly. So the fact exists, is reachable
by two other reads, and is the one thing the live view drops.

## Why it is worse than a missing field

A hold is not rest. It is the conversation waiting on **the operator**, and the
operator is the person who has `follow` open. The verb whose whole job is
"watch this until it comes to rest" tells them nothing is happening at the
exact moment they are the blocker, and then exits — so a scripted or watching
operator sees a clean completion where a routed call is parked with their name
on it. On a foot lane every call to a non-shell tool is held (bl-b65d), so this
is not a corner: it is most of the conversation.

## Ask

The follow lane carries a hold frame — the tool, the tool_use id and the
reason, the same three fields the agent row already has — and does not treat a
held conversation as rest. `stream.delta` has no room for it; it belongs beside
`tools`, which is where the dispatch and the exit already are.

p2: the mechanism is right and the blind spot is one frame wide.