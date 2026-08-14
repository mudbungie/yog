+++
title = "I3/§5.2 promise a 24 h sweep of `.yog-tmp-<pid>` leftovers; only the stage-dir half was ever written"
created = 1786686871
updated = 1786686871
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["docs"]
+++
Found by bl-9639's one-time DESIGN sweep (Voussoir, 2026-08-14), slice §0–§3.

THE CLAIM. §2 I3 (the durability skeleton): "**I3 — All yog file writes are temp-in-destination-directory + `rename`.** Never in-place truncation, never a temp on another filesystem (EXDEV). Temp names are dotfiles (`.<name>.yog-tmp-<pid>`) so no substrate reads them; **leftovers older than 24 h are swept at startup.**" §5.2 repeats it for both scratch artifacts: "Plus two *transient scratch* artifacts that exist only inside an operation and are swept: config staging temps (`.<name>.yog-tmp-<pid>` in the destination dir) and the scripted-editor staging directory `$XDG_STATE_HOME/yog/stage/<nonce>/` (§9.3). Neither is an authority; **leftovers >24 h old are swept at startup.**"

WHAT IS TRUE. The write half is fully implemented — three temp-in-dir + rename sites, each a dotfile: src/ui_state/mod.rs:232, src/bz_host/store.rs:127, src/config_edit/pipeline.rs:81. The sweep half exists for exactly ONE of the two artifacts: `config_edit::branch::edit::sweep_staging` (src/config_edit/branch/edit.rs:232), called at boot from src/engine.rs:68, which enumerates SUB-DIRECTORIES of the stage root and `remove_dir_all`s the stale ones. Nothing in the tree ever enumerates or deletes a `.yog-tmp-*` file: `grep -rn 'yog-tmp' src/` returns the three creation sites and a watcher-exclusion test fixture, and `fn sweep` has one prod definition, the stage one.

WHY IT MATTERS AT ALL. A leftover only happens on a crash between write and rename — but each carries a distinct pid in its name, so repeated crashes accumulate distinct files, and two of the three destinations are the operator's own wall (`<wall>/brazen/`) and yog's data root, not a scratch dir. Nothing reads them (that is the dotfile ruling, and it holds), so this is hygiene, not correctness.

DELIVERABLE — one of two, and the ball is the ruling: (a) implement it, which is small and has a seat already: a `sweep_temps(dir, now_secs)` beside `sweep_staging`, called from the same boot line (src/engine.rs:68) over the destination dirs yog actually writes temps into; or (b) amend I3 and §5.2 to say what is true — the dotfile naming is what makes a leftover harmless, and only the stage directory is swept. Do NOT leave both the promise and its absence standing. Whichever way it goes, the two sentences must end up agreeing with the tree.