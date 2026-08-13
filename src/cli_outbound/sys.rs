//! The crate's single confined `unsafe`. [`super::Stream`]'s drop sends a
//! best-effort `SIGTERM` before escalating to `Child::kill` (SIGKILL);
//! `libc::kill` is an FFI call and therefore `unsafe`, and std exposes no
//! safe wrapper for a bare signal-to-pid. The `unsafe-outside-sys`
//! ast-grep rule (`rules/unsafe-outside-sys.yml`) bans `unsafe` everywhere
//! else in the tree, so this file is the one audited home for it — keeping
//! the whole-crate `unsafe` inventory at exactly one site.

/// Post `SIGTERM` to `pid`. Best-effort: a failed `kill` (e.g. ESRCH — the
/// child already exited and was reaped) is intentionally discarded, since
/// [`super::Stream`]'s drop follows up with a hard `Child::kill` + `wait`
/// regardless.
pub(super) fn sigterm(pid: i32) {
    // SAFETY: `libc::kill` is a thin FFI shim over the `kill(2)` syscall —
    // it dereferences no pointers and only posts a signal to `pid`. The
    // return value is ignored on purpose (the caller escalates to SIGKILL),
    // so there is no error path to mishandle.
    unsafe { libc::kill(pid, libc::SIGTERM) };
}
