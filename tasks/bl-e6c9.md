+++
title = "world::tools::ensure_shim writes an executable in-process, so a peer fork can ETXTBSY the exec that follows it"
created = 1788484551
updated = 1788484686
claimant = "Spellbind-U"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["flake", "world"]
+++
Sighted while verifying bl-98ce's fix: 80 full-suite lib runs at 4-way concurrency, `world::tools::tests::the_seeded_shim_runs_and_passes_argv_through_verbatim` failed twice at `src/world/tools/tests.rs:135` with

    called `Result::unwrap()` on an `Err` value: Os { code: 26, kind: ExecutableFileBusy, message: "Text file busy" }

`src/git_env.rs`'s module doc (bl-fd28) says the ETXTBSY window is closed on the write side: *"Every executable fixture in this crate is written by a CHILD"*, via `test_support::write_exec`, *"so the write fd never exists in this process and a peer fork, in ANY crate, locked or not, has nothing of ours to copy"*, and `rules/no-hand-chmod.yml` is said to make that structural.

That covers fixtures. It does not cover **production**: `world::tools::ensure_shim` (`src/world/tools.rs:216`) writes the shim itself —

    fs::write(&path, &want)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(SHIM_MODE))?;

— in this process, and the caller execs the shim immediately after. The write fd exists here, a fork on any other thread copies it, and an exec inside that window is ETXTBSY. The beat is the sighting, but the hazard is the engine's: yog composes a world's shims and then runs them, and yog forks from every thread.

Worth attacking as a subtraction rather than a bracket — the same relocation bl-fd28 chose for the fixtures: write the shim through the one child yog already has (`git_env`), or write-then-rename so the exec'd inode is never the written one (a rename target has no write fd anywhere). `no-hand-chmod.yml` evidently exempts this site; check whether the exemption is what let the hazard stand.