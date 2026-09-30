+++
title = "scripts/pre-commit collapses to exec bl-gate (userconf); tests/leak_gate.rs asserts the new invariant; gate wording in AGENTS.md"
created = 1790735980
updated = 1790735981
claimant = "Junketing-collapse2"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
ops bl-3166 (phase 2 rollout). The gate body lives once in ~/userconf/bin/bl-gate. The seated .githooks/pre-commit (mainline refusal, execs scripts/pre-commit) stays; scripts/pre-commit becomes `exec bl-gate "$@"`. tests/leak_gate.rs test 6 asserted the hook's text ran make leak-scan before bl-speculate check; it now asserts the hook execs bl-gate and that bl-gate (on PATH) runs leak-scan before check. make check / scripts/check unchanged (bl-1b8d).