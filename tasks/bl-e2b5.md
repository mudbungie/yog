+++
title = "Add build-gated release-plz release pipeline"
created = 1784698909
updated = 1784698909
claimant = "Junctions-yog"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
yog had CI only, and that CI did not even run on pull requests. This adds the same build-gated release-plz pipeline the other repos use — a release PR opened/updated on every push to main (queued, never auto-published), the release itself gated on the CI workflow concluding success on main, and a binaries job in the same run building the `yog` binary for x86_64-unknown-linux-gnu and attaching it to the GitHub Release. Version bumps default to patch (0.0.1 -> 0.0.2), with `[minor]`/`[major]` opt-in markers documented in release-plz.toml.

Three edits:
1. NEW .github/workflows/release-plz.yml
2. NEW release-plz.toml (repo root)
3. EDIT .github/workflows/ci.yml: rename `name: ci` -> `name: CI` (the release workflow gates on `workflow_run: workflows: ["CI"]`, an exact name match), and add a `pull_request:` trigger so PRs — notably the release PR — are gated before merge.