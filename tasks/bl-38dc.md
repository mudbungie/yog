+++
title = "ingest a litany release unattended: an hourly workflow moves the litany pin to the newest published litany and the brazen pin to the one that litany names, proves it through ci.yml on a branch, lands it on main, and the release follows"
created = 1794984548
updated = 1794984548
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Every litany/brazen pin move so far was a hand-filed ball (bl-ebef and its kin). Publishing is automatic and every endpoint reconciles from the registry; the pins are the one link a person carries, so an upstream release reaches no device until somebody files a ball.

Shape: `make ingest` (a script under scripts/, self-tested under fake curl/cargo shims like the deploy self-tests) reads the newest non-yanked litany off the sparse index and reads the brazen requirement off THAT SAME index line, so the two pins step together and the lockfile resolves one brazen; it rewrites both pin lines in Cargo.toml and moves the lockfile with `cargo update --precise`. Nothing newer is the same path with an empty diff. `.github/workflows/ingest.yml` runs it hourly and on dispatch and lands a non-empty diff through the gate exactly as advisories.yml lands a lockfile mend; the landing steps the two workflows share get ONE definition (a script both call), not two copies. A red gate is a hand's case and leaves main untouched.

Upstream half: litany's own ingest ball (brazen -> litany).