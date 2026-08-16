+++
title = "the §8.5 line's in-process query arm still resolves over the frame's cached derivation, so a wall born this instant refuses for one pass"
created = 1786845533
updated = 1786845533
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["residual", "boundary"]
+++
The residual bl-6c9e left, stated so it is not rediscovered as a defect.

bl-6c9e made the **engine's intake** resolve names over the live §3.1 enumeration, so a workspace's birth is a barrier for every gesture and every query that crosses the boundary. One caller was deliberately left reading the frame's cached derivation: `AppModel::boundary_deps` (src/app/balls.rs), which builds the `Deps` for the §8.5 **line's query arm** — the in-process reads the window still answers for itself, plus the acceptance world's stand-in for the transport.

It was left alone for a reason, not by oversight. The frame does no IO (§7.2), so it cannot ask disk; and its own optimistic fold is forbidden here by the standing rule the field's comment states — *the derivation, never the §7.2 fold: a gesture and a machine-facing query may not be decided by a fact that is only optimistic*. Both available answers are refused, so the honest state is: a line query naming a wall born this instant refuses for at most one derivation.

The cost is small and bounded — a read that refuses briefly, never a two-step flow that cannot compose — which is why it did not ride bl-6c9e. Two candidate dissolutions, neither obviously right:

- Migrate the line's query arm onto the same wire path every other window read now takes, at which point the residual dissolves with the caller rather than being fixed.
- Let the arm build its environment where the environment is already built (the intake), which is the same move one altitude down.

Worth taking only when the §8.5 line arm is touched for another reason.