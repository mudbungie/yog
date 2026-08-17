+++
title = "a started receipt returns a name that only message can address; every other agent surface needs the root id"
created = 1786843631
updated = 1786936198
claimant = "Receipt49bc"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "headless", "boundary", "addressing", "design"]
+++
A successful `/prompt` answers `{"ok":true,"kind":"started","conversation":"<minted-name>"}`. The terminal contract says `--agent ID`. Feeding the receipt only handle back composes with `/message` alone; every other conversation action and inspector read treats it as an ID.

Observed examples:

- `/agent` reports `present:false`.
- `/steps` and `/transcript` return empty rows.
- `/stop` and `/retarget` refuse.
- `/message` succeeds because it is the sole lernie verb that resolves a unique stored name.

The code contract states: `This is the one verb that resolves a display name`. Yog separately says: `The prompt action product: the minted conversation name`, while its command usage says `--agent ID`. These deliberate local contracts became non-composable when exposed as one headless API.

The split is more dangerous than empty reads. Name-targeted `/delete-agent`, `/flag`, and `/revoke` or `/restore` can report success or write policy against the name while leaving the real conversation untouched. The only recovery is to wait for `/conversations`, match `name == started.conversation`, and take `root_id`.

Decide one universal address contract and amend DESIGN before code. Prefer one engine-boundary resolution of exact ID, otherwise unique stored name, for every agent-addressed Action and Query; alternatively, delay or widen Started until it can return the root ID. Do not add per-verb lookups. Coordinate immediate post-start visibility with bl-6c9e; this mismatch persists after derivation catches up.

Acceptance: using only the Started handle, an immediate `/agent` read and representative `/message`, `/stop`, and `/seen` actions address the started root; exact IDs remain unchanged; unknown and legacy display-only names refuse; no delete, floor, or flag operation can succeed against the display name while missing the real root.