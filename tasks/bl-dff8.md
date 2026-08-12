+++
title = "headless config discovery is incomplete: no lineage-file read or provider-model query"
created = 1786510266
updated = 1786510266
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["headless", "config"]
+++
README claims every operator gesture is drivable headlessly, but two populated GUI reads have no boundary spelling on `96d5f4e`:

1. The config branch pane browses `config/*`, loads a selected file, and shows its bytes. `/config branch <lineage> <path> <text>` can write, but omitting text is refused; `Query::ReadConfig` explicitly refuses `ConfigFile::Branch`. A headless operator must overwrite a file it cannot inspect.
2. The model picker calls `bz --list-models --provider <row> --json` and fills a roster. `/providers` can discover provider names and `/model` can set an id, but `Query` and `yog gesture --help` have no model-roster read. A headless operator must guess valid ids.

These are one missing inventory class: GUI reads that populate a control but never crossed §8.5. Add narrow typed queries using the same underlying reads as the GUI, with slash/help spellings and fake-substrate tests. If parity is not intended, amend the absolute headless claim instead of retaining capability theater.