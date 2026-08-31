+++
title = "the world's agent-tool shims assume a filesystem that executes what yog writes"
created = 1788139089
updated = 1788139089
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Filed from the android client (yog-android bl-d6c6), which walked the chain for
running this engine on a phone. Two rungs stop it; one of them is yog's own
shape and is worth stating here even though nobody is asking for it yet.

**What is true.** The crate cross-compiles and links for `aarch64-linux-android`
against the pinned NDK with `balls`, `brazen`, `litany`, `ureq`, `rustls` and
`ring` in the graph, no C toolchain acquired — `ring` was already the provider
on both ends. The release link is a ~13 MB PIE executable for that platform's
loader. The Rust half is not the obstacle.

**The shape that is.** `src/world/tools.rs` seeds `<world>/tools/{bl,litany,bz,…}`
— `/bin/sh` re-execs of yog under a namespace — so an agent's bash resolves `bl`
to the embedded balls rather than to whatever is on the ambient `PATH`. They are
files yog **writes at runtime** and something later **executes**.

That is a filesystem assumption, and it is not universal. On Android, an app's
private storage is not executable by platform policy (since API 29): a file the
process just wrote cannot be exec'd at all, and the one exception is the app's
own native library directory, whose contents are placed there at install time
and cannot be generated. So the shim mechanism has no landing site on that
platform — not because of a permission an operator could grant, but because the
mechanism is *write a file, then run it*.

**What would dissolve it, stated as a shape rather than a feature request.** The
shim exists because the agent's `bash` needs a PATH entry, and the PATH entry
must name a file. The fact it carries — *"`bl` means this yog under the `bl`
namespace"* — is already held in `cli_outbound::resolve`, where yog's own spawns
resolve to `current_exe()` with a leading word and never consult the shim. So
there are two homes for one fact today, and only one of them needs a writable
executable file. Whether that is worth collapsing is yog's call; this ball is
the constraint, not the design.

**No ask is attached.** The android app offers the server bootstrap and starts
nothing, stating both blockers in the operator's own terms. Nothing is blocked
on this ball, and the other blocker — Android ships no `git`, and the world,
every workspace and the task store are all git — is not yog's to fix.