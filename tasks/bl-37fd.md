+++
title = "adopt the four-component split: yog server, lernie seat, litany engine, thrall foot"
created = 1787977082
updated = 1787977107
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator ruling 2026-08-28. The harness splits into four separately installed components, meeting only at the wire; brazen is unchanged as the provider adapter.

- **yog** — the standalone server: holder of the world, the balls, the conversations. No UI, no execution. What is `yog serve` today becomes the whole binary.
- **lernie** — the seat: the window and client faces, extracted from yog. The crate name flips at a version fence: the engine's line ends at 0.0.x, the seat begins at 0.1.0; both READMEs state the fence.
- **litany** — the agent-loop engine, the crate formerly named lernie. crates.io name held as a 0.0.0 placeholder.
- **thrall** — the foot: the tool-execution client (REMOTE §2's tool host, severed into its own installable). Name likewise held.

Two invariants ride with the ruling:

1. **Front door only.** Every execution is transported over the real wire — real socket, real handshake, real leaf — extending the bl-ae05 ruling (REMOTE §1.2, "everything through the front door") from the window to execution itself. No in-process executor. No unix-socket second transport in v1: one transport, no place to hide the bug.
2. **Ship inert.** A yog with zero enrolled thralls is valid and is the default. The server is structurally incapable of executing anything until a thrall is enrolled — even a single-box install enrolls its local thrall as an explicit act.

Migration order (strangler; each step ships green): thrall founded → litany renamed → seat severed → yog drops the UI and goes inert-by-default.

Deliverable: DESIGN.md and REMOTE.md amendments recording the split, both invariants, and the order. The seven follow-on balls gate their claims on this one so the ruling text lands before the surfaces fan out.