+++
title = "lernie composes a summary/** carrying literal conflict markers straight into context: the write path refuses them in 0.0.3, the read path still trusts the bytes"
created = 1785460447
updated = 1785644891
claimant = "Riffle"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["investigation"]
+++
## Origin

The fourth, explicitly-optional sub-claim of bl-ebbd ("Suggested fix direction" #4):

> As a cheap mitigation independent of the above: never compose a `summary/**` file
> into context if it still contains literal `<<<<<<<`/`=======`/`>>>>>>>` lines —
> treat that as a hard assembly-time error rather than trusting the file's contents
> blindly.

bl-ebbd's other three sub-claims are delivered in lernie 0.0.3 and it closed against
them. This one is **not** delivered, verified by reading the pinned source.

## Verified absent in lernie 0.0.3

`~/.cargo/registry/src/index.crates.io-*/lernie-0.0.3/src/prompt/dispatch/assembler/body.rs`
is the whole of body composition. Its pipeline is `compose` → `walk` → `select` →
`fit` → `render`. `select` reads each matched file and stores its bytes verbatim:

    let bytes = std::fs::read(worktree.join(file)).map_err(Error::Io)?;
    out.push(Entry { path: file.clone(), content: String::from_utf8_lossy(&bytes).into_owned() });

There is no content predicate anywhere in the module — the only rejection is an I/O
error, and even non-UTF-8 is composed lossily by deliberate design ("declining a
skill's stray binary asset would hold the whole branch hostage to it"). A repo-wide
grep for `<<<<<<<` outside tests finds it only in the compaction-merge modules'
prose and in `decline::content_conflicts`, which reads git index stages — never at
assembly.

## Why it still matters after the upstream fix

lernie 0.0.3 closes the *write* path: `src/prompt/compactor/merge.rs` asks
`content_conflicts` (from `git ls-files -u`, stages 2 **and** 3 populated) *before*
`git add -A`, and on any hit calls `decline` — `merge --abort`, mark
`refs/lernie/conflicted/<compactor-id>`, land nothing. So the harness can no longer
*create* a marker-bearing `summary/**`.

It does not close the *read* path. Any tree that already carries markers — written
by lernie 0.0.2, by an operator's hand-edit, or by a `write_summary` payload that
merely contains the literal strings — is still composed into every subsequent model
call verbatim. The guard is defence-in-depth on a corrupted-context failure that
lernie ARCH §2.7 promises "can never happen": worth having precisely because the
promise is now load-bearing.

Concrete instance that existed: `agents/<agent-id>:summary/001.md` in
workspace `<workspace>` carried nine marker lines (three nested unresolved 3-way
conflicts). That commit (`1596809d26fa89e2f01cb2c2c22cb4501ff62209`) is now
unreachable from any ref, but it was live at the root branch tip for days.

## Venue

lernie, not yog. Same class as the fixes in lernie `[bl-a9eb]`. yog consumes the
behaviour through the pinned crate and has no assembly path of its own.

## Ask

File upstream against lernie: make `assembler::body::select` (or `compose`) refuse a
composed entry whose content carries a line beginning `<<<<<<< `, `=======` or
`>>>>>>> `, as a hard `Error` naming the path — the same "refuse loudly" discipline
`merge::decline` already applies on the write side. Decide upstream whether the
refusal covers every composed file or only `summary/**`; bl-ebbd asked only for
`summary/**`, but the reasoning (context that is not what it claims to be) is not
specific to summaries.

## Resolution (2026-08-01)

Filed upstream as **lernie bl-c867**, carrying the verified-absent evidence, the write-path/read-path distinction, and the open scope question (summary/** only vs every composed file) as a design point to decide there. yog has no assembly path of its own; nothing further to do in this repo.