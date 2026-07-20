+++
title = "Z8: login flow — streamed-piped bz --login + auth-failure detection"
created = 1784524507
updated = 1784524507
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-7687"
on = "claim"

[[blockers]]
id = "bl-e324"
on = "claim"
+++
DESIGN §8.3 as amended (bl-d7a1) / §8 streamed-piped class / §5.3 stream row / §15 M6 Z8 / STORIES S0-T5, S0-T6, S0 step 5. Streamed-piped bz --login --provider <row> runner (cli_outbound addition: line-buffered stdout rendered live, outcome line at exit), provider rows from §5.1 #20/#21, exit-non-zero fallback = show exact command. Detection: auth-failed step in derived steps/response.json facts (§13.3) renders Login affordance beside failed step + toolchain pane. Files: src/cli_outbound/ streamed spawn, src/login/mod.rs or toolgate, view-model.