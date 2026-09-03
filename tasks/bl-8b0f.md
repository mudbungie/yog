+++
title = "DUPLICATE of bl-3792 — evidence folded there, do not claim"
created = 1787544797
updated = 1788407672
claimant = "Spellbind"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["duplicate"]
+++
## The symptom

`cargo test --lib` dies partway through with no failing test and no panic:

    error: test failed, to rerun pass `--lib`
    Caused by:
      process didn't exit successfully: `target/debug/deps/yog-<hash>`
        (signal: 13, SIGPIPE: write on a pipe with no one to read)

The harness prints no `test result:` line at all — the binary is killed, so the
run has no verdict. Rate measured on one box: **2 of 3 runs**, and 2 of 4 on a
full `cargo test`. It is load-sensitive, not deterministic.

## Why it should be impossible

Rust's runtime sets `SIGPIPE` to `SIG_IGN` before `main`, which is why every
write to a dead peer in this tree surfaces as an `io::ErrorKind::BrokenPipe`
rather than a signal. `src/bz_host.rs` states that reliance verbatim: *"yog
never resets SIGPIPE — the disposition is `SIG_IGN`, so a closed stdout
surfaces as a `BrokenPipe` write error that brazen's own `ExitClass::from_io`
maps to the same 141, and restoring it would need an `unsafe libc::signal`
outside the one sanctioned site"*. Nothing in `src/` calls
`libc::signal(SIGPIPE, …)`; `rules/unsafe-outside-sys.yml` guarantees the whole
crate's `unsafe` inventory is one file, and that file does not touch SIGPIPE.
So **something restores the default disposition and it is not yog's own source
saying so** — the first thing to find. Candidates, in order of suspicion: an
in-process embedded-substrate arm (brazen's `bz` bin *does* restore SIGPIPE in
its own `main`, arch §5.8 — the question is whether any path yog links reaches
that), a linked C dependency's initializer, or a test that forks without
exec'ing.

## Where it dies

Both captured runs died in the `wire::*` block — the socket and
spawn-heavy region — with 2427 and 2504 of ~2622 tests finished:

- run A ended after `wire::server::tests::an_unbindable_address_refuses`,
  `wire::seat::tests::*`
- run B ended after `wire::host::exec::tests::a_tools_own_failure_is_its_verdict`

A `write` to a closed TCP socket raises SIGPIPE on the same terms a closed pipe
does, so the wire's own sockets are as good a candidate as any child's stdin.
Test names locate the region, not the culprit: libtest prints on *completion*,
so the test that died prints nothing.

## What it is NOT

Not bl-269a's SIGTERM work. Measured with `--skip engine::serve --skip
engine::stop --skip world::tools::tests::seed`, so no line of that ball's
production code executes: **still 2 failures in 3 runs.** The disposition
`term_disposition` touches is SIGTERM, and only inside a test that restores it.

## Why the gate does not catch it

`make check` runs `scripts/check-coverage.sh` (tarpaulin, `--test-threads=1`)
and never a parallel `cargo test`, so a close is unaffected. `make test` is
where it bites, which is every agent running the suite by hand — and it costs a
whole run each time, with a message that names no test.

Note the shape of the hazard for `check-coverage.sh` if it ever did surface
there: a signalled run is exactly the class bl-673a gave exit 75 to, and a
SIGPIPE-killed binary must never be recorded as a FAIL verdict.

---

Superseded by **bl-3792**, filed a day earlier and claimed. The distinguishing evidence in this body — the brazen `bz` SIGPIPE-restore suspect, the `wire::*` death region, and the --skip measurement proving independence from bl-269a — has been folded into bl-3792 as a comment. Work happens there. Left standing rather than closed only to avoid a gate run against a live queue.
