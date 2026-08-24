//! The crate's confined `unsafe` — **three raw process effects**, all of them
//! things std gives no safe wrapper for and none of them reducible.
//! [`sigterm`] is the best-effort `SIGTERM` [`super::Stream`]'s drop sends
//! before escalating to `Child::kill` (SIGKILL); [`set_env`] is the process
//! environment mutation the nested world needs to become a *place* and not only
//! a value ([`crate::world::inhabit`], DESIGN §16.2, bl-81c9); and
//! [`term_disposition`] is the `SIGTERM` *catch* an engine needs before either
//! of those can happen at all (§8.5, bl-269a) — under the default disposition
//! the process dies where it stands, so no `Drop` runs and the `sigterm` above
//! is never posted. The
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

/// Process-wide "a stop was asked for" flag. Set by [`on_term`] and read
/// through [`term_flag`]; the *only* thing a SIGTERM does to this process once
/// [`term_disposition`] has pointed the signal here.
static TERM_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The SIGTERM handler: one atomic store, which POSIX lists as
/// async-signal-safe. Everything a stop *means* (§8.5) happens outside it, in
/// the face's own loop — a handler that dropped the engine would run arbitrary
/// code, take locks and join threads on a signal stack.
///
/// `pub(crate)` so the §8.5 stop's one process-wide test drives it **as a
/// function** rather than by signalling the test binary: this suite runs under
/// tarpaulin's ptrace, whose signal bookkeeping is exactly what `tarpaulin.toml`
/// already pins the run serial to work around. What a delivered SIGTERM would
/// prove beyond this call is `signal(2)`'s own contract, not yog's.
pub(crate) extern "C" fn on_term(_signo: libc::c_int) {
    TERM_REQUESTED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Point `SIGTERM` at [`on_term`] (`catch`) or back at the kernel's default
/// terminate disposition (`!catch`). The third raw process effect, and the one
/// that makes the other two reachable at all: the default disposition kills the
/// process outright, so no `Drop` runs and [`super::Stream`]'s SIGTERM above is
/// never posted.
///
/// One function with a boolean rather than an install/restore pair, because
/// `signal(2)` is one call either way and is idempotent — the second install of
/// the same handler is a no-op, so there is nothing for a `OnceLock` to guard.
/// The restoring direction is what keeps the *test* that installs the real
/// handler hermetic: it hands the disposition back rather than leaving the rest
/// of a suite unable to die on a signal.
pub(crate) fn term_disposition(catch: bool) {
    let handler = if catch {
        on_term as *const () as libc::sighandler_t
    } else {
        libc::SIG_DFL
    };
    // SAFETY: `libc::signal` is the documented POSIX handler-install call and
    // dereferences nothing; `on_term` is a plain `extern "C"` function whose
    // body is a single atomic store, so nothing unsound can run on the signal
    // stack. The same construction the linked lernie already uses for its own
    // §2.9 stop catch.
    unsafe {
        libc::signal(libc::SIGTERM, handler);
    }
}

/// The flag [`on_term`] sets — `pub(crate)` and a borrow on purpose: it is one
/// process-wide fact and every reader must read *that* one, never a copy.
pub(crate) fn term_flag() -> &'static std::sync::atomic::AtomicBool {
    &TERM_REQUESTED
}
