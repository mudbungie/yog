+++
title = "REGRESSION of bl-f5f6: /attention returns an engine path that remote gestures cannot reuse"
created = 1787206328
updated = 1787206328
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "wire", "security", "addressing"]
+++
## Reproduction

Ask `/attention` over a registered wire seat. A row returns:

```json
{"workspace":"<engine-data-root>/yog/workspaces/<name>","agent":"<agent>",...}
```

Copy that workspace and agent into the documented next action, for example `/seen`. Resolution refuses the absolute value as an unknown workspace.

## Contract

`docs/REMOTE.md` says:

> “Paths never cross the wire… across machines those are meaningless and a disclosure besides. The wire spelling is now the name.”

The queue also promises that its returned address can be copied directly into message, stop and seen gestures. `QueueRow` remains `PathBuf`-backed and its encoder emits the engine-local path.

## Impact and required invariant

This is both disclosure and broken teleoperation: the remote operator receives an address that cannot be used. Attention must carry the boundary workspace name, and its returned pair must round-trip into an agent gesture. Audit adjacent wire replies that serialize `cwd`, workspace or project paths against the same landed invariant.