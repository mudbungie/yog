+++
title = "Parallel cargo test flakes with ETXTBSY: forks outside SPAWN_LOCK inherit recorder-script write fds"
created = 1784336821
updated = 1784349048
claimant = "filtered"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
`cargo test` (parallel) intermittently fails `actions::tests::dispatch_stop_emits_subcommand_repo_branch_argv` with Spawn/ETXTBSY ("Text file busy"): the test holds `test_support::SPAWN_LOCK` while writing+exec'ing a recorder script, but git_tree fixtures fork `git` WITHOUT the lock, and a concurrent fork can inherit the script's not-yet-closed write fd until its own exec. Pre-existing before the repo split (hidden by tarpaulin's serial run).

Interim: `make test` and tarpaulin.toml pin `--test-threads=1`. Real fix: one spawn discipline for every fork site (extend SPAWN_LOCK to the git_tree fixtures, or open-write-close-then-exec with O_CLOEXEC hygiene), then lift the serial pins.