+++
title = "make install replaces ~/.local/bin/yog non-atomically — drivers spawning yog mid-install get ENOENT"
created = 1785645801
updated = 1785645801
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator hit it live 2026-08-02: 'failed to spawn /home/u/.local/bin/yog: No such file or directory' during a lernie prompt, while a close's auto-install (scripts/bl-install-main → make install) was replacing the binary. Makefile:173 uses install(1), which unlinks the target then writes ~20MB — an ENOENT window plus a partial-binary window, reopened on every ball close. Fix: write to a temp name in the SAME directory then rename into place (mv -f), which is atomic — a concurrent spawn gets whole-old or whole-new, never ENOENT. Same treatment for any other spawn-target the install writes (the .desktop/.svg are not spawned; the binary is the one that matters — but check scripts/bl-install-main for its CARGO_TARGET_DIR override path too). This is the same rename-atomicity discipline DESIGN §10 already mandates for yog's own writes; extend it to the install pipeline.