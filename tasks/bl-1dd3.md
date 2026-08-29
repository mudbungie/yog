+++
title = "found thrall: the foot that owns the local execution corpus"
created = 1787977099
updated = 1787977674
claimant = "OrderScribe"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-37fd"
on = "claim"
+++
New repo and crate (crates.io name held at 0.0.0). A thrall is a tool-execution client whose entire wire surface is: advertise, wait on its mailbox, post captures back — REMOTE §2's tool host, severed into a separately installable component. The dial-in transport is unchanged: the thrall dials per ask and holds a connection only while waiting; nothing in the path is the engine speaking first (REMOTE §5, §3).

Two additions over the landed tool host:

1. **Foot-grade certificate.** A thrall's leaf may advertise and execute only — no ask, no act. This narrows REMOTE §2's "every certificate is operator-grade within its registrations" for the class, and closes §9.6's stated residual (a registered client reads the trail of every workspace) for feet without reopening §11's rejection of per-tool/per-verb ACLs: the grade is binary, not a policy layer.
2. **Thrall owns local execution entirely** (per the bl-37fd ruling). Workspace-level substrate access — balls ops, repo tools — is itself a thrall tool, and running a thrall beside the server is the normal install, not a special case. yog itself exposes only engine acts (agent lifecycle: spawn, compact), never local integrations.

Local config gates what a thrall enables; server-side adjudication is unchanged and fails closed; §5's containment-honesty clause carries over verbatim — execution happens on a machine the adjudicator cannot inspect.

MCP enters only as a thrall-local bridge: thrall as MCP client, re-advertising local MCP servers' tools up the wire. Deferred, not v1; yog never learns MCP.

The new repo stands up the full gate apparatus (line cap, coverage floor, leak-scan, hooks) and its own store. Deliverable here: REMOTE amendments naming the component and the grade; founding lands in the new repo.