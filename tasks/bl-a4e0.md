+++
title = "README publication, reset, state, naming and installation claims contradict the code and the workflows"
created = 1786677244
updated = 1786677683
claimant = "Stromboli"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["publication"]

[[blockers]]
id = "bl-2368"
on = "claim"
+++
Source: publication audit follow-up 2026-08-13 (item 6), snapshot yog `e758814`.

Every one of these is README (or DESIGN) text that the code or workflows
contradict. Verify each against the tree, then fix the DOC or the CODE —
whichever is wrong — and say which you chose.

- `rm -rf $XDG_DATA_HOME/yog` expands to `/yog` when the optional variable is
  unset, and it is unquoted. The code's fallback is `$HOME/.local/share/yog`.
  Prefer a product-owned reset command with a preview; failing that, document
  the resolved, quoted path and the exact deletion scope.
- "The only durable, yog-owned state is `ui.json` ... and `ops.jsonl`" omits
  `cadence.yaml`, monitor policy, and detached stderr sinks.
- The README claims a workspace uses a "minted name"; workspace names are
  chosen, or the fixed `home`. Conversation names are the minted ones.
- "No substrate needs installing" contradicts the install receipt:
  `note: the window drives the 'lernie' binary; install it from the lernie
  repo.`
- "Publishing ... [is] never a side effect of CI or delivery" contradicts the
  automatic successful-CI release workflow.
- "DESIGN ... is kept in sync with the code" is false while DESIGN's module map
  and its local-versus-lernie mint statements disagree with source.

Gated behind the personal-material scrub so the two do not conflict on
`docs/DESIGN.md`.

---

Verified every bullet against the tree at d736e9e before editing. Corrections to the snapshot:

- ALREADY FIXED, no edit: the Publishing bullet. bl-5ae6 (e0b98f8) rewrote README's Publishing section; it now says both routes 'put a human at the trigger' and that only a CI run concluding success releases. Nothing in the tree claims publishing is 'never a side effect of CI or delivery'.
- MISSTATED: 'No substrate needs installing' is TRUE, not false. balls/brazen/lernie are exact-pinned linked crates and make drive-cleanroom proves the chain with only yog and git on PATH. The CODE was wrong: Makefile:290's install receipt still told the operator to install lernie from the lernie repo. Fixed the Makefile, kept the README.
- The mint bullet is bigger than 'DESIGN disagrees with source': DESIGN stated the bl-aca4 destination as landed fact in four places while src/names/ still holds words.txt, the Rng seam and mint(). DESIGN now states today's truth with a 'state of the move' paragraph naming bl-cd38 as the consuming ball.

Drift the audit did not name, found while verifying:
- STALE PIN RESTATEMENTS. README says Cargo.toml 'is the pin authority, so no version is restated here or in any doc'. Three docs restated anyway and two had gone stale: AGENTS.md rule 6 said lernie = "=0.0.3" (Cargo.toml has =0.0.6), DESIGN 5.1 #28a cited (=0.0.3), DESIGN 8.3 cited brazen = "=0.0.4" (Cargo.toml has =0.0.5). All three restatements removed rather than corrected.
- MODULE MAP GAPS. 23 tracked non-test source files have no row in DESIGN 12 (src/app/memo.rs, five of src/cli_outbound/*, three of src/config_edit/*, eight of src/git_tree/*, src/projects/{join,runner}.rs, src/shell/{fire,config_marks}.rs, src/shell/config_edit/status.rs, src/shell/acceptance/masthead.rs, src/config_edit/branch/edit.rs). Not fixed here — 23 rows is its own review. Filed separately.
- 'MINTED WORKSPACE' PROSE IN SRC. src/binding/mod.rs (the 3.1 authority module) and ~8 other files still call a workspace name minted, contradicting 3.1's 'A workspace name is chosen or home, never minted'. Not swept here to avoid re-litigating bl-2368's editorial pass. Filed with the module-map ball.
