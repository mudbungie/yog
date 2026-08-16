+++
title = "an agent's own marks space keeps its store but not its worktrees: bl-delivery territory folds off the world's XDG_STATE_HOME"
created = 1786845626
updated = 1786846297
claimant = "Marks-c21d"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "world", "balls", "design"]
+++
DESIGN §16.3 defines a space as balls' state home — "the clone bundle —
landing, store checkout, worktrees" — plus balls' config home. Worktrees are
named in that definition. They are not in that space.

Observed, under synthetic roots with `YOG_MARKS=<marks>` set and a scratch
anchor:

- `yog bl prime` / `yog bl create` found the landing and store at
  `<marks>/balls/clones/<percent-encoded-project>/` — correct, the space's own.
- `yog bl claim <id>` materializes the code worktree at
  `<world>/state/balls/plugins/bl-delivery/<project-path>/<id>` — the WORLD's
  plugin territory, not the space's.

Cause is the seam bl-81c9 examined from the other side. The `bl` arm supplies
balls' two homes explicitly (`Edge::resolve` over `marks::space`), which is
what makes a per-agent branch possible at all. `bl-delivery` is a real
subprocess and rebuilds its own `balls::layout::Xdg` from `$XDG_STATE_HOME` in
its process env (`multiplex::bl_delivery`, mirroring balls' shipped sibling
binary), and `plugin_territory` folds off that. So the injected space reaches
the store and never reaches the territory.

Not a regression from bl-81c9, and bl-81c9 made it strictly better: every
*spawned* agent already split this way, because a workspace-scoped spawn
carries the world overrides and `YOG_MARKS` together; what changed is that the
bare `yog bl` case now splits into the world's territory instead of the
operator's ambient one.

Two candidate answers, and this ball is to pick one and record it in §16.3:

1. The space owns the worktrees, as the doc says. The arm's fold carries
   `XDG_STATE_HOME = space.state` when the space is `own`, so the plugin child
   agrees with the `Edge` — the same "the child resolves the env itself" shape
   bl-81c9 closed for the world.
2. Worktrees are deliberately world-wide (one project checkout territory
   whoever claims), and §16.3's definition drops the word.

Acceptance: one answer, stated in §16.3, and a test that drives a claim under
an explicit `YOG_MARKS` and pins where the worktree lands — the case no test
covers today.