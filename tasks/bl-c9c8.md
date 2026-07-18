+++
title = "macOS: projects enumerate test creates a non-UTF-8 dir name — APFS rejects invalid UTF-8, fixture unbuildable on macos-14"
created = 1784355345
updated = 1784355345
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Every CI run since aa97925 (Y14) fails the macos job on projects::tests::enumerate_decodes_flags_internal_and_skips_non_dirs_and_bad_names at src/projects/tests.rs:55:67 — the .unwrap() on fs::create_dir(clones.join(OsStr::from_bytes(&[0x66, 0x80]))). APFS enforces valid UTF-8 filenames and refuses the creation (EILSEQ); ext4 permits it, so linux passes. The production code's skip-non-UTF-8-basename branch is correct; the INPUT is simply unconstructible on macOS.

Fix: gate the non-UTF-8 fixture arm to platforms that can construct it — #[cfg(not(target_os = "macos"))] around the create_dir line, with the entry-count expectation derived (e.g. a  adjusting the expected len, or split the bad-name arm into its own linux-gated test and leave the main test platform-neutral — prefer the split: one platform-neutral test of decode/internal/skip-file/sort + one cfg(linux... actually cfg(not(macos)))-gated test of the non-UTF-8 skip). Coverage on linux (where tarpaulin runs) is unaffected either way. Verify no OTHER test in the tree constructs invalid-UTF-8 names (grep from_bytes / OsStr::from_bytes / OsString::from_vec across src/ and tests/).

Acceptance: macos-14 CI job green at the close commit (the previous fully-green run was 2600f23; runs since aa97925 are red on exactly this test — 560 passed / 1 failed at tip). Post-close, watch the CI run and report the macos conclusion.