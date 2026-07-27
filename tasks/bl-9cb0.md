+++
title = "marks pane and ball row render for a workspace with no ball: focused_join returns the UnassignedWorkspace row"
created = 1785130699
updated = 1785131546
claimant = "waxier-fix2"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found by drive (docs/drive-logs/2026-07-27-s5-s7-s8-wire-green.md).

The §3.5 join emits an UnassignedWorkspace row per named workspace no ball claims, and that row carries `project: PathBuf::new()` and `ball_id: String::new()` (src/projects/join.rs). `AppModel::focused_join` returns the first row whose workspace matches — so for an unbound workspace it returns THAT row, and two shell surfaces consume it as if it named a ball and a project:

1. src/shell/config_marks.rs: `model.focused_join().map(|r| r.project.clone())` is `Some("")`, so the per-project marks pane renders its heading, the trade text and Read current / Shared / Stealth / Set custom branch for NO project. Clicking Read current runs `bl conf` with cwd "" and the pane prints `failed to spawn bl: No such file or directory (os error 2)` — which reads as a missing `bl` binary, i.e. a toolchain problem, when the truth is 'no project is bound here'. Reproduced live on a world with one workspace and zero projects.
2. src/shell/ball_bar.rs renders `ball ` with an empty id (Close/Release correctly greyed by the enablement predicates).

The pane's own guard text exists for exactly this case ('focus a workspace bound to a project to set its marks') and is unreachable, because `None` never happens for a focused named workspace.

Fix shape (a Rust change, deliberately not made in bl-c517 which is a drive-coverage task): either `focused_join` should skip rows that name no ball (an UnassignedWorkspace row is the absence of a ball, so returning it as 'the focused ball' is the bug), or both consumers should require a non-empty project/ball. The first is one place instead of two. Whichever, the drive beat to add is the negative: a world with a workspace and no project renders NO marks knob and NO ball row.

Test rows this touches: S8-T4 (marks-knob), S4-T3 (join-rows).