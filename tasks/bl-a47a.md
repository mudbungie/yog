+++
title = "B6: pedantic gate — clippy pedantic=deny + sanctioned allow-list + fix wave"
created = 1784433624
updated = 1784433624
parent = "bl-97fb"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-d3f6"
on = "claim"
+++
Per the empirical histogram (297 warnings): (1) Cargo.toml [lints.clippy]: pedantic = { level = "deny", priority = -1 } plus the sanctioned allow-list — the bootstrap's five (needless_pass_by_value, module_name_repetitions, missing_errors_doc, missing_panics_doc, must_use_candidate) PLUS the four empirically-warranted for this GUI codebase, each with a one-line justification comment: doc_markdown (43 hits, dominated by proper nouns yog/bl/egui), cast_possible_truncation + cast_sign_loss + cast_possible_wrap (~9, egui f64->u8 color and index math), implicit_hasher (6, would force BuildHasher generics through internal APIs — also collides with rule 9), similar_names (7, subjective). [lints.rust]: warnings = "deny" (the toolchain is now pinned so this is deliberate-event-only). (2) Fix the remaining ~107 mechanical warnings: cargo clippy --fix -W clippy::pedantic where safe, hand-fix the rest (~15-20 judgment sites: manual_let_else, items_after_statements, struct_excessive_bools — judge each: fix or, if genuinely wrong for the code, add to the manifest allow-list with justification; NEVER inline #[allow]). (3) The existing gate (make lint/check + hook + CI) picks the manifest lints up automatically — verify `cargo clippy --all-targets` is what runs everywhere so the workspace lints apply (it is; confirm no stray -W flags fight the manifest). 300-cap pressure from --fix churn: split files as needed. Gate green + full CI green.