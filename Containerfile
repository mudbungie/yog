# yog as an OCI image — the unit of install, and nothing more.
#
# The image is a DEPLOYMENT artifact. Nothing in yog uses the container
# filesystem as a feature, no state lives in a layer, and the container is not
# a containment claim yog makes on the wire (DESIGN §10.1). Read the README
# section "The image" for the mount contract this file implements.
#
# **What ships is the UI-FREE SERVER** (bl-7942). A bare `yog` boots the
# engine and parks; there is no window, no display stack and no `serve` verb.
# The image existed as a deferral for exactly as long as there was a GL stack
# to keep out of a layer nothing in a server can present.
#
# Two stages. The build stage is the pinned toolchain and the C toolchain the
# TLS stack needs; the runtime stage is the small set of programs the engine
# actually EXECS, and nothing else.

# ---------------------------------------------------------------------------
# Stage 1 — build, under the toolchain rust-toolchain.toml pins.
#
# `rust:<pin>-alpine` and not `-slim-bookworm`, because the host target of the
# alpine image IS `x86_64-unknown-linux-musl`: the release binary comes out
# statically linked with no cross-compilation setup and no `--target` flag to
# keep in step with anything. The tag is digest-pinned so a rebuild of this
# file resolves the same bytes; the tag beside it is for a human reading the
# line.
FROM docker.io/library/rust:1.95.0-alpine3.22@sha256:064dfc925d68d1a63f4fd2871bd7dc6e6ea56692989a487185855d62885d90aa AS build

# `musl-dev` is not optional and not incidental: `rustls` is linked with the
# `ring` provider (AGENTS.md rule 6 says why `ring` and not `aws-lc-rs`), and
# ring compiles C. This is the one C toolchain the build needs and the runtime
# stage carries none of it.
RUN apk add --no-cache musl-dev

WORKDIR /src

# The toolchain pin has ONE home — rust-toolchain.toml — and the `FROM` line
# above is a second statement of the same fact, so it can drift. This makes the
# drift a build failure instead of a silent difference between what the gate
# compiles and what the image ships.
#
# It is copied to /pin and not to the build directory on purpose: a
# rust-toolchain.toml in the working directory sends every later `cargo` and
# `rustc` through rustup's shim, which would try to DOWNLOAD the toolchain and
# its `components` list into an image that already has the compiler. The check
# reads the file; the build never sees it.
COPY rust-toolchain.toml /pin/rust-toolchain.toml
RUN set -eu; \
    pin=$(sed -n 's/^channel *= *"\([^"]*\)".*/\1/p' /pin/rust-toolchain.toml); \
    have=$(rustc --version | cut -d' ' -f2); \
    if [ "$pin" != "$have" ]; then \
      echo "Containerfile: base image rustc $have, rust-toolchain.toml pins $pin" >&2; \
      echo "  bump the FROM tag and its digest in lockstep with the pin" >&2; \
      exit 1; \
    fi

# The manifests and the crate source, and nothing else: yog reads no asset at
# compile time at all. `tests/packaged_files.rs` proves that in both directions
# over the real `cargo package --list` — it fails if the build gains a
# `include_bytes!`/`include_str!` outside `src/`, which is the same question
# this `COPY` list answers.
#
# `--locked` for the same reason the gate uses it: the committed Cargo.lock is
# the dependency answer, and a build that is allowed to solve for a different
# one is not the build the gate judged.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked --bin yog

# ---------------------------------------------------------------------------
# Stage 2 — runtime.
#
# THE RUNTIME LAYER IS WHAT THE ENGINE EXECS, and this engine execs four
# things. `FROM scratch` is wrong here whatever the linking story says:
#
#   git      — every workspace read and every act is git (`src/git_env.rs` is
#              the crate's ONE fork, and `git_env::git()` is most of what goes
#              through it: the §2.3 tree derivation, the §8.6 hold marks, the
#              §3.9 retention walk, balls' own landing repair).
#   openssl  — the wire mint (`src/wire/provision/openssl.rs`). yog links no
#              certificate library and mints nothing in channel (REMOTE §1.4);
#              the recipe shells to `openssl`, and the engine's own boot runs
#              it when a box has none. Without it a fresh mount comes up with
#              no listener at all.
#   sh       — the `$EDITOR` re-entry a §9.3 lineage write performs
#              (`sh -c 'exec {EDITOR} "$1"'`, src/multiplex/litany.rs).
#              busybox provides it.
#   yog      — itself. §16.4/§16.7's self-multiplex means the world's `PATH`
#              head is re-exec shims of this binary, so `bl`, `litany`, `bz`,
#              `bl-delivery`, `bl-tracker` and `tool-control` need no host
#              binary: every one of them IS `yog <namespace> …`.
#
# `ca-certificates` is here because the embedded adapter speaks HTTPS to a
# provider endpoint and `git` may be pointed at an HTTPS remote. It is the one
# thing on this list that is not exec'd but is still load-bearing.
#
# **`lsof` is deliberately absent.** §10's macOS liveness backend shells to it;
# Linux answers the same two questions from `/proc` and the shim is
# `cfg(target_os = "macos")`, so it is not compiled into this binary at all.
#
# **Nothing an AGENT runs is here, and that is the ship-inert posture**
# (REMOTE §12, bl-37fd). A tool call routes to an enrolled thrall over the real
# wire or refuses in band; the engine runs no tool itself. An image that
# carried a toolchain for agents to use would be quietly undoing that.
FROM docker.io/library/alpine:3.22@sha256:14358309a308569c32bdc37e2e0e9694be33a9d99e68afb0f5ff33cc1f695dce

RUN apk add --no-cache git openssl ca-certificates

COPY --from=build /src/target/release/yog /usr/local/bin/yog

# THE MOUNT CONTRACT. XDG is the runtime contract and the image carries no
# state, so this sets the variable and provisions nothing under it.
#
# **`XDG_DATA_HOME` is the world's anchor and therefore the ONLY root that
# mounts** (§16.2). yog composes a nested world under `<yog-data-root>/world`
# and overrides `LITANY_HOME` and `XDG_STATE_HOME` onto it for itself and every
# child it spawns — so those two are *derived*, never mounted, and an operator
# who mounted the three separately would be fighting the nesting. `XDG_DATA_HOME`
# is left ambient precisely because overriding it would recurse.
#
# With `XDG_DATA_HOME=/state`, yog's data root is `/state/yog` — the extra level
# is XDG's, not the image's — and that ONE directory holds:
#
#   /state/yog/world/litany   the nested LITANY_HOME (config and data)
#   /state/yog/world/state    the nested XDG_STATE_HOME: balls clones and
#                             worktrees, and yog's own ui.json / ops.jsonl
#   /state/yog/world/tools    the re-exec shims (seeded by the engine, not here)
#   /state/yog/workspaces     the §3.1 named workspaces
#   /state/yog/wire           the wire material — BESIDE the world, not inside
#                             it, because the world is a generated artifact yog
#                             reseeds and a reseed must not be a revocation
#
# Mount the operator's provisioned data root at /state/yog.
#
# **Nothing here runs a seed.** The engine founds what it finds missing on its
# own boot, into the mount; writing any of it into a LAYER would put the one
# state yog owns where a mount cannot replace it and an upgrade cannot see it.
#
# There is no VOLUME instruction. A VOLUME would let an unmounted run succeed
# against an empty anonymous volume — which for yog is worse than for the other
# components, because the boot would found a whole world there and answer as if
# it were real. Without one, an unmounted run founds its world inside the
# container and takes it down with the container, which is at least visible.
ENV XDG_DATA_HOME=/state

# Workspaces live inside the data root above, so this is only where a start's
# bare rung puts a driver's cwd (`$HOME`, DESIGN §8.1). The image asserts no
# location for a project checkout; whatever path is named has to be a mount if
# work on it is to outlive the container.
WORKDIR /work

ENTRYPOINT ["/usr/local/bin/yog"]
CMD []
