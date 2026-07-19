+++
title = "W6: yog env / yog exec — world escape hatches"
created = 1784435201
updated = 1784435201
parent = "bl-1a3c"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-c68f"
on = "claim"

[[blockers]]
id = "bl-ea39"
on = "claim"
+++
DESIGN §16.6 W6. Two multi-call subcommands beside --editor-apply: `yog env` prints the world's export lines (shell-safe quoting); `yog exec <cmd...>` runs a command inside the world (world env layered over inherited, optional --cwd), per §8.4. Pure argv -> plan fns (tested); the exec spawn reuses cli_outbound; main.rs dispatch stays thin/excluded. These are the composability inversion's mitigations — a human or foreign frontend joins the world with one prefix. README gets a short World section documenting both. Gate as always.