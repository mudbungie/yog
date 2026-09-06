+++
title = "the learning loop is unreachable from the control boundary: no gesture reads, accepts or rejects a staged proposal"
created = 1788673872
updated = 1788673872
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r1"]
+++
Scenario: lane KNOWLEDGE, round 1, driving litany's learning loop (docs/DESIGN_LEARNING_LOOP.md) through a yog engine.

WHAT WORKS

Adopting the loop is two gestures: `/config branch default workflow.yaml <text>` with `worker_flush: [dispatch(compactor), dispatch(reviewer)]` and `reviewer_return: [stage_proposal]`, and `/model reviewer <provider> <model>`. A user correction then reached a reviewer, which staged a real skill patch on `proposal/<reviewer-id>`.

WHAT IS MISSING

Reading it, accepting it and rejecting it are `litany proposal <workspace> [<id>] [--accept|--reject]` only. `yog gesture --help` lists no proposal verb, and `/config` addresses `brazen|models|cadence|workflow|branch|orphan|fork` — never `proposal/*`. So from any seat (the lernie window, the android client, `yog gesture`) the staged patch is invisible: `/lineages` does not list proposal refs and `/governing` reads the followed commit, not a candidate.

I could only read and accept mine because the engine was on this box:

    $ yog litany proposal <yog-data-root>/workspaces/notes
    ID                       LINEAGE  PARENT        STATE  DIFF                             SUBJECT
    20260906T...-3a7ccad2    default  fa9d5d381f9d  fresh  1 file changed, 6 insertions(+)  ...
    $ yog litany proposal <ws> <id> --accept
    accepted <id>: config/default now stands at 71011c3dec3e

On a server install (DESIGN §10.1, the ghcr image under its own unit) that is an `ssh` and a container `exec`, which is exactly what the §8.5 boundary exists to make unnecessary.

EXPECTED

Three gestures, mirroring the verb: a read (`/proposals`), one proposal whole (`/proposal <id>`, message plus diff), and the settle (`/proposal <id> accept|reject`). The accept is the operator act the whole design turns on — "can a person see what was learned, and veto it" — and today the answer at a seat is no.

SEVERITY

p2: the feature works and is genuinely good; it is just not exposed. It is the single highest-leverage thing in the knowledge lane that is one gesture away.