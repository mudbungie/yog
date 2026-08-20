+++
title = "fleet birth failure strands the claim, records spawn success, and leaves no conversation to repair"
created = 1787206306
updated = 1787206306
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "agentic"]
+++
## Reproduction

In an isolated `/tmp/yog-drive` world:

1. Seed a syntactically valid workspace template that cannot start a worker, for example `roles: {}`.
2. Create one ready ball, start `yog serve`, found a workspace with `/prepare`, then arm `/fleet 1`.
3. Wait one pilot tick.

The durable sequence is:

```text
bl claim <ball> --as <workspace>                 exit 0
lernie prompt --name <display> ...              later settles exit -2
yog-fleet spawn <ball> <display>                 exit 0
```

`/board` shows the ball claimed. `/conversations` and `/attention` are empty, and agent-scoped reads refuse the advertised conversation as unknown. Manual `/release` is the only recovery.

## Defect

`dispatch::prompt` returns the minted display name after handing the process to the OS; `fleet::birth` treats that handoff as completed birth and appends a successful spawn row before a conversation exists. S18 also declares that a claim with no conversation is not lease-reapable.

An unattended early driver or configuration failure therefore consumes one fleet slot and one ready ball forever while recording success.

## Required invariant

A successful fleet spawn must mean that a queryable conversation exists. If birth does not converge, the claim must be released or surfaced as actionable attention. Cover the real claim → detached prompt failure → convergence path; a launch handoff alone is not success.