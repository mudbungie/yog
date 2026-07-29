+++
title = "test git fixtures are not hermetic: the machine's global core.hooksPath reaches into them — 66 tests fail under the new identity hook"
created = 1785287220
updated = 1785287220
claimant = "scorched"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found 2026-07-28 by the bl-d82f investigation, reproduced on clean main: the machine-wide identity hook (userconf githooks-global, reached via global core.hooksPath) fires inside yog's throwaway test repos, refusing their 't@t.local' commits — 66 failures, and every bl close gate on this repo bounces until fixed.

The defect is yog's: the suite depends on machine git config. Fixtures already set repo-local identity (fixture.rs: user.email/user.name/gpgsign) — they are one env-scrub short of hermetic. Fix at the single fork site (src/git_tree/tests/git.rs run_git): GIT_CONFIG_GLOBAL=/dev/null, GIT_CONFIG_SYSTEM=/dev/null. Audit for any other test-side real-git spawn that commits.

Acceptance: cargo test green on this machine with the hook active.