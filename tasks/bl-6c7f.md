+++
title = "a half-provisioned entry is told to run the host's recipe on the wrong box: material::REMEDY names this box's root, but an entry's material is minted on its host"
created = 1787545431
updated = 1787545431
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## The wrong place, the right recipe

`material::REMEDY` is `make wire-certs` — the act that mints **this box's own**
root. An entry's material is not minted here: §1.4 and REMOTE §8.2 are explicit
that the host's operator issues the visiting box's leaf and the anchors, leaf
and key are then carried by hand, out of channel. So when a partial entry
refuses (`src/wire/entries.rs`, bl-5b68), the sentence it inherits from
`material` names the correct recipe and the wrong machine — an operator who
follows it mints a fresh loopback root on the client and is no closer.

The empty-entry case already says it properly, because bl-5b68 wrote that
sentence itself: *"its material is minted on the host that issued it (`make
wire-certs` there) and carried here by hand"*. The partial case is the one
still inheriting the flat root's wording.

## Do not fix it at the call site

The tempting repair is to post-process `material`'s sentence where entries
raises it. That is a module rewriting another module's prose and it will drift
the first time either changes. The honest fix is a remedy that knows **which
relationship it is talking about** — the flat root and an entry are two
different provisioning acts on two different boxes, and only the material
itself knows which one it is.

That is a small amount of shape, not a flag: the remedy is a fact of the role
being read, and `material::read_dir` already takes the role.

## Scope

Small and self-contained. Keep `material`'s existing message byte-identical for
the flat root — that path is reached from the engine's own boot and its wording
is load-bearing there. Verify the claim against the tree before editing: this
body is a report from the ball that found it, not evidence.