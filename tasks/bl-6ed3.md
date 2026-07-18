+++
title = "CI hygiene: lib.rs 99.90% coverage regression from module-decl folds blocks linux make ci; .githooks/pre-commit is non-executable so closes land ungated"
created = 1784350147
updated = 1784350147
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Discovered while delivering bl-592b. Two linked defects keep CI red and let regressions land unnoticed.

1) LINUX ci JOB RED — coverage 99.90% (< 100% floor). CI run 29620934183 (commit faccc40) linux job fails `make ci`. The single uncovered line is src/lib.rs:58 `impl AppState {`. from_args IS exercised (test app_state_from_args_copies_repo), so this is an llvm-cov/tarpaulin region-attribution artifact, NOT a testability gap. It was introduced purely by the Y2/Y7/Y15 folds adding three `pub mod binding; pub mod opslog; pub mod xdg;` declarations to lib.rs — no change to the impl block itself. Verified: at 39072d9 (pre-folds) lib.rs = 16/16, 100%; at fb73110 (Y15, pre-bl-592b) lib.rs = 15/16, 99.90%. bl-592b does not touch lib.rs. Reshaping from_args did not clear the artifact. Needs a robust fix that restores 100% (e.g. restructure so line 58 gets a covered region, or an appropriate tarpaulin config) delivered via a proper worktree — lib.rs is NOT touched by bl-592b or its sibling fs_watcher follow-up.

2) ROOT CAUSE of ungated slippage — .githooks/pre-commit is committed mode 100644 (non-executable); .githooks/post-commit is 100755. git skips a non-executable hook ("hook was ignored because it is not set as executable"), and bl close treats a repo with no executable pre-commit as UNGATED (delivers unchecked). That is why the Y2/Y7/Y15 coverage regression AND the bl-592b macOS near-miss both landed without the 300-cap/100%-coverage gate firing. Fix: `git update-index --chmod=+x .githooks/pre-commit` (chmod +x + commit) so bl close and local commits actually gate. This also explains why origin/main sat stale at ca2f3bb for several deliveries: the auto-push post-commit hook is fine, but nothing was catching the red state.

ACCEPTANCE: chmod +x on .githooks/pre-commit committed; lib.rs restored to 100% coverage; linux `make ci` green on origin/main.