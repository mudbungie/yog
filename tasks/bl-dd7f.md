+++
title = "a config-kind dispatch error renders a remedy beside the reason, and the picker names the row that failed"
created = 1786683729
updated = 1786685134
claimant = "Newel"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Ruled at bl-9b52 (operator, 2026-08-13), question 2: yes — a remedy beside the reason.

Today (bl-9b52's falsifying screenshot, s0-04-transcript.png): a dispatch through a provider row that does not resolve renders the reason verbatim ('lernie prompt: provider error (Config): unknown provider `openai-chatgpt`') with a Dismiss and NOTHING else — no path to the §9.1 raw-TOML editor (the only remedy for a config-kind failure), while the composer's provider picker shows a DIFFERENT row (anthropic) than the one the conversation actually dispatched through.

Scope: (1) config-kind step errors get an affordance pointing at the §9.1 editor, exactly as auth-kind errors get Login (§8.3/§13.3) — the judgement happens at first dispatch against a wall that exists, not at birth (§9.2's birth gate was retired for judging against a wall that did not exist yet, bl-00ee; do not resurrect it). (2) The picker beside a failed step must name the row that failed, not the template default.