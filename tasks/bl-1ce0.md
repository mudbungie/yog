+++
title = "the landing repair's idempotence test fails intermittently on macOS: converge's first pass returns NotFound"
created = 1786764175
updated = 1786764175
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Seen once on a macOS CI job, not reproduced since (the next macOS job on the
same surface ran it green), and never on Linux.

The failure: multiplex::landing::tests::the_repair_is_idempotent panicked on
`converge(&world.edge, &world.root).expect("first")` with
`Os { code: 2, kind: NotFound, message: "No such file or directory" }` — the
FIRST convergence pass, not the second, so the test's own subject (idempotence)
was never reached.

Why it is odd. Two sibling tests in the same file — a_tracker_less_landing_
regains_the_whole_schedule and the_repair_spends_no_scalar_config — build the
same scratch world, damage it the same way and run the same first pass, and
both passed in the same job. Nothing structural distinguishes the failing one.
So this is a race or an environment-dependent path, not a defect in the
assertion.

Where a NotFound can come from inside converge, in order:
  - Hooks::load(landing)? — the schedule read, after is_landing already said
    the landing is there.
  - seed::seed_landing — balls' own re-derive, which reads the sibling roster.
  - commit() — three `git` forks through git_env::git().current_dir(landing).
    A spawn that cannot find its program is exactly ENOENT, and the landing
    tests are the one family in this binary that forks without holding
    test_support::spawn_guard(), whose whole job is to keep a write-then-spawn
    pair from overlapping a peer thread's exec. That is a hypothesis, not a
    diagnosis: the program here is `git`, not a script the suite just wrote.

What would settle it: run the landing tests under repetition on a macOS runner
with the panic carrying its site (the io::Error is bare — converge's `?` sites
do not say which read failed), or give the three git forks and the seed their
own error context so the next occurrence names itself. The cheap first move is
the context, since the failure is rare enough that adding it costs nothing and
a second sighting then diagnoses itself.

Filed out of bl-8626, which fixed the macOS wire-mint class (LibreSSL has no
`x509 -copy_extensions`) and the non-UTF-8 path test (APFS refuses the name)
and left this one, being a different defect class with no reproduction.