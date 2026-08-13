//! DELIBERATE ast-grep fixture — NOT part of the crate and never compiled
//! (it lives under `rules/`, outside `src/`, and is named by no Cargo
//! target). Its only job is to be flagged by every rule in `rules/`.
//!
//! Smoke test, both directions (see the `rules-audit` Makefile target):
//!   - `ast-grep scan rules/fixtures` MUST exit non-zero, flagging every
//!     deliberate violation below:
//!       * unsafe-outside-sys.yml    → violations 1–2
//!       * locks-outside-state.yml   → violations 3–4
//!       * no-rc-refcell.yml         → violations 5–6
//!       * no-pub-borrow-return.yml  → violations 7–8
//!       * no-pub-generic-bounds.yml → violation 9
//!       * no-named-lifetimes.yml    → violation 10
//!       * no-assert-outside-tests.yml → violation 11
//!       * no-lint-suppression.yml     → violation 12
//!   - `ast-grep scan src` MUST exit zero (the sanctioned `unsafe` lives in
//!     src/cli_outbound/sys.rs and the sanctioned locks in src/state.rs and
//!     src/git_tree/probe_cache.rs, all of which the rules ignore; no
//!     `Rc`/`RefCell` survives anywhere).
//! If any violation ever stops being flagged, that rule has silently regressed.

// Violation 1: an `unsafe` block outside sys.rs (matches kind `unsafe_block`).
fn uses_unsafe_block() {
    unsafe { libc::_exit(0) };
}

// Violation 2: an `unsafe fn` outside sys.rs (matches `function_modifiers`).
unsafe fn declared_unsafe() {}

// Violation 3: a `Mutex` outside state.rs (locks-outside-state.yml — the
// `type_identifier` in type position and the `identifier` at `::new`).
fn uses_a_mutex() {
    let _m: std::sync::Mutex<u32> = std::sync::Mutex::new(0);
}

// Violation 4: an `RwLock` outside state.rs (locks-outside-state.yml).
static SHARED: std::sync::RwLock<u32> = std::sync::RwLock::new(0);

// Violation 5: an `Rc` — banned everywhere, no test carve-out (no-rc-refcell.yml).
fn uses_rc() {
    let _r: std::rc::Rc<u32> = std::rc::Rc::new(0);
}

// Violation 6: a `RefCell` — banned everywhere (no-rc-refcell.yml).
struct HoldsRefCell {
    inner: std::cell::RefCell<u32>,
}

// Violation 7: a `pub fn` returning a borrow (no-pub-borrow-return.yml — the
// `reference_type` in `return_type`; the elided lifetime is the hidden
// coupling this bans).
pub fn borrow_return(s: &str) -> &str {
    s
}

// Violation 8: a `pub fn` returning an opaque `impl Trait` (no-pub-borrow-return.yml
// — the `abstract_type` in `return_type`).
pub fn opaque_return() -> impl Iterator<Item = u32> {
    std::iter::empty()
}

// Violation 9: a `pub` item carrying a generic bound (no-pub-generic-bounds.yml —
// the `type_parameter` with a `trait_bounds` child). An UNbounded `<T>` would
// be clear; the `: Ord` bound is what fires.
pub struct PubBound<T: Ord> {
    pub first: T,
}

// Violation 10: a named lifetime (no-named-lifetimes.yml — the `lifetime` node
// `'a`, which the rule's `not`/`regex` excludes only for `'static`/`'_`). The
// discipline: borrow on the way in (elided), hand back owned — so no signature
// ever names a lifetime.
pub struct Held<'a> {
    r: &'a str,
}

// Violation 11: an `assert!` outside any test (no-assert-outside-tests.yml —
// the `macro_invocation` whose `macro` field is `assert`, not inside a
// `#[cfg(test)]` mod). Prod states invariants in the type system or handles the
// failure; the assert vocabulary belongs to tests.
fn asserts_in_prod(x: u32) {
    assert!(x > 0, "prod should never assert");
}

// Violation 12: a lint suppression outside tests (no-lint-suppression.yml — the
// `attribute_item` matching `allow(`). Policy lives in Cargo.toml `[lints]`,
// paired with a justification; prod code carries no inline `#[allow]`.
#[allow(clippy::needless_return)]
fn suppresses_a_lint() -> u32 {
    return 0;
}

// Violation 13: a bare `Command::new` (no-bare-command.yml — the
// `call_expression` whose `function` text ends in `Command::new`, outside
// `src/git_env.rs`). Every child yog spawns must be built by
// `crate::git_env::command`, which strips the ambient `GIT_DIR` and friends
// from the whole descendant process tree; a hand-rolled spawn hands a hook's
// repo to whatever the child forks (bl-916a).
fn bare_child() -> std::io::Result<std::process::Child> {
    std::process::Command::new("lernie").spawn()
}
