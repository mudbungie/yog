.PHONY: all build release test coverage lint fmt fmt-check check run ux reload icon icon-seats install-hooks install uninstall print-install-stamp ci publish clean rules-audit line-cap leak-scan deny \
        drive drive-preflight drive-cleanroom drive-seat drive-unseat drive-seed drive-log

# Install location for `make install`. Defaults to the XDG-ish user-local
# convention; override for system-wide installs or packaging:
#   make install INSTALL_PREFIX=/usr/local
INSTALL_PREFIX ?= $(HOME)/.local
INSTALL_BIN    := $(INSTALL_PREFIX)/bin
# The freedesktop seats for the launcher entry and its icon. `Icon=yog` in the
# .desktop resolves by NAME through the hicolor theme, so the SVG's basename
# and the entry's `Icon=` must agree — as must `StartupWMClass` and the
# window's app_id (src/main.rs), or a running window gets a generic icon.
INSTALL_APPS   := $(INSTALL_PREFIX)/share/applications
INSTALL_THEME  := $(INSTALL_PREFIX)/share/icons/hicolor
INSTALL_ICONS  := $(INSTALL_THEME)/scalable/apps
# The commit the installed binary was built from — the ONE record of "what is
# installed", written by `install` (the act that makes it true, whoever ran it)
# and read by scripts/install-main to decide whether main's tip needs building
# at all (bl-6ff1). Beside the binary, so it moves with INSTALL_PREFIX and a
# second prefix cannot inherit the first one's answer.
INSTALL_STAMP  := $(INSTALL_BIN)/.yog.commit
# The fixed sizes laid beside the scalable SVG, for the shells that will not
# read one. Mirrors theme::icon::PNG_SIZES; `make icon` emits exactly these.
ICON_SIZES     := 16 32 48 64 128 256

# Build output root. Defaults to `target`; the local CICD script
# (scripts/install-main) overrides it to the repo's own target/ while
# building from an ephemeral worktree, so the release build stays incremental.
# Exported so the cargo invocations below honor it too.
CARGO_TARGET_DIR ?= target
export CARGO_TARGET_DIR

# The runtime world `ux`/`reload` act on: the SAME root yog itself resolves
# for `$XDG_DATA_HOME/yog` (`yog_data_root`, src/xdg/mod.rs — the env var if
# set, else `~/.local/share`). Both targets record the pid of the instance
# THEY launched in YOG_PIDFILE and kill only that recorded pid — never a bare
# `pkill -x yog`/`killall yog`, which matches every yog process on the box
# regardless of which world it runs against: other operators, UAT drives on
# isolated seats, scratch-world instances all share the image name (bl-6260 —
# a concurrent fleet's `make ux` loop pkilled three live UAT instances on an
# isolated seat with zero stderr).
YOG_DATA_HOME := $(or $(XDG_DATA_HOME),$(HOME)/.local/share)/yog
YOG_PIDFILE   := $(YOG_DATA_HOME)/yog.pid

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
	$(MAKE) leak-scan
	cargo clippy --all-targets -- -D warnings
	$(MAKE) rules-audit
	$(MAKE) deny

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
# a hand-run view, never a gate — 74 of 339 files sit in the band, and a warning
# firing on a fifth of the tree is noise (and a gate there is only the cap moved
# to 200). The band is where
# the drift actually happens: `src/theme/mod.rs` reached 291 across three balls
# (bl-ae05, bl-4305, bl-51cb), no one of which did anything wrong — the same
# shape as balls.rs reaching 308. The hard cap catches the wall; the ≥250 rule
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

check: fmt-check lint coverage

# Launch the window over every enumerated workspace, ON MAIN'S TIP, and keep it
# there: when a merge lands on main the window closes and reopens on a binary
# built from the new tip, with no operator verb in between (bl-d4f0). Optionally
# preset the startup focus on one workspace with WS=:
#   make run
#   make run WS=/path/to/workspace
#
# It launches the INSTALLED binary, not `cargo run`: the installed artefact is
# the thing `scripts/install-main` already converges onto refs/heads/main on
# every ref move, so tracking main here is a WATCH of that one fact (the
# `INSTALL_STAMP` commit) rather than a second build path with its own answer.
# The machinery is scripts/run-main; this is the door. Consequences worth
# knowing before you reach for it:
#   - it is main, not your working tree. To look at uncommitted work use
#     `make ux` (release, your tree) or `cargo run` (debug, your tree).
#   - it is a RELEASE build and it installs, so an unrelated `yog` you launch
#     from the shell or the desktop entry is upgraded by the same convergence.
#
# `icon-seats` is a prerequisite because on Wayland the compositor resolves the
# window's icon through the INSTALLED hicolor seats (app_id -> yog.desktop ->
# Icon=yog), ignoring the binary's embedded icon entirely — a rebuilt binary
# alone can never refresh the mark, so the launch verb refreshes the seats its
# own window will resolve through (bl-121d).
run: icon-seats
	@scripts/run-main "$(INSTALL_BIN)/yog" "$(INSTALL_STAMP)" "$(WS)"

# Re-emit every icon artifact from the generator (DESIGN §11): the scalable SVG
# and one PNG per size in ICON_SIZES. All of them are derivations, never
# hand-edits; tests assert each checked-in file still equals what `theme::icon`
# produces, and this is the only sanctioned way to move them.
icon:
	@cargo run --quiet --example icon -- assets
	@echo "re-emitted assets/yog.svg + PNGs at $(ICON_SIZES) from theme::icon"

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

# The freedesktop icon seats, as ONE target both `install` and `run` depend on
# (DESIGN §11, bl-121d). Three staleness traps live here; each line answers one:
#   - orphaned sizes: a seat laid when ICON_SIZES held a size later dropped is
#     never overwritten (or uninstalled) again — so every sized seat is swept
#     before the current set is laid, making orphans impossible.
#   - a stale icon-theme.cache: GTK judges a cache at the theme root valid
#     while its mtime is >= the toplevel dir's, and installing into existing
#     SUBdirs never bumps the toplevel — a third-party gtk-update-icon-cache
#     run (a Steam/Chrome shortcut installer) then serves its old index
#     forever. Rebuilding the cache makes the fresh files authoritative.
#   - a running shell: GtkIconTheme rescans only when the toplevel theme-dir
#     mtime moves, so a live GNOME session keeps its old texture until it
#     does. The cache rebuild writes a new file into that dir (bumping it);
#     the `touch` fallback bumps it when the tool is absent.
icon-seats:
	@mkdir -p "$(INSTALL_APPS)" "$(INSTALL_ICONS)"
	@rm -f "$(INSTALL_THEME)"/*/apps/yog.png
	@install -m 0644 assets/yog.svg "$(INSTALL_ICONS)/yog.svg"
	@for n in $(ICON_SIZES); do \
	  seat="$(INSTALL_THEME)/$${n}x$${n}/apps"; \
	  mkdir -p "$$seat"; \
	  install -m 0644 "assets/yog-$$n.png" "$$seat/yog.png"; \
	done
	@install -m 0644 assets/yog.desktop "$(INSTALL_APPS)/yog.desktop"
	@gtk-update-icon-cache -f -t "$(INSTALL_THEME)" 2>/dev/null \
	  || touch "$(INSTALL_THEME)" 2>/dev/null || true

# The binary is a live spawn target (lernie/bl/a running instance's `reload`
# exec it mid-install), so it gets rename(2) atomicity: `install` writes a temp
# name in the SAME directory, then `mv -f` swaps it into place — a concurrent
# spawn sees whole-old or whole-new, never ENOENT from install(1)'s
# unlink-then-write window. The icon seats are never spawned, so they keep the
# plain `install` recipes in `icon-seats`.
install: release icon-seats
	@mkdir -p "$(INSTALL_BIN)"
	@install -m 0755 $(CARGO_TARGET_DIR)/release/yog "$(INSTALL_BIN)/.yog.tmp" && \
	  mv -f "$(INSTALL_BIN)/.yog.tmp" "$(INSTALL_BIN)/yog"
	@git rev-parse HEAD >"$(INSTALL_STAMP)" 2>/dev/null || rm -f "$(INSTALL_STAMP)"
	@echo "installed $(INSTALL_BIN)/yog"
	@echo "installed $(INSTALL_APPS)/yog.desktop + the scalable SVG and PNGs at $(ICON_SIZES) under $(INSTALL_THEME)"
	@echo "note: no substrate to install — lernie, balls and brazen are compiled in (DESIGN §16.7)."

# The UX-testing loop: one command between landing a change and looking at it.
# Rebuilds + installs the RELEASE binary and restarts it on LIVE state — the
# real `$XDG_DATA_HOME/yog` world, real conversations, no env overrides. That is
# the artefact an operator actually launches. It is the WORKING-TREE verb, and
# that is the whole difference from `make run`, which builds and relaunches
# refs/heads/main: use `ux` to look at work you have not landed. The two share
# one binary path, so whichever ran last is what is installed. Safe to hammer:
# it kills only the pid IT recorded last time
# (YOG_PIDFILE, guarded below), so repeat invocations are idempotent — but it
# is NOT safe to run against a box with other yog instances up under a
# DIFFERENT `$XDG_DATA_HOME` (another operator, a UAT seat): those are simply
# untouched, never matched, never killed.
#
# `install` is a PREREQUISITE, not a recipe line: make runs it to completion
# first and skips this target entirely if the build fails (`-k` too — it refuses
# to remake a target whose prerequisite errored). A broken build therefore never
# reaches the kill, so your running instance survives it, and the old window
# stays usable for the whole compile.
#
# It BLOCKS rather than detaching: yog's stderr goes straight to your
# terminal, which is most of the diagnostic value during a UX pass. Ctrl-C ends
# the session; re-run to iterate. The pid is captured by having a tiny `sh -c`
# record its OWN pid to YOG_PIDFILE and then `exec` into yog — `exec` replaces
# the shell's process image in place, so the recorded pid IS yog's real pid,
# with no background/`wait` needed to keep the foreground/Ctrl-C behavior
# identical to a plain `"$(INSTALL_BIN)/yog"` line.
ux: install
	@mkdir -p "$(YOG_DATA_HOME)"
	@pid=$$(cat "$(YOG_PIDFILE)" 2>/dev/null); \
	if [ -n "$$pid" ] && [ "$$(ps -p "$$pid" -o comm= 2>/dev/null)" = "yog" ]; then \
	  kill "$$pid" 2>/dev/null; \
	fi
	@sh -c 'echo $$$$ >"$(YOG_PIDFILE)"; exec "$(INSTALL_BIN)/yog"'

# The auto-reload half of CICD delivery (scripts/install-main calls this
# after a background `make install` lands a main merge). Unlike `ux` it does
# NOT rebuild (the caller already did) and does NOT block: it only relaunches
# an instance that was already running, detached, logging to
# `$(CARGO_TARGET_DIR)/yog.log`. If yog wasn't running, a landed merge stays
# quiet rather than surprise-launching a window nobody asked for. "Was already
# running" means the pid recorded in YOG_PIDFILE (this world's own instance),
# never a bare `pkill -x yog` scan of the whole box — a landed merge on this
# checkout must never reach into another operator's or another world's yog.
reload:
	@mkdir -p "$(YOG_DATA_HOME)"
	@pid=$$(cat "$(YOG_PIDFILE)" 2>/dev/null); \
	if [ -n "$$pid" ] && [ "$$(ps -p "$$pid" -o comm= 2>/dev/null)" = "yog" ]; then \
	  kill "$$pid" 2>/dev/null; \
	  echo "reload: yog was running, relaunching on the freshly installed binary"; \
	  mkdir -p "$(CARGO_TARGET_DIR)"; \
	  nohup sh -c 'echo $$$$ >"$(YOG_PIDFILE)"; exec "$(INSTALL_BIN)/yog"' >>"$(CARGO_TARGET_DIR)/yog.log" 2>&1 & \
	else \
	  rm -f "$(YOG_PIDFILE)"; \
	  echo "reload: yog was not running, nothing to relaunch"; \
	fi

# --- the real-substrate drive (STORIES.md, "Real-substrate drive") ----------
# The second half of the done-bar, as make verbs. `make ux` and `make reload`
# above are the operator's LIVE-world verbs by design (bl-6260); every target
# below is the opposite by design — each run gets its own scratch world under
# $(DRIVE_ROOT), and `drive.sh` refuses outright if that root and the live
# `$XDG_DATA_HOME` overlap in either direction, because a run wipes its world
# before it starts.
#
# The logic lives in `scripts/drive/drive.sh`, not here: it preflights the host,
# prefixes this checkout's `target/release` onto PATH (yogdrive.sh launches
# `yog` from PATH, so the prefix is what proves the build in hand rather than
# whatever is installed), gives each run verb its own world, and emits the log
# skeleton from the verdict rows. These targets are the door, not the machinery.
#
#   make drive                          the whole ladder, four worlds
#   make drive DRIVE_RUNS="run run-s7"  a subset
#   make drive-cleanroom [DRIVE_VERB=run-s3s4s6]
#   make drive-preflight                name every missing host tool at once
#   export YOG_SEAT=$$(make -s drive-seat)   ... then `make drive-unseat`
#   make drive-seed                     a scratch world, path on stdout
#   make drive-log [DRIVE_LOG_DIR=...]  re-emit a run's log skeleton
#
# DRIVE_ROOT is deliberately NOT defaulted here — the default is drive.sh's, so
# there is one home for it; an override from the environment or the command line
# still reaches the script.
DRIVE      := scripts/drive/drive.sh
DRIVE_VERB ?= run

drive-preflight:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) preflight

drive: release
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) ladder $(DRIVE_RUNS)

drive-cleanroom: release
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) cleanroom $(DRIVE_VERB)

drive-seat:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) seat

drive-unseat:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) unseat

drive-seed:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) seed

drive-log:
	@DRIVE_ROOT="$(DRIVE_ROOT)" $(DRIVE) log $(DRIVE_LOG_DIR)

uninstall:
	@rm -f "$(INSTALL_BIN)/yog" "$(INSTALL_STAMP)" "$(INSTALL_APPS)/yog.desktop" "$(INSTALL_ICONS)/yog.svg"
	@rm -f "$(INSTALL_THEME)"/*/apps/yog.png
	@gtk-update-icon-cache -f -t "$(INSTALL_THEME)" 2>/dev/null \
	  || touch "$(INSTALL_THEME)" 2>/dev/null || true
	@echo "removed $(INSTALL_BIN)/yog + the desktop entry and every icon (any size)"

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

clean:
	cargo clean
