+++
title = "design-doc jargon leaks into operator UI: '§9 — stage → validate → hash-guard → atomic rename', 'project marks (bl store branch)'"
created = 1785646882
updated = 1785647055
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
RE-VERIFIED after bl-c225 landed: the jargon moved, didn't die. Current strings — Config pane subtitle '§9.5 — every setting is a control over the file that holds it'; raw row label 'raw config.toml — bz is the only lawful parser (§9.1)'; plus unexplained ⚑ flag icons on some provider rows (local/claude-code/openai/mistral/openai-responses/google/ollama carry a flag glyph with no legend — say what it means or drop it). §-references are DESIGN.md coordinates an operator cannot dereference; 'lawful parser' is spec voice. Fix: operator words ('every setting edits the file that holds it', 'raw file — validated by bz before it lands'), no §-refs anywhere in rendered UI, and the ⚑ either legended or removed. Sweep all rendered strings for '§'.