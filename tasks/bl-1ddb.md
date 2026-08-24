+++
title = "the Login pane rides the boundary: rows via Query::Providers, the sign-in via Action::Login and the lane, aimed at the focused workspace's channel — entry workspaces included"
created = 1787548691
updated = 1787548691
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-c285"
on = "claim"
+++
## What this lands (REMOTE §8.3; needs bl-c285)

The window's Login surface (src/shell/login_pane.rs + src/shell/ram/login.rs) stops spawning bz itself and becomes a consumer of the boundary — the REMOTE §9.5 surface-by-surface migration's second surface, after clients:

- **Rows:** `Query::Providers { workspace }` over the channel that hosts the FOCUSED workspace (bl-028a: a local workspace asks this window's own engine; an entry workspace asks its entry's channel on its entry's material). `LoginHolder`'s in-process reads (`RealBzRunner`, `credential_presence`) and its `bz`/`wall`/`creds_dir` fields go; the holder keeps the rendered rows and the lane's painted lines. The `↻` re-ask becomes a re-ask of the same query.
- **Sign-in:** the Login button posts `Action::Login`; the painted stream is the follow-class lane. Closing the pane no longer SIGTERMs the run (engine-owned, bounded by bl-c285's replace + hour sweep). Amend the DESIGN §5.3 RAM row for streamed-verb output ('a device code is for the human at *this* keyboard') and §5.1 #21's 'When it is read' note — the rows now arrive by the boundary read — citing this ball.
- The bl-5894 parking survives shape-changed: the lane and its painted lines still ride WallRam per workspace, so a run never paints under another sphere.
- **The bl-3b62 which-sphere sentence** gains the channel fact for an entry: the sphere's name plus that it is held elsewhere (the roster row's client-side channel stamp — never a host address in the sentence).
- **Run-by-hand fallback per channel** (src/login/mod.rs::by_hand): a local workspace keeps `yog exec --ws <workspace> bz --login …`; an entry-hosted workspace spells the `yog seat` act, because there is no `yog exec --ws` for a wall this box does not hold.
- **The loopback remedy sentence** (REMOTE §8.3): when the focused workspace is an entry AND the row is browser-only, paint beside the run that the authorize URL redirects to the ENGINE's loopback — complete it in a browser on that box, or forward the port by hand (the streamed URL's own redirect_uri names it). A stated operator remedy, never a channel feature.
- The auth-failed banner's inline seat (§11: one machinery, two seats — login_section) migrates with the pane; both seats keep working.

## Discipline

Acceptance: shell/acceptance/first_run.rs (pressable Login beside the builtin oauth row) must stay green; add a beat for an entry workspace's pane painting rows served by a second engine (test_support wire fixtures — never a committed certificate). Paint assertions through crate::paint_probe only (no hand-rolled walks — rules/no-hand-rolled-paint-walk). Sweep the stale 'the window spawns bz' / 'always the browser flow' prose in src/login/mod.rs and src/shell/login_pane.rs headers to cite REMOTE §8.3.