+++
title = "Upstream delivery wiring: auto-push main on delivery, CI (linux + macos-arm64), crates.io publish wiring (no publish)"
created = 1784349057
updated = 1784349057
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-fed1"
on = "claim"
+++
Three deliverables, one coherent delivery-infrastructure change: (1) .githooks post-commit hook: when a commit lands on main (bl delivery), push main to origin — automatic delivery to github.com/mudbungie/yog; document in README. (2) GitHub Actions: linux job runs make ci (fmt-check, clippy -D warnings, tarpaulin 100% floor with pinned 0.35.2), macos-14/arm64 job runs build + test (no tarpaulin); trigger on push to main. (3) crates.io publish wiring for the yog crate at 0.0.1 (name reserved via 0.0.0 placeholder): complete package metadata (description, license, repository, keywords, categories), drop publish=false, add a 'make publish' target that runs cargo publish --dry-run then requires an explicit CONFIRM=yes to run cargo publish. DO NOT PUBLISH — publication at 0.0.1 is an active human decision.