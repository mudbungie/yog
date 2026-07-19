+++
title = "B5: panic-free prod — restriction lints + no-assert/no-suppression rules"
created = 1784433624
updated = 1784435095
parent = "bl-97fb"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-4bb4"
on = "claim"

[[blockers]]
id = "bl-ca57"
on = "claim"
+++
Per the assessment: 22 prod unwrap/expect + 17 guarded index sites, zero asserts, zero panic!/todo!/dbg!. Convert ALL without signature ripple: (1) the 6 Mutex .lock().unwrap() -> .lock().unwrap_or_else(std::sync::PoisonError::into_inner) (poison-immune); (2) locally-fallible fns already returning Result: .ok_or(...)? (cli_outbound stdout/stderr takes, apply/edit parent(), ui_state parent/file_name); (3) infallible serializes: unwrap_or_default() with a comment (opslog build_line, ui_state serialize); (4) balls.rs expects -> filter_map (a broken invariant drops the row instead of panicking — derive-from-disk semantics); (5) start_pane guarded unwraps -> if-let/let-else restructure; (6) ui_state descend double-unwrap -> entry-API rewrite; (7) fs_watcher iter.next() x2 -> slice-pattern match; (8) all 17 index/slice sites -> .get()/.get_mut() with graceful skip or slice patterns (each has a natural fallback; renamed[&p] HashMap index -> .get().copied().unwrap_or(false)-style). Then manifest: add to [lints.clippy] (workspace or crate Cargo.toml): unwrap_used, expect_used, panic, todo, unimplemented, dbg_macro, indexing_slicing, string_slice = deny. clippy.toml: allow-unwrap-in-tests etc. per the bootstrap doc (add allow-indexing-slicing-in-tests). NOTE tarpaulin interplay: ignore-panics=true excluded panic branches from coverage; removing panics changes the covered-line set — keep 100% (the gate proves it). Install rules/no-assert-outside-tests.yml + rules/no-lint-suppression.yml verbatim (the cfg(test) carve-outs must match the repo's inline-tests layout: tests live both in mod tests blocks AND sibling tests.rs files declared #[cfg(test)] mod tests; VERIFY the follows-based carve-out actually exempts the sibling-file pattern with the installed ast-grep — if it does not, extend ignores with the repo's test-file globs and document); extend fixtures; smoke-test. Audit: no #[allow] exists in prod today outside the one scoped cfg_attr in probe_cache (removed in B3) — verify and clean any stragglers. Gate green.