+++
title = "the runner has no git identity, so multiplex_bl's close leg fails on every CI/speculate build"
created = 1786688515
updated = 1786763301
claimant = "Thimble"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Every GitHub Actions build of the gate fails one test — `tests/multiplex_bl.rs::the_bl_arm_runs_the_whole_rung_on_the_embedded_balls`, at the `bl close` assertion. The cause is environmental, not a defect in the code under test: `bl-delivery` runs `git commit --no-verify` in the test's throwaway project and git refuses with `empty ident name` / `Please tell me who you are`, because no workflow configures `user.name`/`user.email` and a runner image ships none.

Verified on a speculative build (2026-08-14): the whole lib suite passed, the failure is that one integration binary, and the same tree's local gate is green because a developer box has a global gitconfig. So the remote gate can never record a PASS verdict — every close pays the local build, which is the merge queue's whole point defeated.

Two candidate fixes, and the choice is a ruling: set the identity in the workflows (one `git config --global` step, cheap, but it makes the suite depend on ambient state the test cannot see), or have the test seed the identity in the repository it creates (`git -C <proj> config user.name/user.email`, hermetic, and the same discipline `git_env` already applies to `GIT_DIR`). The second looks right: a test that founds a repository owns that repository's config, and the harness should not need the box to be configured.

Found while delivering bl-1eb0, whose speculative build this reddened; unrelated to that ball's diff.

---

root cause is narrower than 'the runner has no identity': balls' delivery boundary (safegit::delivery_env) rebuilds bl-delivery's git env with GIT_CONFIG_NOSYSTEM=1 and GIT_CONFIG_GLOBAL=/dev/null, so a scratch global gitconfig can never reach the squash commit — only repository-local config and the six GIT_AUTHOR_*/GIT_COMMITTER_* values cross. Dev boxes passed on git's guess off the passwd name field (populated there, empty on a runner) or an exported GIT_AUTHOR_NAME. Fix as recommended: the founded project seeds user.name/user.email itself, the six identity vars are scrubbed so the seed is the only source, and the delivery assertion reads the author back. Class check: multiplex_lernie's repos are lernie's own (global config applies, green remotely), leak_store_gate already seeds repo-local config, test_support passes -c identity — one instance, not a class.
