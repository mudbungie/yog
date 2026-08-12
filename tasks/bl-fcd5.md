+++
title = "the headless boundary has no wall: /config brazen through yog gesture refuses because no workspace is focused — the gesture must name its workspace"
created = 1786509736
updated = 1786509736
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found while landing bl-c0e2 (workspace blast radius, f34c43c). Provider config
now lives inside the workspace wall, and the wall is derived from focus — but a
headless seat has no focus, so a /config brazen gesture through yog gesture /
yog headless refuses with "focus a workspace first". A teleoperator (VISION V5)
cannot reach provider config at all.

The shape exists already: ConfigFile::Branch names its workspace in the gesture
spelling; ConfigFile::Brazen needs the same workspace field. This is a boundary
serialization change — per the control-boundary discipline, land it as a new
variant first (additive), with all three serializations (slash line, envelope,
typed action) and the refusal kept for a gesture that names no workspace and
has no focus to fall back on.

Verify against src/boundary/{config.rs,dispatch.rs} and §8.5 as amended by
f34c43c before editing.