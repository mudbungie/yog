+++
title = "multiple armed projects have no world-level concurrency or whole-day spend ceiling"
created = 1787206353
updated = 1787206353
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["missing", "design", "agentic"]
+++
Each armed fleet fact names one workspace, one project and one cap. Multiple projects require multiple independently armed workspaces. Each planner enforces only its own cap.

Spend ceilings are also workspace-local; the architecture says the compared figure is the target workspace's. A nominal ceiling can therefore be consumed once per workspace. There is no world/portfolio limit on concurrent drones, total tokens, wall time or spend, and no ordering across projects.

For a whole-day operator this is the safety boundary above every individual queue: starting several fleets must not multiply an intended global allowance silently.

## Required design result

Decide whether yog owns a world-level cap/budget or whether an external supervisor is explicitly the only authority. If yog owns it, derive usage from existing durable facts rather than storing counters, define precedence with workspace ceilings, and drive simultaneous projects proving the total never exceeds the declared world allowance.