+++
title = "design the project-work contract: reconcile balls delivery with lernie agent history"
created = 1785649814
updated = 1785649814
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design"]
+++
Source: bl-e249's Claude Code comparison, 2026-08-01. Verify every premise against the then-current pins before ruling.

## Contradiction, not a feature request

A ball-bound yog root currently spans two Git worlds:

- balls owns the project worktree and the branch that `bl close` delivers;
- yog places that absolute path in `goal.md` and starts `lernie prompt` with that process cwd;
- pinned lernie 0.0.3 runs every tool in its own `<workspace>/agents/<id>` worktree and `commit_tool` stages only that tree.

Therefore project edits are outside lernie's commit-per-side-effect history. They are not carried by an agent ref, inherited by a child, present in a lernie replay, or isolated when siblings run. Current `docs/VISION.md` nevertheless says sibling terminal refs are diffable and "The winning variant's branch is what the ball's close squashes." Current `docs/DESIGN.md` also says tools inherit the driver's cwd; actual pinned code does the opposite.

Lernie main commit 58a9271 adds a durable per-agent `cd` mark, but its own ARCH says external edits are "uncommitted — off its branch, invisible to a parent, absent from replay" and the mark deliberately does not cross forks. A creation-time cwd alone does not resolve the contradiction.

## Deliverable: a cross-suite design ruling before implementation

Amend `docs/VISION.md` and `docs/DESIGN.md` so one architecture answers:

1. Which repo/ref/worktree is authoritative for project bytes for a root, child, fan candidate, judge, and adopted winner?
2. How is the target introduced by mechanism, inherited, validated after deletion/move, and exposed without parsing goal prose?
3. How are mutating children and fan candidates isolated so two writers never share one balls worktree?
4. Which commits capture project side effects, and how do transcript history, project diff, bundle/replay, crash recovery, and balls close join without storing a second copy?
5. What exactly does Adopt select, and which branch does balls squash?
6. How do bare, path, and ball starts differ?
7. Which lower-crate implementation tasks and release/pin sequence follow?

Preserve the layer law: balls owns project worktree lifecycle and delivery; lernie owns agent context/transcript; yog owns the policy tying them together. Reject path parsing from prose, a yog-side index, and two stored copies of project bytes.

## Downstream

This ruling gates the project-diff premise in bl-3746, project-instruction discovery, VISION V1-V3, and the project side of the armed fleet.