+++
title = "W8 embed balls: typed store reads in-process, mutations via yog bl multiplex"
created = 1784784298
updated = 1784956067
claimant = "Exultantly-b05e"
parent = "bl-b5d1"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
DESIGN §16.7 W8. BlRunner prod impl (BlStore) reads the balls store IN-PROCESS over the promoted typed surface (reads::Catalog/Entry -> Ball); mutating verbs spawn `yog bl <verb>` through the W12 multiplex arm (balls::run), and Binary::Bl is self-multiplexed.

Deviations from the original spec, all recorded in DESIGN §16.7 W8 "as built":
1. The CLOSED listing still spawns. balls promoted only the LIVE Catalog; the dead-ball history walk (reads::history) stayed crate-private, so `bl list -s closed --json` remains a subprocess (now `yog bl ...`) and projects::balls::parse_list survives as the last JSON hop. New upstream ask recorded: U-balls-2, promote the dead-set walk.
2. "Unlistable" (the §3.5 orphaned-project signal) is now balls own foundedness test (landing carries config/), because Catalog::load is silent-empty on a missing tasks/ and no longer fails as a process.
3. yog stopped mirroring balls path arithmetic: xdg::Env::balls_layout() returns balls::layout::Xdg and every balls path derives from it.
4. W5 retired for bl by emptying its probe list (classifies Ok with zero spawns); the uniform gate_check call sites stay, W13 still deletes the module.

Blocker found and cleared: the sanctioned rev 15b50589 existed only in the LOCAL balls mirror (upstream main was 51 commits behind and diverged), so the git pin could not fetch. That exact commit was pushed to a new upstream branch `yog-w8-pin` (additive, main untouched, no CI triggered). Cargo pin also carries version = "=0.5.7"; deny.toml gained the allow-git entry.

Consequence: crates.io refuses git deps, so `make publish` / release-plz cannot ship yog until balls releases the promotion and the pin becomes =x.y.z.