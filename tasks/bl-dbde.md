+++
title = "the remote teleoperator cannot set priority, blockers, hierarchy or tags—the facts its fleet schedules"
created = 1787206351
updated = 1787206351
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["missing", "boundary", "balls", "agentic"]
+++
Yog's board reads balls' `priority`, `tags`, `parent` and blocker graph, orders ready work by priority, and the fleet selects that order.

The control boundary exposes only title/body/name on create and update. Balls itself supports priority, tags, parent, subtasks and arbitrary claim/close blockers. A local operator can escape through `yog exec bl`; a remote seat cannot.

Consequently a remote coordinator can arm the fleet but cannot reprioritize its queue, gate dependent work, express decomposition, or add policy tags through yog. Those are not optional metadata: they decide what the autonomous loop runs.

## Required design result

Expose the scheduling facts needed for whole-day teleoperation through the existing create/update boundary without cloning balls' grammar or storing a second representation. Define read/write parity, validation and help for each accepted fact, plus a real boundary drive that creates priorities and blockers and observes the resulting board/fleet order.