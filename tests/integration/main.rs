//! The consolidated integration-test binary: one process for every end-to-end
//! test that does **not** need a private one.
//!
//! Cargo compiles every file directly under `tests/` into its own binary, each
//! statically linking the whole dependency tree with debuginfo. At 28 files
//! that cost ~1.8 s of process-launch overhead per binary on every
//! `cargo tarpaulin` run and ~175 MB of duplicated linkage per binary in each
//! of the several worktrees this repo keeps live. The files below live one
//! directory down, so they are modules of this single root instead.
//!
//! Three tests stay standalone at `tests/` and must never move here — each
//! mutates process-global state (`std::env::set_var`, and in one case
//! `set_current_dir`), `unsafe` in edition 2024 and sound only when no peer
//! thread can observe the mutation. Their soundness argument *is* "one
//! `#[test]` per binary", which merging would silently void:
//! `multiplex_bl.rs`, `multiplex_lernie.rs`, `git_env_scrub.rs`.
//!
//! Merging changes one thing beyond the layout: ~25 tests now run
//! thread-parallel in ONE process, all of them forking, so a fixture script
//! written with a plain `fs::write` here can be exec'd while a peer thread's
//! fork still holds the write fd (ETXTBSY). Every executable fixture therefore
//! goes through [`support::write_executable`] — read its doc before adding a
//! test that writes one.

mod support;

mod boundary_prompt;
mod boundary_search;
mod boundary_verbs;
mod editor_roundtrip;
mod glyph_coverage;
mod pluggability;
mod reply_streams;
mod sigterm_durability;
mod silent_driver_death;
mod stories_inv1;
mod stories_inv3;
mod stories_s0_t1;
mod stories_s0_t2;
mod stories_s0_t3;
mod stories_s0_t5;
mod stories_s0_t6;
mod stories_s1_t1;
mod stories_s1_t2;
mod stories_s1_t3;
mod stories_s2_t1;
mod stories_s3_t1;
mod stories_s3_t2;
mod stories_s3_t3;
mod stories_s3_t4;
mod stories_s3_t5;
mod stories_s3_t6;
mod stories_s3_t7;
mod stories_s4_t1;
mod stories_s4_t2;
mod stories_s4_t3;
mod stories_s4_t4;
mod stories_s4_t5;
mod stories_s4_t6;
mod stories_s4_t7;
mod stories_s5_t3;
mod stories_s5_t4;
mod stories_s5_t5;
mod stories_s5_t6;
mod stories_s6_t1;
mod stories_s6_t2;
mod stories_s6_t3;
mod stories_s6_t4;
mod stories_s6_t5;
mod stories_s7_t1;
mod stories_s7_t2;
mod stories_s7_t3;
mod stories_s7_t4;
mod stories_s7_t5;
mod stories_s8_t1;
mod stories_s8_t2;
mod stories_s8_t3;
mod stories_s8_t4;
