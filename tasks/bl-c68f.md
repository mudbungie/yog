+++
title = "W1: world-env module — compose the nested Env and the world layout"
created = 1784435199
updated = 1784435199
parent = "bl-1a3c"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
DESIGN §16.6 W1. New src/world/mod.rs (~150): from the ambient xdg::Env, compute yog's data-root anchor and derive the world subtree (<yog-data-root>/world/{lernie,state,brazen/config.toml,tools}); compose the world Env = ambient + {LERNIE_HOME, XDG_STATE_HOME, BRAZEN_CONFIG} overrides; XDG_DATA_HOME/XDG_CACHE_HOME left ambient (the anchor + brazen creds/cache sharing, §16.2). Pure over an injected ambient Env; every override and both nested and shared derivations table-tested; yog re-derives all substrate roots and its own two artifacts (ui.json, ops.jsonl) through the world Env. Task 0: confirm bl-delivery derives worktree territory from its own $XDG_STATE_HOME (read /home/u/dev/balls source via git show, or empirically with a scratch repo + overridden env) and record the finding in the module doc. Gate as always.