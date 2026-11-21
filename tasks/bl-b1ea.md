+++
title = "A workspace's brazen config carries its own copy of the built-in provider rows, so a corrected built-in never reaches an existing workspace"
created = 1794985256
updated = 1795235029
claimant = "Rabid-fix"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The model-list endpoint pin on the ChatGPT-subscription row was corrected in brazen's defaults (brazen bl-6e28), and every existing workspace kept listing the short catalogue because its seeded config restates the row and outranks the built-in. Two representations of one fact. Either stop seeding rows that equal the built-in, or seed only overrides.

---

Landed as the rule and not a migration. No yog code path writes a row into a wall (DESIGN 16.2, born empty), so there was no seeding to stop: a restated row is always a hand's, pasted whole through /config brazen or copied in as a file. The rule is now in DESIGN 16.2, the README and the /config help page. A mend in code was attacked and refused: brazen folds a file row over the built-in per top-level field (oauth, models and ambient each taken whole), so a field equal to today's built-in is harmless and a field equal to an older built-in is the same bytes as a deliberate pin; yog holds no history of the table to tell them apart. Existing walls are brought under the rule by hand, once, read against the built-in table.
