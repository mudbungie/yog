+++
title = "the image is a published artifact and the gate has never read one: §10.1 names the registry and requires an image-side disclosure scan ahead of every push"
created = 1788069063
updated = 1788069064
claimant = "OrderCustoms"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator ruling 2026-08-30 answered the question DESIGN §10.1 left open —
"Where these images publish is unanswered" — with `ghcr.io/mudbungie/<component>`,
one package per repo, **conditional on a mechanical guarantee that no image
ships a secret**.

The condition is not covered by anything standing. `make leak-scan` reads the
git INDEX; an OCI image is a different artifact built from a different input
set (the build context, the base layers, the package index, the build
configuration), and no byte of it has ever been read by a gate. The source gate
sits ahead of the commit; nothing sits ahead of a push.

This ball is DOC-ONLY in yog: amend §10.1 to record (a) the registry ruling and
its shape, and (b) the image scan as a REQUIRED property of every containerized
repo, so the two repos that already build images and the two that will follow
land against one statement rather than four.

The implementation is per repo, in each repo's own store — there is no shared
build tooling and no meta-repo (§10.1), and a scan that spanned them would be
the first place the components met somewhere other than the wire.