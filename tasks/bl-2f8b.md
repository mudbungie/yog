+++
title = "macOS-only: multiplex landing repair test fails with NotFound on the balls clone path — /var vs /private/var canonicalization is the suspect"
created = 1787546576
updated = 1788409678
claimant = "Spellbind-A"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["flake", "macos"]
+++
## The failure

`multiplex::landing::tests::the_repair_is_idempotent` fails on the macOS CI leg
(run 32672624965, on the release-v0.0.5 PR; 2670 passed, this 1 failed):

    panicked at src/multiplex/landing/tests/mod.rs:85:48:
    first: Custom { kind: NotFound, error: "git status
      (<tmp>/yog/world/state/balls/clones/%2F<percent-encoded tmp path>%2Fproj/config):
      No such file or directory" }

Linux is green on the same tree, and macOS is reported-but-not-gating
(`.github/workflows/macos.yml`), so nothing is blocked — but a leg that stays
red rots into being ignored, which is how the last two macOS breaks (bl-1015)
got to 14 tests deep before anyone looked.

## The suspect

The balls clone path is keyed on the **percent-encoded literal invocation
path**. On macOS, `$TMPDIR` lives under `/var/folders/…`, and `/var` is a
symlink to `/private/var` — so any step that canonicalizes (git itself does,
`std::fs::canonicalize` does) produces `/private/var/…` while a step that
percent-encodes the uncanonicalized path produces `%2Fvar%2F…`. Two spellings
of one directory ⇒ the store is founded under one key and looked up under the
other ⇒ NotFound. Linux has no such symlink, which is exactly the observed
split.

Check where the test (or the landing repair it drives) derives the clone key
versus where the fixture creates it; the fix is to canonicalize at ONE place
before encoding — or in the test fixture, to canonicalize the tmp dir before
handing it to anything, which is what several other tests in this tree already
do for the same reason. Verify against the tree; this body reasons from the
log, not from the code.

---

Premise checked against the run's own log and REFUTED; the real author is bl-419d, already fixed on main.

The ball redacted the path. The log (run 32672624965, job 100227026466) carries it whole:

    git status (/var/folders/<...>/T/.tmpBUgsCY/yog/world/state/balls/clones/%2Fvar%2Ffolders%2F<...>%2FT%2F.tmpBUgsCY%2Fproj/config): No such file or directory (os error 2)

Both halves are the SAME spelling — `/var/...` for the tempdir prefix and `%2Fvar%2F...` for the percent-encoded key. There is no `/private/var` anywhere in it, and a canonicalization split needs two spellings. Confirmed in the code: nothing canonicalizes on this path. `Edge::resolve` (balls edge.rs) stores `invocation_path` verbatim; `Xdg::clone_dir` (balls layout.rs) is `percent_encode(&invocation_path.to_string_lossy())` and nothing else; and the fixture computes `landing` ONCE in `World::new` and hands the same `Edge` to both `found()` and `converge()` — one expression, one value, no second derivation to disagree with.

What the error actually is: the `NotFound` came from the SPAWN, not from a read. `landing::run` calls `git_env::output`, whose error kind is preserved by `sited`; a non-zero git exit becomes `io::Error::other`, so a `NotFound` can only be `Command::spawn` failing ENOENT — the program or the cwd. The cwd cannot be it: `world.damage()` ran `git add -A` and `git commit` in that exact directory moments earlier, and `seed_landing` had just written files into it. So it was the PROGRAM.

Why the program went missing: bl-419d. The tree that produced this failure is f95996c1, and `src/multiplex/lernie/tests.rs` there performs TWO real returning `execvp`s of `/nonexistent/yog-successor` inside the lib test binary (`perform_maps_each_outcome_to_its_exit` and `a_failed_exec_leaves_sigpipe_ignored`). `git_env::command` gives every command seven env deltas, so std captures the environment into a `CStringArray`, points the process's own `environ` at it, and FREES that array when the exec returns — holding only the env READ lock, so peer threads are walking it by design. bl-419d measured 73,435 torn entries in 21.2M reads. A peer whose `PATH` read comes back torn or empty gets `execvp` ENOENT, which is exactly this message. bl-419d's own two sightings were the same module (`multiplex::landing::tests::the_repair_spends_no_scalar_config`) with the other face of the same corpse: `InvalidInput: nul byte found in provided data`. The landing tests are the densest git-forkers in the binary, so they are the likeliest victim, never the author.

Chronology: this run was created 2026-08-23T23:06:24Z; bl-419d's fix (223a7f40) was committed 2026-08-24T05:04:23Z — six hours later. Current tree: the lib suite reaches `git_env::exec` exactly once, with a NUL in argv, so std refuses above `do_exec` and no `execvp` is spent; the one real returning exec lives in `tests/exec_return.rs`, a single-`#[test]` binary. macOS CI has been green on this test since.

Why macOS and not Linux: allocator. glibc's `free` leaves the bytes readable, so the same race usually reads intact there; Darwin's reuses them.

Delivered: NO canonicalization was added. Adding one would encode a false explanation into the fixture and buy nothing — the failing path has one spelling and the defect is fixed. What the tree got instead is the diagnosis, in the two places a reader will be standing: `git_env::exec`'s doc now names the torn read's SECOND face (a `NotFound` that reads as a missing directory but is a missing program), and `landing::sited`'s doc — which exists precisely because a rare macOS `NotFound` was once undiagnosable (bl-1ce0) — now says the label locates the fork without promising the path is the fault, and points at the mechanism. Linux green: `cargo test --lib multiplex::landing` 12/12.
