+++
title = "DESIGN: sign-in is stranded on the box with the browser — a seat must be able to run SSO for a remote workspace's wall, where the agents that need the credential actually run"
created = 1787548511
updated = 1787548511
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design"]
+++
## The gap, found live

An operator registered a client into a remote workspace (REMOTE §8.2, the
entry), fired a chat there, and hit `no models configured`. They then ran the
Login flow — which succeeded, onto the WRONG BOX: the seat's own wall shows
`openai-chatgpt oauth2 stored`, the host workspace's wall shows `missing`.
Agents run on the host and read the host's wall (§16.2, per-wall credentials
since the blast-radius ruling), so the chat still cannot run.

This is structural, not a misclick. Login today (`src/login/mod.rs`, §8.3 as
amended):

- the window spawns `bz --login --provider <row> --browser` **on its own box**,
  with the local wall's env layered (`bz.and_env(login.wall)`);
- the flow is the loopback AuthCode flow (RFC 8252) **always** (bl-b4e5): bz
  binds a localhost callback and opens the browser beside it. Browser and
  callback must share a box, by construction;
- bz's other flow — device (RFC 8628) — is refused for any row whose `oauth`
  block omits the optional `device_url`, "which is most of them", and yog
  forces `--browser` unconditionally (a desktop has a browser; it also — until
  the split — was always the box that needed the credential).

So there is no spelling that puts the BROWSER on the seat and the CREDENTIAL
in the host's wall. That spelling is what this design must produce.

## The shapes to weigh

**(a) The login is an act on the boundary, executed by the ENGINE, streamed to
the seat.** bz runs on the host, its wall env correct by construction; its
sign-in lines cross the wire as a streamed reply (the streamed-piped class
already exists, and the wire already carries N-frame answers). The human at
the seat gets the URL. Two sub-cases:
  - rows WITH a device endpoint: the device flow is finished — the URL + user
    code paint at the seat, the human completes in any browser anywhere, bz on
    the host polls the token endpoint. **The credential never crosses yog's
    wire.** This is the shape that also works from a phone.
  - rows WITHOUT one (openai-chatgpt today): the AuthCode redirect aims at a
    loopback the seat's browser cannot reach. The manual remedies are an ssh
    port-forward (works today, an operator act, undocumented) or a paste-back
    arm (the seat's human copies the redirect landing back into the stream —
    an out-of-band completion). Whether either belongs in the product, and
    whether the real fix is UPSTREAM (brazen gaining `device_url` on rows
    whose provider serves one, or a paste-back flow), is this design's call —
    yog cannot edit brazen; the upstream ask must be filed as part of the
    deliverable.

**(b) Sign in at the seat, deposit the credential to the host.** The browser
UX is native, but a bearer credential crosses yog's wire, and DESIGN §5.1 #22
is explicit that yog never reads or writes a credential — bz owns them. A
deposit act would put yog in the custody chain. Weigh it honestly, but (a)
looks like the grain of the design.

## Constraints

- **No wire-only verbs** (REMOTE §3): whatever login becomes, it is a boundary
  capability every face gains, not a wire feature.
- **§1.4 is about wire material** (certificates, out-of-channel forever) — a
  provider credential is a different object; do not import that ruling by
  analogy without argument.
- **§5.1 #22**: yog renders the flow, never touches the credential.
- The window's Login pane must aim at the FOCUSED workspace's wall — which
  since bl-028a can be a remote one; the pane currently has no notion of that.
  What the pane paints for a remote workspace, and what `yog seat` spells for
  the same act, are both in scope (one capability, N faces).
- Deliverable per house rules: amend the authority docs (REMOTE and/or DESIGN
  §8.3), file the sequenced implementation balls, file the upstream brazen ask.