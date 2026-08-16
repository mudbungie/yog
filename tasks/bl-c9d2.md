+++
title = "a workspace whose birth died mid-way can never be prepared again: the name is wedged and the refusal names the wrong reason"
created = 1786845530
updated = 1786845530
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "boundary"]
+++
Found beside bl-6c9e, which fixed the *success* half of the same guard and left this one standing.

`resolve_workspace` (src/boundary/dispatch/resolve.rs) refuses any `Prepare` whose raised path already exists on disk but is not in the caller's enumeration. Since bl-6c9e that guard is about scope alone for a workspace that was born — but a directory under yog's flat names root holding no `repo.git` is enumerated by no root at all, so it fails the same test forever.

That state is reachable and not exotic. `execute_ensure_workspace` skips its create only on `<workspace>/repo.git`, and `lernie new` makes the directory before it makes the marker: a `new` that dies part-way (a killed process, a full disk, a substrate refusal) leaves a directory with no marker. From then on every `/prepare` naming that workspace answers `unknown workspace "<name>"` — a sentence about addressing, for a condition that is a half-written filesystem — and the name is unusable, with no verb that says why and none that clears it. The only move is a filesystem one, out of band, which is what the §3.6 unmaking exists to make unnecessary.

Two things to weigh, and they may be one fix:

1. The refusal conflates "a name another client holds" (the scope case the guard exists for, absence by ruling) with "a directory here is not a workspace" (a local, repairable state). The first must keep leaking nothing; the second is the caller's own territory and can be named.
2. Whether a `Prepare` may adopt a marker-less directory under the names root at all. `lernie new` into an existing empty directory is the natural resume, and `create_workspace`'s own skip already treats resume as the general path one level down — so the fix may be that the raise resolves to the path and lets the idempotent ensure step decide, rather than a second refusal standing above it.

Not urgent: it takes a failed birth to reach. It is a wedge with no in-band exit once reached.