+++
title = "start replies with a display name that most agent verbs cannot address"
created = 1786843631
updated = 1786843631
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "headless", "boundary", "addressing"]
+++
A successful `/prompt` replies with `{"conversation":"<display>","kind":"started","ok":true}`. This is the only agent identity in the start receipt.

Feeding that returned `<display>` back through `--agent` gives inconsistent results:

- `/agent` reports `present:false`.
- `/steps` and `/transcript` return empty rows.
- `/stop` reports that the branch does not exist.
- `/retarget` reports that no agent exists.
- `/message` is the exception and succeeds by display name.

A later `/conversations` query reveals a separate `<root-id>`. Supplying that root ID makes all tested inspectors and controls work.

Expected: the start receipt returns the general agent address, with display text separate if useful, or every `--agent` consumer resolves the returned identity consistently. An action receipt must compose directly with the next boundary call; discovering an undisclosed identifier through a roster query cannot be required.