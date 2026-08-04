+++
title = "delete the obsolete workspace tool-grant rewrite and correct its stale design"
created = 1785649830
updated = 1785823716
claimant = "grant-cutter"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Audit collateral from bl-e249. Verify against the exact pin before editing.

Yog pins lernie 0.0.3, whose shipped `template/providers.yaml` already grants the worker the entire five-tool pool: `bash, dispatch, load_skill, message, read_file`. Yog's `src/start/grant.rs` and DESIGN §8.1 still claim the template has only `bash, read_file, load_skill`, then run a creation-time rewrite to append the other two. The rewrite currently returns "already granted", so the mechanism is redundant while its comments and architecture are false.

Delete the no-op grant path, its staging/config-edit machinery and tests that exist only for it; correct DESIGN and remaining comments. Prove a fresh yog workspace still receives the exact pinned lernie worker grant and no extra config commit. If the current pin no longer has the full grant when claimed, stop and amend this task instead of deleting a needed path.