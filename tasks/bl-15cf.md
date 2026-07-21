+++
title = "Z9: post-review hygiene — clip bound, dead field, HOME fallback, draft predicate"
created = 1784603599
updated = 1784604751
claimant = "aster"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-e324"
on = "claim"

[[blockers]]
id = "bl-6c40"
on = "claim"

[[blockers]]
id = "bl-02bf"
on = "claim"
+++
Fable review of Z3 landing (ecf1a33) minors m1/m4/m5/m6 + doc staleness from Z6 — small, mechanical, sequenced after Z6/Z4/Z8 to avoid worktree conflicts. (1) m1: opslog clip bound is pre-JSON-escape while CAP=4096 is post-escape — clip against build_line output length (reuse largest_fit) or lower LOG_GOAL_CAP (src/start/exec.rs:37) to ~1024; strengthen prompt_clips_a_large_logged_goal to pin <=CAP post-serialization. (2) m5: Step::EnsureSeeded{lernie_home} field is computed by the planner but ignored by the executor which re-derives layout_under() — one fact two derivations; use the planned field or strip it. (3) m6: Env::home_dir falls back to PathBuf::default() (src/xdg/mod.rs:170-173) so unset HOME yields cwd "" on the bare rung — real fallback or loud abort (ops row). (4) m4: draft-survival ('clear draft only on clean send') and two enablement one-bits live in coverage-excluded shell glue (input_bar.rs:65-67,131-133; workspace.rs:84; start_pane.rs:82) — move to covered predicates in actions, or amend STORIES to scope draft-survival as glue-by-design (decide; lean: covered predicate). (5) DESIGN §15 M6 Z6 'Files' still names src/start/exec.rs as the precondition site; post-Z3 the prepare path is src/start/run.rs where require_tools landed — fix the file hint (one line). ALL tests green + full gate before close.