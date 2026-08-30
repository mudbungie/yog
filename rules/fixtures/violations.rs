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
//!       * no-bare-command.yml         → violation 13
//!       * no-hand-rolled-paint-walk.yml → violation 14
//!       * no-bare-fork.yml            → violation 15
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
    std::process::Command::new("litany").spawn()
}

// Violation 14: a hand-rolled paint walk (no-hand-rolled-paint-walk.yml — the
// `call_expression` whose function is a `field_expression` reading `text` off a
// `galley`, outside `src/paint_probe.rs`). `Galley::text()` is the string that
// went IN, so an assertion on it is blind to the elision the paint layer is the
// only witness for; 1815 tests once passed while covering no truncation at all
// (bl-bc06), and two later balls each found another private copy (bl-36c3,
// bl-70b8). Painted glyphs come from `crate::paint_probe`.
fn reads_the_input_text(t: &egui::epaint::TextShape) -> bool {
    t.galley.text() == "Login"
}

// Violation 15: a bare fork (no-bare-fork.yml — the `call_expression` whose
// function is a `field_expression` ending in `.spawn`/`.output`/`.status` and
// whose arguments are empty, outside `src/git_env.rs`). Building the child in
// `git_env` and then forking it by hand leaves the ETXTBSY window open: a fork
// on any thread copies a peer's open write fd into a child that holds it until
// its own exec, and the peer's exec of the script it just wrote fails while it
// does (bl-6397). `crate::git_env::{spawn, output, status}` is the one fork.
fn bare_fork(cmd: &mut std::process::Command) -> std::io::Result<std::process::Output> {
    cmd.output()
}

// Violation 16: a bare exec (no-bare-fork.yml — the same shape, `.exec` on the
// end). `CommandExt::exec` does not fork: it resets `SIGPIPE` to `SIG_DFL` in
// THIS process on its way to `execvp`, and an `execvp` that fails returns into
// a process that now dies where it used to error (bl-3792).
// `crate::git_env::exec` is the one exec, and it puts the disposition back.
fn bare_exec(cmd: &mut std::process::Command) -> std::io::Error {
    use std::os::unix::process::CommandExt as _;
    cmd.exec()
}
