+++
title = "the deploy path seats a binary unit while the server now runs the image: make deploy would overwrite the cutover"
created = 1788138752
updated = 1788138752
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The bl-b973 cutover left the deployment box running the OCI image under a `docker run` user unit, pinned to an immutable `yog:<version>-<short-commit>` tag. `scripts/deploy/` still seats the OTHER install: `seat.sh` scps a binary-shaped `yog.service` (`ExecStart=%h/.cargo/bin/yog`) plus the hourly `yog-update` reconciler, so `make deploy HOST=…` — the one documented way to (re)seat a server — would silently replace the container unit with a binary one and re-arm a reconciler that reads a cargo install and a cgroup neither of which is true of a container.

Two facts make this worth closing rather than remembering. The reconciler is not merely inert against a container unit, it is WRONG in the direction that acts: it would see the installed binary differ from what is running and restart the unit. And the box now carries the divergence with no marker in the tree — the container unit exists only on that box (its predecessor kept beside it as a `.prefence-bak`), so the repo cannot tell you what its own server is running.

Decide and implement ONE of: (a) `scripts/deploy/` grows the container unit as the seated one and `yog-update` is either taught the image or retired with it — the reconcile question becoming "is a newer image loaded" rather than "is a newer crate published"; (b) the deployment box goes back to the binary path and the image stays a build artifact; or (c) `seat.sh` takes the install shape as its second argument, which is a flag and therefore a smell — read it as the option to argue against.

Whichever wins, the box and `scripts/deploy/` must agree afterwards, and the answer belongs in README "The image" beside the mount contract it already states.