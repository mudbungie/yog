//! The crate's confined `unsafe` — **two raw process effects**, both of them
//! things std gives no safe wrapper for and neither of them reducible.
//! [`sigterm`] is the best-effort `SIGTERM` [`super::Stream`]'s drop sends
//! before escalating to `Child::kill` (SIGKILL); [`set_env`] is the process
//! environment mutation the nested world needs to become a *place* and not only
//! a value ([`crate::world::inhabit`], DESIGN §16.2, bl-81c9). The
//! `unsafe-outside-sys` ast-grep rule (`rules/unsafe-outside-sys.yml`) bans
//! `unsafe` everywhere else in the tree, so this file is the one audited home
//! for it — keeping the whole-crate `unsafe` inventory at exactly one site,
//! which is why the env fold lives here rather than beside its caller.

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

/// Set every `(var, value)` pair in **this process's** environment. The one env
/// mutation in the tree, and the only way the world can reach an in-process
/// substrate arm (§16.7): the linked `balls`/`lernie` read `getenv` themselves
/// and spawn children that do the same, so there is no `Env` to inject — the
/// process has to stand in the world. Its production callers are the `world`
/// module's two place-folds and nothing else: [`crate::world::inhabit`] (the
/// §16.2 override set) and [`crate::world::inhabit_space`] (the §16.3 space,
/// layered on it one var deep — bl-c21d).
pub(crate) fn set_env(pairs: &[(String, String)]) {
    for (key, value) in pairs {
        // SAFETY: `setenv(3)` is not thread-safe — a concurrent `getenv` in
        // another thread may read a table this call frees — and that is the
        // whole of why std marks it unsafe. Every caller stands at the process
        // edge, single-threaded: `world::inhabit` is reached only from
        // `multiplex::dispatch`, which `main.rs` calls above clap, above the
        // hatches and above eframe, before this process has spawned a thread.
        // The two integration binaries that drive `dispatch` in-process each
        // run exactly one `#[test]` for this reason, and the src-side unit
        // tests reach no folding arm (their argv is a discovery probe or a
        // parse error, both of which answer before the fold).
        unsafe { std::env::set_var(key, value) };
    }
}
