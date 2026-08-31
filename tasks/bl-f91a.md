+++
title = "EXIT the litany interim git pin: flip Cargo.toml back to a registry pin and delete deny.toml's allow-git entry when litany next publishes"
created = 1788151367
updated = 1788151367
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
bl-fd24 took the one §16.7 phase-2 interim exception: litany rides
`version = "=0.0.3"` plus an exact `git`/`rev` (the bl-ddaa injection-seam
amendment — `tools(workspace, agent)` and `RoutedCall::cwd`) because that
change is on litany's main and not yet on the registry. The exception's named
exit is this ball:

1. When litany's next version publishes to crates.io (its release flow: the
   release PR merge cuts and publishes), change `Cargo.toml`'s litany line to
   the plain exact registry pin of that version.
2. Delete the `allow-git` entry (and its interim paragraph) from `deny.toml`
   `[sources]`, restoring "no allow-git list".
3. Restore CLAUDE.md rule 6's "registry-only, with no exception in force"
   wording (it currently records the exception, deliberately).
4. `cargo update -p litany`, gate, land. `make publish` is unblocked by this
   close and not before.

Verify the published crate carries bl-ddaa (the seam signature
`fn tools(&self, workspace: &Path, agent: &str)`) before flipping — a publish
cut below that commit does not exit the exception.