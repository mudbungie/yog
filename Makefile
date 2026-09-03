.PHONY: all build release test coverage lint fmt fmt-check check install-hooks install uninstall print-install-stamp ci publish clean rules-audit line-cap leak-scan deny image image-scan \
        corpus drive drive-preflight drive-cleanroom drive-seed drive-log wire-certs fixture \
        deploy deploy-status

# Install location for `make install`. Defaults to the XDG-ish user-local
# convention; override for system-wide installs or packaging:
#   make install INSTALL_PREFIX=/usr/local
INSTALL_PREFIX ?= $(HOME)/.local
INSTALL_BIN    := $(INSTALL_PREFIX)/bin
# The commit the installed binary was built from — the ONE record of "what is
# installed", written by `install` (the act that makes it true, whoever ran it)
# and read by scripts/install-main to decide whether main's tip needs building
# at all (bl-6ff1). Beside the binary, so it moves with INSTALL_PREFIX and a
# second prefix cannot inherit the first one's answer.
INSTALL_STAMP  := $(INSTALL_BIN)/.yog.commit
# Build output root. Defaults to `target`; the local CICD script
# (scripts/install-main) overrides it to the repo's own target/ while
# building from an ephemeral worktree, so the release build stays incremental.
# Exported so the cargo invocations below honor it too.
CARGO_TARGET_DIR ?= target
export CARGO_TARGET_DIR

# The runtime world this box's engine holds: the SAME root yog itself resolves
# for `$XDG_DATA_HOME/yog` (`yog_data_root`, src/xdg/mod.rs — the env var if
# set, else `~/.local/share`). Only `WIRE_DIR` below derives from it now; the
# `ux`/`reload` launch verbs that used to live here were the window's, and a
# server is started by its unit, not by a Makefile (`make deploy`).
YOG_DATA_HOME := $(or $(XDG_DATA_HOME),$(HOME)/.local/share)/yog
# The REMOTE §9.5 wire's key material (bl-b6fa). BESIDE the world subtree, not
# inside it: the world is a generated artifact yog reseeds, and a reseed must
# not be a revocation. Same `yog_data_root` fold as everything above, so the
# directory the engine reads and the directory this mints into are one path.
WIRE_DIR      ?= $(YOG_DATA_HOME)/wire
WIRE_HOST     ?= 127.0.0.1
WIRE_PORT     ?= 7737
# Set to a common name to ask the same recipe for ONE extra client leaf instead
# of a mint (REMOTE §8.2) — the leaf a visiting box participates as. Empty is
# unset, so the default target is the mint it has always been.
WIRE_LEAF     ?=

all: check

build:
	cargo build

release:
	cargo build --release

# Tests run in parallel. Fixtures write throwaway executables and fork
# subprocesses; a fork that inherits a not-yet-closed write fd on a script
# another test is about to exec is the ETXTBSY race that once forced serial
# runs. Each of the two suites answers it at its own root:
#   - unit tests (`src/`): every fork routes through one `SPAWN_LOCK` spawn
#     discipline (see `src/lib.rs` `test_support`).
#   - the consolidated `tests/integration` binary: yog is a plain library
#     there, so its `#[cfg(test)]` lock is absent and no test-side lock can
#     reach its forks — instead the write fd never exists in this process at
#     all (`support::write_executable`).
test:
	cargo test

TARPAULIN_PIN := 0.35.2

coverage:
	@have=$$(cargo tarpaulin --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$have" != "$(TARPAULIN_PIN)" ]; then \
	  echo "tarpaulin $(TARPAULIN_PIN) required (have: $${have:-none}); see tarpaulin.toml" >&2; \
	  echo "  cargo install cargo-tarpaulin --version $(TARPAULIN_PIN) --locked" >&2; \
	  exit 1; \
	fi
	cargo tarpaulin --fail-under 100 --skip-clean --engine llvm --out Stdout

# The complete static gate: the 300-line cap + clippy (reads Cargo.toml
# [lints]) + the ast-grep rules audit + the cargo-deny supply-chain audit. All
# are pinned so the gate is reproducible — ast-grep 0.44.1 (sgconfig.yml),
# cargo-deny 0.20.2 (deny.toml), toolchain 1.95.0 (rust-toolchain.toml). CI runs
# this exact target via `make ci`; there is no CI-only lint step to drift from.
# `line-cap` goes first: it is milliseconds, so a structural violation fails
# before the minute-scale tools start.
lint:
	$(MAKE) line-cap
	$(MAKE) beat-audit
	$(MAKE) leak-scan
	cargo clippy --all-targets -- -D warnings
	$(MAKE) rules-audit
	$(MAKE) deny

# The two mechanical shapes of a drive beat that proves nothing (bl-70b8): a
# beat that reaches only `pass` or only `fail`, so the outcome it exists to
# catch writes no verdict row at all; and an assertion whose whole pattern is
# one interpolation, which is `grep -q ""` — true of everything — the moment its
# subject is missing. Both directions, the discipline `leak-scan` and
# `rules-audit` already hold: the harness must be clean AND the script's own
# fixture must still be flagged, so an edited pattern that silently matches
# nothing cannot pass as a green check forever. Milliseconds, so it sits beside
# `line-cap` at the head of the gate.
beat-audit:
	@scripts/beat-audit.sh --self-test
	@scripts/beat-audit.sh

# The disclosure gate (bl-fd5a): no credential, routable address, home path,
# address, pasted dialogue, agent-session artifact or unreadable blob in the
# tree. Both directions in one target, the same discipline as `rules-audit` —
# the tree must be clean, AND the scanner's own fixtures must still fire, per
# RULE and per LINE, so an edited pattern that silently matches nothing cannot
# pass. `scripts/leak-rules.sh` is the one definition of what counts,
# `leak-scan.sh` runs it, and this is the door. It runs second in `lint`, right
# after `line-cap`: both are seconds against clippy/tarpaulin's minutes, and a
# leak should fail before a compile starts. It also runs from `scripts/pre-commit`
# BEFORE the verdict cache is consulted — the one gate step no stored verdict
# may skip (bl-167d).
#
# It reads INDEX BLOBS, not the worktree: `git checkout-index` materializes the
# index into a scratch tree and the scan reads that, so the bytes scanned are
# the bytes committed. Until bl-167d it enumerated `git ls-files` and grepped
# the WORKTREE files those names pointed at, which passed any leak that was
# staged and then overwritten with a clean copy on disk.
# Mint the wire's local CA and its server/client/window leaves (REMOTE §1.4,
# §8; bl-b6fa, amended bl-ae05). **Operator tooling, never an in-channel
# protocol** — but no longer a script: the recipe is `yog wire-certs`, because
# an installed binary has no repository to find a script in and the engine's own
# boot performs the same act on an unprovisioned box. One recipe, two callers.
# Nothing it writes is in the repo — `WIRE_DIR` is under the yog data root,
# beside the world. Refuses to overwrite; `FORCE=1` rotates, which distrusts
# every certificate already issued.
# `WIRE_LEAF=<common-name>` asks for the other act (REMOTE §8.2): one extra
# client leaf under that name, over the CA already here — the host half of
# provisioning an entry on a visiting box. The pair is then carried to that box
# by hand, which is §1.4 verbatim and forever.
# `WIRE_FOOT=1` beside it grades that leaf a FOOT (REMOTE §4.2): a tool host
# that may advertise, take invocations and complete them, and say nothing else
# to the boundary. Unset is operator grade — inherited from the environment like
# FORCE, so there is no word to mistype into a demotion.
# ALL SIX READINGS PASS THROUGH, so `make wire-certs FORCE=1` rotates and
# `make wire-certs WIRE_FOOT=1 WIRE_LEAF=x` grades a foot (bl-a0dd). Two of them
# reached the verb only by environment inheritance before, so the make-variable
# spelling this comment teaches was silently inert for exactly the two settings
# whose absence is a no-op: a refusal saying "Re-run with FORCE=1" answered the
# re-run with the same refusal.
#   make wire-certs
#   make wire-certs WIRE_HOST=engine.example.com WIRE_PORT=7737
#   make wire-certs WIRE_LEAF=phone
#   make wire-certs WIRE_LEAF=buildbox WIRE_FOOT=1
# The binary itself takes NO argv — every setting is an environment reading, so
# it is a prefix there and the verb refuses a trailing word rather than
# ignoring it:
#   WIRE_HOST=engine.example.com WIRE_PORT=7737 yog wire-certs
wire-certs:
	@WIRE_DIR="$(WIRE_DIR)" WIRE_HOST="$(WIRE_HOST)" WIRE_PORT="$(WIRE_PORT)" \
		WIRE_LEAF="$(WIRE_LEAF)" FORCE="$(FORCE)" WIRE_FOOT="$(WIRE_FOOT)" \
		cargo run --quiet -- wire-certs

# Regenerate the wire conformance corpus (REMOTE §3, bl-32cb) from the boundary
# itself. The corpus is committed under corpus/; a test verifies it on every
# run, so this target is only ever needed after a wire-visible change — and it
# REFUSES a shape that changed its fields while PROTOCOL (src/wire/hello.rs)
# stood still, which is the rule made mechanical.
corpus:
	@YOG_CORPUS_OUT="$(CURDIR)/corpus" cargo test --quiet --lib boundary::corpus::tests::gate -- --exact
	@echo "corpus: regenerated from the boundary"

leak-scan:
	@scripts/leak-scan.sh --self-test
	@scripts/leak-scan.sh

# Supply-chain audit (cargo-deny 0.20.2 — see deny.toml): licenses, advisories
# (yanked + the one ignored eframe-stack RUSTSEC — ttf-parser unmaintained),
# bans (openssl-sys / native-tls — rustls-only since the embedded brazen brought
# TLS in, §16.7 W10), and known-registry sources.
deny:
	cargo deny check

# Static audit of every ast-grep rule (rules/, pinned ast-grep 0.44.1 — see
# sgconfig.yml). Both directions: `src` must be clean (exit 0), and every
# deliberate violation in rules/fixtures must fire (scan exits non-zero) so a
# silently-broken rule cannot pass unnoticed. Rules: `unsafe` confined to
# cli_outbound/sys.rs, `Mutex`/`RwLock` confined to state.rs, no `Rc`/`RefCell`
# anywhere.
rules-audit:
	ast-grep scan src
	@if ast-grep scan rules/fixtures >/dev/null 2>&1; then \
	  echo "rules-audit: rules/fixtures was NOT flagged — a rule has regressed" >&2; \
	  exit 1; \
	fi
	@echo "rules-audit: src clean; fixtures flagged (all rules live)"

# The 300-line cap on source files (AGENTS.md, "Repo discipline"; docs and
# config are exempt). This target is the ONE definition of the cap and of what
# counts as a source file — the pre-commit hook and CI both call it, neither
# restates it.
#
# It scans the WHOLE TREE, not the staged diff. The hook used to walk
# `git diff --cached` only, which made the cap a sampling rather than an
# invariant: a file that crossed 300 lines and was never touched again was
# never looked at again, and `src/app/balls.rs` duly rode at 308 undetected
# until an unrelated task happened to edit it (bl-12dc). CI never checked the
# cap at all. The obvious objection to a whole-tree scan is latency and a
# pre-existing violation blocking an unrelated commit; both are answered by
# measurement rather than argument — the scan is 287 files in under half a
# second, against a gate that already runs clippy and tarpaulin (minutes), and
# the tree was audited clean
# to zero violations before this landed, so the stricter gate blocks nobody on
# arrival. bl-52f8 then swept the whole tree to a 200-line aspiration, so the
# 300 wall is a backstop rather than a target.
#
# The cap is a variable so the same target answers the other question AGENTS.md
# asks: `make line-cap LINE_CAP=199` lists the ≥200 pre-split band. That stays
# a hand-run view, never a gate — a large fraction of the tree sits in the
# band, and a warning firing on that is noise (and a gate there is only the cap
# moved to 200). The band is where
# the drift actually happens: one file reached 291 lines across three balls, no
# one of which did anything wrong — the same shape as `src/app/balls.rs`
# reaching 308. The hard cap catches the wall; the ≥250 rule
# is a design-time projection on the author about to add, which is why it is
# read on demand and not enforced.
#
# `git ls-files` reads the INDEX, so a staged addition is covered before it is
# ever committed, and a staged deletion is already gone. Offenders are reported
# ALL AT ONCE rather than one per run. The empty-set guard is the target's own
# negative check, in the spirit of `rules-audit`'s fixtures: a broken pattern or
# a wrong working directory would otherwise enumerate nothing and pass silently,
# which is the exact failure mode this target exists to end.
LINE_CAP := 300
LINE_CAP_EXEMPT := \.(md|txt|toml|yaml|yml|json|lock)$$|(^|/)(Makefile|LICENSE|\.gitignore|\.githooks/)

line-cap:
	@files=$$(git ls-files | grep -Ev '$(LINE_CAP_EXEMPT)' || true); \
	n=$$(printf '%s\n' "$$files" | grep -c . || true); \
	over=$$(printf '%s\n' "$$files" | while IFS= read -r f; do \
	    { [ -n "$$f" ] && [ -f "$$f" ]; } || continue; \
	    c=$$(wc -l < "$$f"); \
	    [ "$$c" -gt $(LINE_CAP) ] && printf '  %s: %s lines\n' "$$f" "$$c"; \
	    true; \
	  done); \
	if [ "$$n" -eq 0 ]; then \
	  echo "line-cap: enumerated 0 source files — the scan is broken, not the tree" >&2; \
	  exit 1; \
	fi; \
	if [ -n "$$over" ]; then \
	  echo "error: source files over the $(LINE_CAP)-line cap:" >&2; \
	  printf '%s\n' "$$over" >&2; \
	  echo "       split along a real seam (DESIGN §12) — do not shave lines." >&2; \
	  exit 1; \
	fi; \
	echo "line-cap: $$n source files, all within $(LINE_CAP) lines"

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

# The complete gate, and the exact target CI runs (`ci`). Coverage goes through
# `scripts/check-coverage.sh` rather than the bare `coverage` target so the
# pre-commit hook, `make check` and CI share ONE coverage step: the held-and-
# replayed output and the signaled-tarpaulin retry (bl-673a) belonged to the
# hook alone while this line named `coverage`, so a runner-side kill reddened a
# release-PR CI run where the same kill was survivable at close. `make coverage`
# stays the bare, always-verbose invocation for a hand-run.
check: fmt-check lint
	@scripts/check-coverage.sh

# Arm this clone's git hooks: one symlink per file in .githooks/, seated in the
# repo's own hooks directory. Symlinks, not copies, so an updated hook is live
# without a re-run — and so a hook can resolve the repo from its own path.
#
# NOT `core.hooksPath`, which this machine sets GLOBALLY to a chain hook whose
# documented second job is to exec `<git-common-dir>/hooks/<name>`. Pointing
# core.hooksPath at .githooks per-repo would silence that machine-wide hook for
# this repo; seating the links where it looks keeps both.
#
# Refused from a linked worktree: `bl claim` deletes those, and links pointing
# into one would rot the moment the ball closed.
install-hooks:
	@top=$$(git rev-parse --path-format=absolute --show-toplevel) && \
	common=$$(git rev-parse --path-format=absolute --git-common-dir) && \
	if [ "$$common" != "$$top/.git" ]; then \
	  echo "install-hooks: run this in the main checkout, not a linked worktree" >&2; \
	  exit 1; \
	fi; \
	mkdir -p "$$common/hooks"; \
	for h in .githooks/*; do \
	  ln -sfn "$$top/$$h" "$$common/hooks/$${h#.githooks/}"; \
	done; \
	echo "hooks: $$common/hooks/{$$(ls .githooks | tr '\n' ',' | sed 's/,$$//')} -> $$top/.githooks"

# The binary is a live spawn target (the substrate shims re-exec it, and a
# running engine's own children resolve it mid-install), so it gets rename(2)
# atomicity: `install` writes a temp name in the SAME directory, then `mv -f`
# swaps it into place — a concurrent spawn sees whole-old or whole-new, never
# ENOENT from install(1)'s unlink-then-write window.
install: release
	@mkdir -p "$(INSTALL_BIN)"
	@install -m 0755 $(CARGO_TARGET_DIR)/release/yog "$(INSTALL_BIN)/.yog.tmp" && \
	  mv -f "$(INSTALL_BIN)/.yog.tmp" "$(INSTALL_BIN)/yog"
	@git rev-parse HEAD >"$(INSTALL_STAMP)" 2>/dev/null || rm -f "$(INSTALL_STAMP)"
	@echo "installed $(INSTALL_BIN)/yog"
	@echo "note: no substrate to install — litany, balls and brazen are compiled in (DESIGN §16.7)."

# --- fixture worlds (bl-8741) -----------------------------------------------
# The one-command door over `yog fixture`: lay a named, deterministic world
# state, boot an engine on it, print the address a harness dials, and remove
# both on the way out.
#
#   make fixture                 list the states
#   make fixture STATE=busy      lay it and serve it until Ctrl-C
#
# It depends on `release` and prefixes this checkout's build onto PATH, for
# `drive`'s reason: a run must prove the binary in hand rather than whatever
# happens to be installed. `FIXTURE_ROOT`, `WIRE_HOST` and `WIRE_PORT` are the
# verb's own environment readings and pass through untouched.
#
# **A harness in another repository should spend the VERB, not this target.**
# `yog fixture <state>` answers one JSON object and exits; the consumer boots
# `XDG_DATA_HOME=<root> yog` itself, because the consumer is the one that has
# to kill it. That is also why there is no `make` variable here for anything
# the verb already reads from the environment.
fixture: release
	@PATH="$(CARGO_TARGET_DIR)/release:$$PATH" scripts/fixture.sh $(STATE)

# --- the real-substrate drive (STORIES.md, "Real-substrate drive") ----------
# The second half of the done-bar, as make verbs. Each run gets its own scratch
# world under $(DRIVE_ROOT), and `drive.sh` refuses outright if that root and
# the live `$XDG_DATA_HOME` overlap in either direction, because a run wipes its
# world before it starts.
#
# The logic lives in `scripts/drive/drive.sh`, not here: it preflights the host,
# prefixes this checkout's `target/release` onto PATH (a run boots `yog` from
# PATH, so the prefix is what proves the build in hand rather than whatever is
# installed), gives each run verb its own world, and emits the log skeleton from
# the verdict rows. These targets are the door, not the machinery.
#
#   make drive                           the whole ladder
#   make drive DRIVE_RUNS="run-headless"  a subset
#   make drive-cleanroom [DRIVE_VERB=run-headless]
#   make drive-preflight                 name every missing host tool at once
#   make drive-seed                      a scratch world, path on stdout
#   make drive-log [DRIVE_LOG_DIR=...]   re-emit a run's log skeleton
#
# DRIVE_ROOT is deliberately NOT defaulted here — the default is drive.sh's, so
# there is one home for it; an override from the environment or the command line
# still reaches the script.
DRIVE      := scripts/drive/drive.sh
DRIVE_VERB ?= run-headless

drive-preflight:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) preflight

drive: release
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) ladder $(DRIVE_RUNS)

drive-cleanroom: release
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) cleanroom $(DRIVE_VERB)

drive-seed:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) seed

drive-log:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) log $(DRIVE_LOG_DIR)

# Seat the deployment on a server: build the image here, carry it over the ssh
# channel, and point the `yog` user service at that exact tag (bl-bf35,
# containerized in bl-b973, reconciled with this tree in bl-c6e2). HOST is an
# ssh destination and the only parameter — no machine is named in this tree.
#
#   make deploy HOST=myserver
#
# This is NOT `make install`. That target builds and seats a binary from this
# checkout for the operator's own box; a server takes the image, which is the
# unit of install and carries its own toolchain pin and its own disclosure gate.
#
# **This target seats a RECONCILER, and this comment used to say there is
# none** (bl-4e3c). The binary-era one is still retired and still wrong for the
# reasons that retired it — it reconciled a cargo-installed BINARY against the
# crates.io index and read quiescence off the unit's cgroup, which under a
# container unit answers about `docker run`, a client process. What replaced it
# polls the §10.1 registry for RELEASED version tags only and asks the engine
# itself, over the §8.5 control boundary, whether a turn is in flight. So this
# target is the bootstrap, the dev-build path and the emergency path — a human
# carrying an unreleased tag over ssh — and released versions arrive on their
# own. `scripts/deploy/reconcile.sh` and DESIGN §10.1 carry the reasoning.
deploy:
	@[ -n "$(HOST)" ] || { echo "usage: make deploy HOST=<ssh-host>" >&2; exit 2; }
	@scripts/deploy/seat.sh "$(HOST)"

# What that server is running right now: the unit, and the immutable tag it was
# pointed at — the box's own answer to "what version is this".
#
# Since bl-4e3c it also answers "and is it still upgrading itself": the timer's
# next fire, and any `YOG_REFUSED` line, which is the one state that makes a box
# stop moving forward on purpose. A refusal that only a `journalctl` reader ever
# sees is a refusal nobody sees.
deploy-status:
	@[ -n "$(HOST)" ] || { echo "usage: make deploy-status HOST=<ssh-host>" >&2; exit 2; }
	@ssh "$(HOST)" 'systemctl --user --no-pager --lines=0 status yog.service; \
	  echo; grep -E "^YOG_(IMAGE|REFUSED)=" "$$HOME/.config/yog/deploy.env" 2>/dev/null; \
	  echo; systemctl --user --no-pager list-timers yog-reconcile.timer 2>/dev/null; \
	  echo; journalctl --user -u yog.service --no-pager -n 15'

uninstall:
	@rm -f "$(INSTALL_BIN)/yog" "$(INSTALL_STAMP)"
	@echo "removed $(INSTALL_BIN)/yog"

# The stamp path as a value. scripts/install-main asks for it rather than
# restating $(INSTALL_BIN), so "where the installed commit is recorded" has one
# home — and an `INSTALL_PREFIX=` override moves the stamp and the check
# together (bl-6ff1).
print-install-stamp:
	@echo '$(INSTALL_STAMP)'

ci: check

# Publish to crates.io. The dry-run always runs; the real upload runs ONLY
# with an explicit CONFIRM=yes. Publishing 0.0.1 is a deliberate human
# decision, never a side effect of running this target.
#   make publish              # dry-run only
#   make publish CONFIRM=yes  # dry-run, then real publish
publish:
	cargo publish --dry-run --locked
ifeq ($(CONFIRM),yes)
	cargo publish --locked
else
	@echo "dry-run passed. Publishing is an explicit decision: re-run with 'make publish CONFIRM=yes'."
endif

# The OCI image — the unit of install for a box that takes containers rather
# than binaries. `Containerfile` is the whole of what it builds and states why
# each layer is what it is.
#
# The version is READ FROM Cargo.toml and never typed here: the crate version
# has one home, and a tag typed into a Makefile is that fact stored twice. The
# `sed` range restricts to `[package]` so a dependency's own `version =` cannot
# be picked up. Both `:<version>` and `:latest` are applied to the same build.
#
# Podman or docker, whichever the box has, podman first — it needs no daemon
# and no group membership, which is the difference between "the operator can
# build this" and "the operator can build this after an admin says yes".
# Override with `make image ENGINE=docker`.
#
# IT PUSHES NOTHING, and there is no `push` target to forget to guard. The
# registry is named — `ghcr.io/mudbungie/yog`, one package per repo, pushed
# only from that repo's release workflow at tag time (DESIGN §10.1, operator
# ruling 2026-08-30) — and the push still does not live here. A push is not
# undoable: a tag can move, but the bytes anyone pulled are theirs. What
# publishes is the version tag and the manifest digest, both immutable, and
# never a moving `latest`; the `:latest` applied below is LOCAL, a convenience
# on one box nobody else can pull.
IMAGE_NAME    ?= yog
IMAGE_VERSION := $(shell sed -n '/^\[package\]/,/^\[/{s/^version *= *"\([^"]*\)".*/\1/p;}' Cargo.toml)
IMAGE_TAG     := $(IMAGE_NAME):$(IMAGE_VERSION)
ENGINE ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null)

image:
	@test -n "$(ENGINE)" || { echo "image: no podman and no docker on PATH" >&2; exit 1; }
	@test -n "$(IMAGE_VERSION)" || { echo "image: no version in Cargo.toml" >&2; exit 1; }
	@echo "image: $(notdir $(ENGINE)) build -> $(IMAGE_TAG)"
	@$(ENGINE) build -f Containerfile \
	  -t "$(IMAGE_TAG)" -t "$(IMAGE_NAME):latest" .
	@$(ENGINE) image inspect "$(IMAGE_TAG)" \
	  --format 'image: {{.Id}} {{.Size}} bytes'
	@$(MAKE) --no-print-directory image-scan

# The image-side disclosure gate — DESIGN §10.1's condition on the registry
# ruling, and the check nothing in this repo previously performed. `leak-scan`
# reads the git INDEX; an image is built from inputs no commit has — the build
# context as the engine actually receives it, the base layers, the package
# index, and the image CONFIG. yog's is the image whose trust root matters
# most: the Containerfile promises no CA and no leaf in any layer (REMOTE §1.4),
# and until this target nothing read the layers to check.
#
# It is a step of `image` and not a target beside it, for the reason the
# pre-commit hook is not a target beside `commit`: a gate a person has to
# remember to run is not a gate. Run it alone to re-judge an image already
# built. `scripts/image-scan.sh` states what it scans and how it isolates the
# authored content; this target only decides which tag and runs BOTH
# directions — the planted-secret self-test first, because a scan that has
# stopped matching passes everything forever, then the real image.
#
# NOT part of `check`. `check` must run on a box with no container engine and
# must not depend on an artifact a build step produced; this needs both. It is
# the image's gate, and it runs where the image is made.
image-scan:
	@test -n "$(ENGINE)" || { echo "image-scan: no podman and no docker on PATH" >&2; exit 1; }
	@test -n "$(IMAGE_VERSION)" || { echo "image-scan: no version in Cargo.toml" >&2; exit 1; }
	@ENGINE=$(ENGINE) scripts/image-scan.sh --self-test "$(IMAGE_TAG)"
	@ENGINE=$(ENGINE) scripts/image-scan.sh "$(IMAGE_TAG)"

clean:
	cargo clean
