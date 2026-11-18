+++
title = "Container server cannot see a sibling tool's sign-in: an ambient-credential row reads 'not signed in' under the container unit"
created = 1794985432
updated = 1794985432
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug"]
+++
## Finding

A provider row whose only credential source is an `ambient` block naming a file under `~/` (the shape: an oauth2 row borrowing another CLI's own credential file) reports credential `missing` when the server runs under `scripts/deploy/yog.service`, while the same row on a binary install reports `ambient`.

Cause, each step observed on a container deployment:

- The image runs with `HOME=/` and `XDG_DATA_HOME=/state`; the unit mounts only the data root and the work dir. brazen expands the ambient path against `$HOME` (brazen `src/native/creds.rs` `expand_home`), so the path resolves to a directory at the container root that does not exist and that the run user cannot create.
- The sibling tool's credential file exists in the operator's home on the host and is fresh; nothing carries it into the container.
- No seat-driven sign-in had been run for the row (the sign-in tail answers the empty frame), and the workspace's own credential store holds only the other oauth row.

So signing in with the sibling tool on the host is invisible to the server by construction, and nothing tells the operator so: the row reads 'not signed in' with no reason.

## Not decided here (operator's call)

1. Mount the sibling tool's credential DIRECTORY read-only into the container at `$HOME`-relative position (a directory, not the file: a file bind pins an inode and an atomic rewrite by the owning tool would leave the container reading the old one). Cost: it exposes that whole directory to the server's agents, and a borrowed credential is read-only to brazen, so it goes stale unless the owning tool keeps refreshing it on the host.
2. Own the credential instead: `bz --login` for the row via the seat. The row has no device block, so the run is the loopback flow on the server's loopback at an ephemeral port — reachable only through a port-forward, and whether the vendor accepts brazen's default redirect host for that client is unverified.

## Second defect waiting behind the first

The same row's `user-agent` pin on the container deployment is an old CLI version with no `header_commands` refresh, and the image carries no sibling CLI to ask. The backend has refused stale versions before, so the row may 400 on the first request even once a credential resolves.