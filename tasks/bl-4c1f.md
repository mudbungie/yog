+++
title = "consume lernie 0.0.10: the chat-wedge fixes (alternation-wide unpaired decline, crash settlement at the drive boundary)"
created = 1786844653
updated = 1786845082
claimant = "Dills"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Bump the lernie pin to =0.0.10 once it is on crates.io (release PR is in flight). What it carries, and why yog wants each:

- lernie bl-15f0: the hop warrant judges §2.3 tool pairing over the WHOLE alternation, so delivered mail behind a crash-orphaned tool window declines loudly instead of sending an orphan tool_use the provider 400s forever — the 'chat silently stopped answering' class, observed live on two agents in a yog workspace.
- lernie bl-4187: a markless unpaired trailing window is settled at the drive boundary (one in-band died tool_result per unanswered id, committed BEFORE delivery), so an ordinary deposit REVIVES a crashed chat instead of meeting the decline; fork-from-history stops being the only recovery. yog's bl-ace6 orphaned-mail banner renders the decline class this leaves behind (buried pre-settlement debris).

Cargo.toml is the pin authority; update its lernie comment (it currently narrates the 0.0.9 reasons). Verify brazen parity holds (one brazen in the lockfile, §16.7).