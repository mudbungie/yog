+++
title = "consecutive_fires beat is probabilistic: post-fire mint seeds re-roll from real entropy, so the acceptance fixture races lernie's word pool"
created = 1786684759
updated = 1786684759
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Filed from Lintel's bl-a842 run and Gimbal's earlier unnamed flake sighting (2026-08-13): shell::acceptance::mint_seed::consecutive_fires_each_predict_and_spend_a_seed_of_their_own failed once at mint_seed.rs:89 with left == right ("will be named metronome" twice), then passed five consecutive re-runs.

Root cause (verified by Lintel): only the OPENING seed is pinned (MINT_SEED). src/shell/ram.rs:62 re-rolls from real entropy on every landed fire (self.mint_seed = entropy_seed();), so the second and third draws are genuine random draws and assert_ne!(third, first) is a probabilistic assertion over lernie's 541-word pool. This flake has broken at least one unrelated close gate today (bl-2d19's first gate attempt).

Fix direction: the fixture should own the post-fire seeds the same way bl-cba6 made it own the first one — deterministic seed sequence in the acceptance fixture, entropy only in production. Verify premises against the tree before editing.