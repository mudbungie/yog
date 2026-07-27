+++
title = "CI red on macOS: tests/multiplex_bl.rs worktree-materialization assert fails (linux green)"
created = 1785132694
updated = 1785132917
claimant = "waxier-34ae"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found while landing bl-89a4 (registry adoption); NOT caused by it — reproduced on unmodified main (0d914fc) and present in CI runs for a3bce10, 0d914fc and cc4ef38.

STATUS (waxier-34ae): fixed in tests/multiplex_bl.rs.

Root cause, confirmed from the macOS log for run 30240521334 (cc4ef38):

    panicked at tests/multiplex_bl.rs:213:5:
    worktree materialized at /var/folders/g3/.../T/.tmpnZy6c3/state/balls/plugins/bl-delivery/var/folders/g3/.../T/.tmpnZy6c3/proj/bl-c69e

The mirrored project component is spelled `var/folders/...`. balls keys its
store — and bl-delivery mirrors its territory (balls arch §11, mirrored by
`src/binding::under`) — on the directory the process runs in, read from the
kernel via `getcwd`, which returns the *resolved* path. On macOS the tempdir's
`/var/folders/…` is a symlink to `/private/var/folders/…`, so bl-delivery
mirrored `private/var/…` while the test derived `var/…` from `tmp.path()`.
The claim succeeded (exit 0); only the assertion looked in the wrong place.
Linux has no such symlink, hence green. The territory-root half of the path
was never the problem: reached through the `/var` symlink it resolves to the
same directory.

Fix (house pattern — derive the expected path from the same source the code
under test uses, no cfg(target_os), no prod change): after
`std::env::set_current_dir(&proj)`, rebind `let proj = std::env::current_dir()`
so every later derivation uses the kernel's resolved spelling. No-op on Linux.
Precedent: bl-592b (`src/fs_watcher/tests.rs::workspace`), `src/watch/tests.rs`,
`src/cli_outbound/tests/*` canonicalize-both-sides.

Local: `cargo test --test multiplex_bl` green.