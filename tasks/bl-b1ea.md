+++
title = "A workspace's brazen config carries its own copy of the built-in provider rows, so a corrected built-in never reaches an existing workspace"
created = 1794985256
updated = 1794985256
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The model-list endpoint pin on the ChatGPT-subscription row was corrected in brazen's defaults (brazen bl-6e28), and every existing workspace kept listing the short catalogue because its seeded config restates the row and outranks the built-in. Two representations of one fact. Either stop seeding rows that equal the built-in, or seed only overrides.