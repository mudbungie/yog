+++
title = "Design amendment: the yog world — nested tool state, pinned tuple, crate adoption phases, no-marks knob (DESIGN.md)"
created = 1784434931
updated = 1784435000
claimant = "filtered"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Amend docs/DESIGN.md with the decided architecture evolution (decisions settled; record and decompose). CORRECTED per user 2026-07-18 (supersedes the earlier body's point 4):

1. yog is an APPLICATION, not a layer. Play-on-top survives via redirection knobs; compatibility becomes the user's decision.
2. The nested world: LERNIE_HOME under yog's data root (seeding via lernie's own bootstrap verb — upstream bl-6d83; yog never apes lernie seeding); XDG_STATE_HOME override on child bl (nested clones/worktrees/op-logs; task 0: verify bl-delivery derives territory from its own env); BRAZEN_CONFIG nested; brazen credentials + model cache SHARED with ambient (auth reuse decided).
3. Balls store branch: shared balls/tasks by default (coordination point; stable contract). Per-project NO-MARKS knob (stealth via bl conf task-remote none, and/or custom task-branch) for users who want yog without leaving marks; document the trade.
4. CORRECTED — crates, no shipped binaries: the end state embeds all three as exact-pinned crates and yog ships/installs NO tool binaries; the Cargo pins ARE the version mechanism. Binaries matter only as agent tools: ambient-world use = the user's own installs (orthogonal); embedded-world use (mostly balls — ambient bl computes wrong clone/worktree paths against yog's nested state root, so this is correctness, not convenience) = yog exposes the embedded crate surface as tools via lernie's external-tool convention (nested <data-root>/tools/lernie-tool-bl etc., a multi-call re-exec of the yog binary speaking bl's argv contract against the nested roots).
   - Phase 1 (host binaries, because lernie is not lib-ready): world-env composition, nesting, no-marks knob, lernie-init seeding, yog env / yog exec escape hatches, startup version gate (phase-1-only device; probe what exists — bl/lernie lack --version today). Any install convenience is phase-1-scoped and explicitly retired by phase 2.
   - Phase 2 (per upstream readiness): balls via its [lib] (0.5.7); brazen pin-exact (canonical Event types + validation); lernie gated on upstream bl-231c (library port + driver/successor exec parametrization; linked lernie = yog re-execs itself as the driver binary). Process semantics non-negotiable regardless of linking: drivers are processes holding flocks; balls plugin dispatch stays subprocess. Plus the agent-tool exposure shims.
5. Produce the implementation decomposition as bl-ready task specs (§15 style) — phase 1 fully; phase 2 as gated placeholders naming upstream deps.