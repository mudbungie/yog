+++
title = "DESIGN 10.1 names a ghcr package for the seat that will never exist: record the seat's reasoned exception"
created = 1788583177
updated = 1788584008
claimant = "Animations-X"
priority = 4
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The one-image-per-component ruling in docs/DESIGN.md 10.1 lists `ghcr.io/mudbungie/lernie` among the four packages, and 10.1's opening says the two remaining repos follow the shape the containerized ones landed. The seat has since answered with a ruling rather than a build (lernie bl-18c7, recorded as lernie DESIGN 6.4): it ships no OCI image. An image is the unit of install for a box that takes images, and a seat's box is a desktop; its unit of install is the published crate, compiled on the box (the Linux reconciler, lernie bl-155a; `cargo install` on a mac, lernie bl-9380 / DESIGN 6.3). Every layer a seat image could carry — the GL stack, the display client libraries, fonts — is something the box has by being a desktop, while the display socket, the GPU device, the seat's XDG state and its wire material would all have to be mounted back in. The one face a container could host, the headless verb surface, is a different product bl-9380 already named, and one binary carries both faces today.

The ask is one doc edit here: in 10.1, drop `/lernie` from the package list or mark it as the recorded exception, and add a sentence under the registry ruling citing lernie DESIGN 6.4 so the two documents agree. No code, no workflow, no image gate changes — yog's own image line is untouched. 10.2's table row for the seat (cannot cross-produce) stays correct as written.