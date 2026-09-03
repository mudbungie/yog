# yog

yog is the **server** for [litany](https://github.com/mudbungie/litany) loops:
it holds every litany workspace, derives what needs attention, and drives the
loop lifecycle — start work from nothing, a path, or a ball, message or stop
agents, assign and close balls — over the litany and balls substrates it
embeds. It has **no interface of its own**: every read and every act crosses
one control boundary, over an mTLS wire or a deposit in its own inbox, and a
**seat** on the far side decides what a human sees. The desktop window is
[lernie](https://github.com/mudbungie/lernie), its own package.

It is its own package ([mudbungie/yog](https://github.com/mudbungie/yog)),
deliberately outside the litany and balls workspaces: both ship as composable
components and yog composes on top of them. The batteries-included direction is
**landed**: balls, brazen and litany are all linked crates, exact-pinned in
`Cargo.toml` — which is the pin authority, so no version is restated here or in
any doc. Ball reads run in-process, and every substrate spawn targets yog's own
executable under a verb namespace — `yog bl <verb…>`, `yog bz <args…>`, `yog
litany <argv…>` — dispatched to the embedded crate exactly as each upstream's
own thin binary does (DESIGN §16.7). Nothing is installed alongside: there is no
host `bl`, `bz` or `litany` in the chain, which is what `make drive-cleanroom`
proves by putting only `yog` and `git` on `PATH`. litany, balls and brazen take
no dependency back.

## Architecture

**`docs/DESIGN.md` is the authority** — the state inventory, the attention
model, the write paths and the module map live there. It is
the *architecture* authority, not a generated mirror: when code and DESIGN
disagree, one of them is a bug, and the discipline is to fix the doc rather than
code around it (AGENTS.md). Two guards hold it to the tree: its `§`-citations
(`tests/design_citations.rs` proves every cited section resolves to a real
heading) and its module map (`tests/design_module_map.rs` proves every source
file has a row, every row names a live path, and every rule §12 states about
its own table). The 300-line cap is `make line-cap`'s, and lives nowhere else.
This section is only the map.

yog is a stateless derivation: every answer is a pure function of on-disk state
at that tick, so two instances against the same repos converge with no
coordination. The durable, yog-owned state is a closed list — **DESIGN §5.2 is
the normative one** — and it is short: `ui.json` (pins, collapse overrides,
attention watermarks — §4.1), the `ops.jsonl` action log (§4.2), `cadence.yaml`
(the clock's periods and the arming entries for the monitor and the loop, §7.2 —
present only once you tune or arm something, and deleting it *is* the reset),
the alignment monitor's policy file that a `cadence.yaml` entry names
(`monitor.md` by default, present only while armed), and one stderr sink per
detached spawn (`<yog-state>/detached/<ts>-<workspace leaf>.err`, §8.1/§13.3),
each written by the detached child itself and projected into its ops row at read
time. Everything else is derived from disk. The code is split the house way: pure
view-model modules — the per-tick `git_tree`, the `nav` roster, `attention`,
`projects`/`binding`, `start`, `ui_state`, the inspector projections
(`transcript`, `steps_view`, `inboxview`, `budgets`) and `config_edit` — each
with its own `wire` module, which is how that projection is said over the
boundary. There is no glue in front of them: a seat is a separate program.

The **congeries palette**, the application mark and the three information
altitudes are the seat's (`lernie`); DESIGN §11 is retired and says so. What is
left of that vocabulary here is what a derived row *says in words* — the badge
phrases in `src/badge.rs` — because the words are content and cross the
boundary inside the row, while the hues were always a statement about paint.

Two composite flows sit on top of the derivations: the **start flow**
(DESIGN §3.4) — a prompt lands as a new root in the named workspace (a fresh
world bootstraps one automatically; creating more is a deliberate act, walling
off spheres like clients or corporate vs. personal) and may carry a path or a
ball payload (`bl create` + `bl claim --as <workspace-name>`), fired as a
detached `litany prompt`. A start that binds a work target also **freezes that
project's instruction files** into the agent's first commit (DESIGN §3.7): yog
walks from the target's git root down to the target, pins each `AGENTS.md` it
finds as exact bytes before the first inference, and authors the workspace's
`config/default` so the worker composes them — the filename set is `AGENTS.md`
unless a workspace's `instructions.yaml` says otherwise, and the goal you typed
stays your payload, never a concatenation. Then the **config editors**
stage-and-validate brazen `config.toml`, litany global config, and
per-workspace config branches (DESIGN §9). Replays
(`<litany-data>/replays/*`) are enumerated through the same derivation,
read-only.

## Running

```
yog                    # boot the engine and park until a signal
yog gesture '/attention'   # ask the boundary a question from this world
yog --help             # the whole surface
```

A bare `yog` **is** the engine: it enumerates the world, derives it, answers
the gestures inbox and listens on the wire, and parks until `SIGTERM`. There is
no verb to select it, because there is one face (it was `yog serve` while a
window stood beside it). A seat — the `lernie` window, an android client,
`yog gesture` from an agent's own bash — is a client of that boundary.

yog enumerates every litany workspace (named workspaces under
`<yog-data-root>/workspaces/` — the resolved root is spelled out under "The
world" below — foreign workspaces and replays under the
litany data root). A workspace
name is **chosen** — stated by the operator at a raise — or the fixed `home`
the empty-world bootstrap uses without asking; it is never minted
(DESIGN §3.1). Minting is what names a *conversation*, from an embedded
wordlist (§3.3). Balls bind to workspaces by claimant: every claim a workspace
makes is stamped `--as` that chosen name, so assignment is late-mutable
metadata, not a location (DESIGN §3.2). Which workspace a seat is
looking at is the seat's own state and rides in the gesture; the engine keeps
no focus. There is no `--repo` flag — the whole roster is the answer.

`make install` builds and seats the binary from your checkout; a server runs
the image under its own unit (`make deploy HOST=<ssh-host>`). The binary links
no display stack and needs no `apt install` step.

**No substrate needs installing.** litany, balls and brazen are compiled in, and
every `litany` / `bl` / `bz` yog runs is yog re-execing itself under that verb
namespace (DESIGN §16.7). The `LITANY_BINARY` / `BL_BINARY` / `BZ_BINARY` env
vars still override the physical target, as test seams and escape hatches back
to a host binary.

## The world

yog composes its own **nested world** — a substrate environment under
`<yog-data-root>/world` that overrides `LITANY_HOME`, `XDG_STATE_HOME`, and
`PATH` and hands it to every child it spawns, so yog's `bl`/`litany` state never
collides with your ambient tools. The world is the authority in **DESIGN §16**.

**The reset is one `rm`, and the path it takes is resolved, not spelled.**
`<yog-data-root>` is `$XDG_DATA_HOME/yog` when `XDG_DATA_HOME` is set and
non-empty, else `$HOME/.local/share/yog` — the XDG fallback yog's own fold
applies (`src/xdg`), and the reason a literal `rm -rf $XDG_DATA_HOME/yog` is
wrong: with the optional variable unset that command names `/yog`, and unquoted
it splits on any space in the value. Ask yog for the resolved value rather than
re-deriving it — `yog env` prints the world's roots shell-quoted, and
`LITANY_HOME` is `<yog-data-root>/world/litany`:

```
rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/yog"
```

**Scope of that deletion, exactly:** everything yog owns and nothing else — the
world subtree (the nested litany home, the nested `XDG_STATE_HOME` holding
balls' state and yog's own durables, and the generated `world/tools/` shims),
every workspace yog created under `<yog-data-root>/workspaces/`, and every
per-workspace wall with its brazen config, credentials and model cache. Your
**ambient** litany, balls and brazen state is untouched — it lives under your
real `$XDG_*` roots, which the world never writes to. Foreign workspaces and
replays are untouched too: they live under the litany data root, outside this
path. There is no undo and no reset verb; the deletion is the reset, which is
the severability the nesting was chosen for.

**Nothing brazen-shaped is shared** (DESIGN §16.2). A workspace is an app-wide blast radius, and provider rows are
credential-adjacent workspace settings, so brazen's config, the credentials it
points at and the model cache beside them all resolve inside the focused
workspace's **wall**:

```
<yog-data-root>/world/walls/<workspace>/brazen/config.toml
<yog-data-root>/world/walls/<workspace>/brazen/credentials/<provider>.json
<yog-data-root>/world/walls/<workspace>/brazen/models/<provider>.json
```

Your machine's own brazen state is never read. A wall is **born empty** — it is
keyed by the name the workspace is created under, so nothing can seed it
earlier —
so a newborn workspace answers brazen's shipped provider rows and nothing else;
custom rows and sign-ins are per-workspace acts you perform after birth, which
is what keeps a corporate sphere's logins from shining into a personal one. A
seat inside *no* workspace has no wall, so `yog bz …` there refuses (exit 64)
rather than reaching for machine state — including a shell the escape hatches
below dropped into, since they hand out the world, and the world names no
sphere.

The `PATH` override fronts `world/tools/`, where yog seeds a **shim per
namespace** — `bl`, `litany`, `bz`, balls' two plugin siblings `bl-delivery` and
`bl-tracker`, and the capability control — each a one-line re-exec of yog itself
(DESIGN §16.4, §16.7). So an agent yog starts gets yog's own compiled-in balls
when it types `bl` — same implementation, same nested state — and its claims are
stamped `--as $YOG_NAME` without the prompt telling it to. **Every verb works,
`bl prime` included**: the plugin-sibling shims are exactly what lets a checkout
primed by the embedded `bl` run a plugin chain that is yog at every hop, so the
old "these verbs are refused, run a host `bl` by absolute path" carve-out is
gone. The shims are generated artifacts — yog rewrites any that drift, and
deleting the directory loses nothing.

Two escape hatches (DESIGN §8.4) let a human — or a foreign frontend — join that
world from a shell with one prefix:

```
eval "$(yog env)"          # drop THIS shell into the world; `bl`/`litany`/`bz`
                           # now resolve to the world's shims — yog itself,
                           # against yog's nested state
yog exec bl list           # run one command inside the world (its exit is yog's)
yog exec --cwd /path bl close bl-1234 --as me

# …and inside ONE workspace's wall, which is what anything brazen-shaped needs:
yog exec --ws /path/to/ws bz --login --provider openai --browser   # sign in
eval "$(yog env --ws /path/to/ws)"                                 # a whole shell
```

`yog env` prints the world's shell-quoted `export` lines; `yog exec [--cwd DIR]
[--ws WORKSPACE] <cmd…>` layers the world env over your inherited environment,
inherits stdio, and propagates the child's exit code (a terminating signal maps
to `128 + signum`).

**`--ws` names a workspace's wall.** Providers, sign-ins and the model cache
belong to a workspace, not to the machine — so `bz` outside one refuses rather
than falling back to anything shared, and `--ws` is how a headless seat says
which one. It is the same flag, spelled the same way, as `yog gesture --ws`. Both work headless — no display required.

Beyond the hatches, **every operator gesture crosses one control boundary**
(DESIGN §8.5, VISION §4.8). A gesture is a JSON envelope deposited create-only
into `<yog-state>/gestures/`; the running engine consumes it and writes
`gestures/replies/<id>.json`:

```
yog &                                   # the engine (worker + watcher + inbox + wire)
yog gesture '{"op":"scan","workspace":"/path/to/ws"}'      # deposit-and-wait sugar
yog gesture '{"op":"conversations","workspace":"/path"}'   # queries answer typed JSON
```

**Or type it.** The same gestures have a slash spelling — the line a human uses
at a terminal, a TUI, or a chat window, and the one a seat's composer reads
when a draft starts with `/` (type `/` alone for the roster, `//` to say a
literal slash). A line takes its unspoken targets from the seat it is typed at:
a seat's own selection, or these flags:

```
yog gesture --ws /path/to/ws '/scan'
yog gesture --ws /path/to/ws --agent c-1 '/message ship it'
yog gesture --project /path/to/repo --as cobalt '/close bl-1f2a'
yog gesture '/balls'
```

**What needs you is a queue, not a badge.** `/attention` is every conversation
waiting on somebody, anywhere, with why it is asking and what it last said.
`/seen` answers one: it writes the watermark a seat writes when you open that
conversation, and hands back the queue that remains, so an agent driving yog
closes one decision per gesture.

```
yog gesture '/attention'
yog gesture --ws /path/to/ws --agent c-1 '/message go with the second option'
yog gesture --ws /path/to/ws --agent c-1 '/seen'
```

**Every command answers `--help`**, and help is itself a gesture — a query,
asked *about* a command, so it is the same question wherever you type it:

```
yog --help                    # the whole surface: the engine, gestures, hatches, namespaces
yog help exec                 # one yog command's page
yog exec --help               # the same page, asked at the command
yog gesture --help            # every gesture, one line each
yog gesture --help close      # one command's page
yog gesture '/close --help'   # the same page, said as a line
```

At a seat, a draft of `/help`, `/help close`, `/close --help` or a bare `/`
answers identically. Help reads the interface rather than the world, so the
terminal answers it in place — no running yog required, and it exits 0. That
holds for every command, not just the gestures: `yog env --help` prints its
page instead of the export lines, `yog tool-control --help` prints instead of
booting, and `yog bl --help` / `yog bz --help` are balls' and brazen's own
pages, reached without founding a world or needing a workspace.

Actions run the same §8 executors the GUI's buttons do and log the same
`ops.jsonl` rows; queries return the same typed data the GUI renders. The roster
is deliberately not restated here — `yog gesture --help` prints every command,
one line each, read off the interface itself. Exit: `0` ok, `1` refused/failed,
`2` never deposited, `124` no consumer answered (an unclaimed deposit remains
and converges later; a gesture whose engine died mid-run is answered *in
doubt* by the next engine boot — a refusal telling you to read the world
rather than re-send, because an action is not idempotent and a re-send is a
second act).

**Each agent tracks on a balls space of its own** (DESIGN §16.3). A workspace's space is its own clone bundle and its own balls
config home, under its wall, so two agents' task churn never collides; the
project's shared `balls/tasks` board is a destination an agent is *pointed at*,
not the water every agent swims in. An agent raised on a ball is launched onto
that project's board from birth, subagents inherit their parent's space, and
`/marks [<branch>]` reads or amends the branch a space rides. There is no yog
config file — the value written is balls' own `tasks_branch` key in balls' own
config, so removing the space deletes config, not code.

## Building and contributing

| Target | What it does |
| --- | --- |
| `make build` / `make release` | Debug / release build |
| `make test` | Run the test suite (parallel; see the Makefile note on spawn discipline) |
| `make coverage` | Enforce the 100% coverage floor via pinned tarpaulin (see `tarpaulin.toml`) |
| `make lint` / `make fmt` | Clippy `-D warnings` + ast-grep rules audit + `cargo deny check` / rustfmt |
| `make rules-audit` | Run the pinned ast-grep rules (`rules/`) over `src` and verify the `rules/fixtures` negatives still fire |
| `make check` | fmt-check + lint + coverage — the complete gate CI and the pre-commit hook mirror |
| `make drive` [`DRIVE_RUNS="…"`] | Drive the real-substrate ladder (`docs/STORIES.md`): release build, host preflight, one **scratch** world per run verb, a PASS/FAIL line and a `verdicts.jsonl` row per beat, and a pre-filled drive-log skeleton. Never the live world — an overlap with `$XDG_DATA_HOME` is refused |
| `make drive-cleanroom` [`DRIVE_VERB=<verb>`] | The same ladder with only `yog` and `git` on `PATH` — the standing batteries-included done-bar |
| `make drive-preflight` | Name every missing host prerequisite at once (python3, git, the `yog` under drive, the world seed, and the workspace **wall** — whether a scratch world can birth a workspace at all, plus the brazen fixtures seeded into it) |
| `make drive-seed` | Lay a scratch world and print its path — the starting point for a hand-steered capture pass (`docs/QUALITY.md` §3) |
| `make fixture` [`STATE=<name>`] | Lay a named, deterministic **fixture world**, boot an engine on it and print the address a client harness dials; Ctrl-C stops the engine and removes the root. Bare, it lists the states. See "Fixture worlds" below |
| `make drive-log` [`DRIVE_LOG_DIR=<d>`] | Re-emit a run's drive-log skeleton (sha, host tuple, load, beat table) from its verdict rows |
| `make image` [`ENGINE=docker`] | Build the OCI image from `Containerfile`, tagged `yog:<Cargo.toml version>` and `yog:latest`, then run `image-scan` on it. Pushes nothing (see "The image") |
| `make image-scan` | Re-judge an already-built image on its own, both directions — the planted-secret self-test, then the real image |
| `make install` [`INSTALL_PREFIX=<p>`] | Release-build and drop `yog` into `$INSTALL_PREFIX/bin` (default `~/.local/bin`) |
| `make install-hooks` | Seat every `.githooks/` hook as a symlink in the repo's own hooks directory — do this once per clone, in the main checkout |

### Fixture worlds

A client harness — the seat's snapshot pass, an emulator screencap loop —
needs a yog serving a **known** world at an address it can dial, laid the same
way every run. `yog fixture` is that, and it is a verb rather than a script for
the reason `wire-certs` is: an installed binary has no repository to find a
script in, and every consumer of this one lives in another repository.

```
yog fixture                  # the roster, one state per line
yog fixture busy             # lay it; one JSON object on stdout
```

The object is the whole contract — `state`, `root`, `address`, `anchors`,
`chain`, `key`, `origin`, `hold` — so a harness needs no second document to
look a path up in:

```
root=$(yog fixture busy | jq -r .root)
XDG_DATA_HOME="$root" yog &          # boot an engine on it
# …dial the address with ca.pem + client.pem + client.key, render, compare…
kill %1 && rm -rf "$root"            # tear it down
```

**It lays and prints; it does not boot.** The consumer owns the engine process
because the consumer is the one that has to kill it. `make fixture STATE=busy`
is the one-command door over the pair for a hand-run.

| State | What it lays |
| --- | --- |
| `empty` | a seeded world with no workspaces — the first-run state |
| `busy` | one workspace, six conversations across every resting state and every `refs/litany/*` mark |
| `wound` | two wounded conversations: one whose `stderr.log` speaks, one mute |
| `orphan` | an orphaned delivered message and an orphaned tool window |
| `transcript` | a compacted transcript with an entry past every preview cap |
| `settings` | a tuned `cadence.yaml` and a workspace wall carrying provider rows |

**What makes it deterministic.** Every byte a state contains is compiled in;
every commit, message and step is dated from the recipe's own offsets rather
than the laying machine's clock; the `config/default` root commit is pinned at
a fixed instant, so its oid is byte-identical across runs; and the address is
**stated** before the engine binds, because self-provisioning writes
`127.0.0.1:0` and only the listener ever learns what that became.

The residual is named rather than hidden: yog serves derived ages, and the
engine's clock is the system's — there is no environment seam that fakes it and
this deliberately does not add one, because a product that can be told to lie
about the time is worse than a harness that normalises. `origin` reports the
second every offset was measured back from, so an exact age is computable.

**`hold` is the one thing a harness must act on.** A *speaking* conversation is
not a file: liveness is derived from an open `response.json` write fd and a held
executor lock, so no static tree can be one. `hold` names those two paths, and
the harness opens them for the run — one line of shell (`exec 9<dir`), in the
process that already owns the engine. `make fixture` does exactly that.

**It never touches your own world.** The root defaults under your cache root
(`FIXTURE_ROOT` names another) and is a scratch tree the lay wipes before it
writes, so a root overlapping this box's yog data root — in either
direction — is refused before anything is removed.

Code style is governed by the Rust Bootstrap v3 standard — the flat-numbered,
yog-adapted rules in [`AGENTS.md`](AGENTS.md), machine-enforced by `rules/*.yml`
(pinned ast-grep), the clippy manifest, and `cargo-deny`; read it before writing
code.

Task tracking uses [balls](https://github.com/mudbungie/balls) (`bl`);
never commit directly on `main` — all changes land via a claimed worktree
and are delivered by `bl close`. The pre-commit hook enforces this, a
300-line ceiling on source files, and the 100% coverage floor.

## Delivery

Delivery to the upstream ([mudbungie/yog](https://github.com/mudbungie/yog))
is automatic. `bl close` squash-merges the worktree onto `main`, but it seals
that commit itself — outside `git commit` — so git's `post-commit` hook never
fires on a close. Delivery instead rides balls' own extension seam: the
`scripts/bl-push-main` plugin, wired into `close.post`, pushes `main` to
`origin` right after the close lands it. A push failure only warns — it never
fails the close, so a landed change is never lost to a transient network error;
re-push manually with `git push origin main`. Balls task state travels its own
path, pushed by the `bl-tracker` plugin; the two are independent.

Wiring the plugin into a checkout is checkout-local, not a repo edit: symlink
`scripts/bl-push-main` into the landing's `config/plugins/bin/`, then
`bl conf append close.post bl-push-main`. The `.githooks/post-commit` hook is
retained for the rare legitimate manual commit made directly on `main`; run
`make install-hooks` once per clone to arm the git hooks.

### Local install (CICD)

The *local* half of delivery is `scripts/install-main`: it recompiles `main`'s
tip and `make install`s `yog` into `$PATH`, so a landed merge is on this box's
PATH without a manual build. It restarts nothing: a running engine is a
service, and whether to restart it is its unit's question. It builds
from an **ephemeral `git worktree --detach main`**, never the root checkout —
a plumbing delivery leaves the root working tree stale and possibly holding
uncommitted work, so the `main` ref is the only trustworthy source of "what
landed". The build shares the repo's `target/` (`CARGO_TARGET_DIR`) to stay
incremental, runs **detached** (`setsid`) so whatever moved `main` returns at
once, and logs to `target/cicd-install.log` (the relaunched yog's own stderr
goes to `target/yog.log`, kept separate so it doesn't grow the CICD log
forever). Like `bl-push-main` it **always exits 0**.

**Its trigger is the fact, not a verb.** The job is "`main`'s tip is not what
is installed", so the trigger is `refs/heads/main` *moving*, however it moved:
`.githooks/reference-transaction` fires on the transaction's `committed` state,
and dispatches when a line names `refs/heads/main` with a changed oid. That
covers a `bl close`'s plumbing update, a `git pull`/`git merge` on `main`, a
push received into this repo, and a hand repair (`git reset --hard`,
`git revert`) with one mechanism. It was previously a `close.post` balls plugin
named `bl-install-main`, which saw only the close — every other route left
`~/.local/bin/yog` on an older tip. That registration is retired (`bl conf
remove close.post bl-install-main`) and the script is no longer a plugin; two
paths to one outcome is a double build.

Two properties make a hook on so hot a ref safe. It is **idempotent**:
`make install` stamps the commit it built beside the binary
(`$(INSTALL_BIN)/.yog.commit`, named by `make print-install-stamp`), and
install-main compares `main`'s tip to it before building, so a ref write that
installs nothing new costs one `rev-parse` and a string compare. And it
**cannot recurse**: the ephemeral worktree's own ref writes re-enter the hook
as `HEAD`/`ORIG_HEAD`, never `refs/heads/main` (`worktree remove` and
`worktree prune` write no refs at all), so the ref-name test is the loop guard
and the stamp compare is the second one behind it.

Arming it is `make install-hooks`, once per clone, in the main checkout.

On every push to `main`, GitHub Actions runs `make ci` on Linux
(`.github/workflows/ci.yml` — fmt-check, clippy `-D warnings`, tarpaulin 100%
floor). **Linux is the gate**: a release publishes only on a green run of that
workflow.

macOS (Apple silicon) builds and runs `make test` in a workflow of its own
(`.github/workflows/macos.yml`), on the same triggers, and is **reported but
not gating**: it passes, and a release still waits only on Linux. The
difference between the two is coverage — tarpaulin's 100% floor runs on Linux
alone, which is where every line is compiled (nothing but the `lsof` spawn shim
is `cfg`'d out).

### Server install

The section above builds `main`'s tip out of a checkout. A server wants none of
that — a checkout on it would be a second source of truth about what is
running. **A server runs the image** (see "The image" below): it is the unit of
install, it carries its own toolchain pin and its own disclosure gate, and it
lands on the box as one immutable tag.

    make deploy HOST=<ssh-host>          # build here, carry it over, seat it
    make deploy-status HOST=<ssh-host>   # the unit, and the tag it is running

`HOST` is an ssh destination and the only parameter. No machine, address or
account name is committed anywhere in this tree — that is the leak gate's rule
and the severability one at once: pointing this at another box is a different
argument, never an edit.

**What `make deploy` does, in order.** `make image` builds under the pinned
toolchain and runs `image-scan`; the result is retagged
`yog:<version>-<short-commit>`; that tag travels by `save | ssh … load` and the
loaded name is reconciled back to it on the box; the unit and one generated
environment file are seated; the unit is restarted onto the new tag; and the
engine is **verified answering**, or the deploy fails. It refuses a dirty
worktree, because the tag names a commit and the whole point of an immutable
tag is that the box can say what it is running.

**The carrier renames the image, so the box renames it back** (bl-0719).
podman's `save` archive spells a locally built image `localhost/yog:<tag>` —
the registry podman invents for it — and `docker load` faithfully restores
*that* name, while the unit is pointed at the bare `yog:<tag>` this checkout
built. `docker run` then cannot resolve it and the unit crash-loops. The deploy
reads the name `docker load` reports and retags it to the tag the unit knows,
rather than writing `localhost/` into `deploy.env`: the tag is a fact about the
crate — a version and a commit — while the prefix is a fact about which engine
carried it, and the unit's name must not encode the carrier's quirk.

**The last act is a verification, and its exit code is the deploy's**
(`scripts/deploy/verify.sh`). A deploy that prints success over a crash loop is
the defect it exists to prevent: `systemctl --user is-active` says `active`
whenever the `docker run` client process exists, which is not the same fact as
an engine serving. One ssh and one bounded wait past the unit's `RestartSec`,
then five beats — the unit is active; the running container's image is
*exactly* the tag just deployed; `yog --version` inside it answers the version
just built; and an anonymous `openssl s_client` gets a certificate chain out of
the §9.5 listener at the address the engine itself names in `wire/address`.
That last beat needs no seat and no credential: a TLS server sends its
certificate before it ever asks the client for one, so the dial is refused for
want of a client certificate — which is mutual auth working — while the chain
it arrives with proves the wire is bound and serving. On a failure the unit's
last 20 journal lines print with it. It is a separate script so that re-asking
"is it answering?" never means re-seating the box:

    scripts/deploy/verify.sh <ssh-host> <image-tag> <version>

(`make deploy-status` prints the tag the box was pointed at.)

**`make deploy` pushes nothing to any registry.** The ghcr package publishes
only from this repo's release workflow at tag time (DESIGN §10.1); a deploy is
a stream between two boxes and writes to no third place.

### Unattended upgrades

**A seated box upgrades itself to released versions, and this reverses what
`seat.sh` used to say** (bl-4e3c, operator instruction 2026-09-02). That file
carried *"an upgrade is this script, run by a human"*; it now seats a timer
that reconciles the box against `ghcr.io/mudbungie/yog` every fifteen minutes.
The reversal is deliberate and DESIGN §10.1 records why both original
objections stopped holding — in short, releases now have exactly one immutable
registry to poll, and idleness is asked of the engine rather than of a cgroup
that was answering about the wrong process.

**The two paths are split by what they carry, not by who is watching.**
`make deploy` stays the bootstrap, the first seat, the dev-build path and the
emergency path: it carries an unreleased `yog:<version>-<commit>` by
`save | load` and restarts unconditionally, because a human is there.
`reconcile.sh` handles released versions only — a strict `<major>.<minor>.<patch>`
tag, never `latest`, never a dev spelling — and defers instead.

**A package with nothing in it is a clean no-op**: the pass says so and exits 0,
deliberately not a failure — a timer going red every fifteen minutes on a
condition nobody has promised to fix is a timer that gets switched off. A
package that *refuses* an anonymous read is a different answer and does fail,
with the remedy named. Measured (DESIGN §10.1): anonymously those two are the
same `403`, because ghcr will not even issue a pull token for a package that is
absent or private — so **until the first release publishes and its package is
flipped public, a seated box's pass refuses with the visibility remedy**, which
is the right act stated for the benign reason. The box keeps serving the tag
`make deploy` put on it; nothing is downgraded.

What one pass does: read `YOG_IMAGE` out of `deploy.env`; ask the registry
anonymously for its released tags (the package is public, so the box holds no
credential — if a pull ever needs one, the pass fails naming the remedy);
stop if nothing is newer, if the box is *ahead* of the registry, or if this
tag already failed here; ask the engine over the §8.5 control boundary whether
a turn is in flight; and only then pull, repoint, `reset-failed`, restart and
prove.

**The idle question is `{"op":"workspaces"}` and it added no gesture.** A
workspace row already carries `running` — "whether anything in it is
Live/InFlight right now" — so the union over the rows is the whole question,
asked in one deposit. A reply that is `stale` defers too, since that is the
engine saying its own answer may be out of date. A turn is never killed to
make room for an upgrade; the timer simply asks again.

**A release that does not serve is rolled back and never retried.**
`verify.sh --local` runs the same five beats on the box; on a failure the unit
goes back to the tag it was serving, is re-proved on it, and the failed tag is
recorded as `YOG_REFUSED` in `deploy.env` so no later pass re-attempts it. The
bound is an invariant, not a counter: without it a bad release would restart
the engine every fifteen minutes forever. Re-running `make deploy` clears the
refusal, which is the human review it was waiting for. Fleet-wide, the lever is
the registry: a version publishes once, so a bad release is superseded by the
next one and every box picks it up on its own next pass.

Where a refusal shows up: `systemctl --user list-units --failed`, and
`journalctl --user -u yog-reconcile`.

What gets seated is two units, two scripts and one file (`scripts/deploy/`):

| | |
|---|---|
| `yog.service` | `docker run` of the immutable tag, `--network host`, one state mount, `ExecStop=docker stop -t 30`, `Restart=always` — under the user manager, so the world stays the operator's |
| `yog-reconcile.timer` | the retry cadence and the whole of it: every 15 min, 10 min after boot, randomized so a fleet does not ask in unison |
| `yog-reconcile.service` | one `oneshot` pass, left in `failed` when it refuses so an operator can see it |
| `~/.local/bin/yog-reconcile` | `reconcile.sh`, beside the `verify.sh` it calls |
| `~/.config/yog/deploy.env` | generated on the box: `YOG_IMAGE=<the tag>`, the git identity the container commits under, and `YOG_REFUSED` after a rollback |

The tag lives in that file rather than in the unit because the crate version
has one home and a version typed into a unit is that fact stored twice. The
git identity lives there because yog's substrate commits and git refuses
against an identity-less container — `seat.sh` takes it from the deploying
checkout's own `git config`, so no name is committed to this tree.

Lingering (`loginctl enable-linger`) is enabled by the deploy, and it is
load-bearing: without it the user manager — and so the engine — stops at
logout, which presents as "it just stopped overnight".

**The mount is one directory**, `~/.local/share/yog` — the box's real XDG data
root, exactly where a binary install's world would be — bound to `/state/yog`
inside the container, plus `~/work` for §8.1's bare-rung cwd. The nesting is
why it is one and not three; "What mounts where" below is the whole contract.

**The stop is `docker stop`, not a signal to the unit.** DESIGN §8.5's 30 s
grace is only spent if SIGTERM reaches PID 1 *inside* the container; killing
the `docker run` client detaches from the container rather than stopping it.
`TimeoutStopSec` is longer than that grace, or systemd's own kill lands
mid-window and the grace is decorative.

**The hourly reconciler is retired** (bl-c6e2), and re-running the deploy
disables it on a box that still has it. It reconciled a *cargo-installed
binary* against the crates.io index and read quiescence off the unit's own
cgroup. Against a container unit both facts are wrong in the direction that
**acts**: it would see the installed binary differ from what is running and
restart the unit under it. The reconcile question for an image is "is a newer
image loaded", and nothing on the box can answer that without a registry to
poll — which is deliberately not there.

So an upgrade is `make deploy`, run by a human, and the restart it performs is
unconditional. That is a real loss and worth naming: the retired reconciler
**deferred** a restart while a turn was in flight, because an unattended timer
cannot know whether it is interrupting anything. A deploy can — there is a
person at the keyboard who chose the moment. What a killed turn costs is
unchanged: the tools in flight die, their side effects stand, and the next
model call is paid again to re-derive from a window of error results. The
conversation itself survives, because the pinned litany settles the unpaired
tool-use tail at the next drive boundary (ARCH §6 crash settlement, upstream
bl-4187, consumed here in bl-4c1f), so an ordinary deposit revives the branch.

**A release that cannot start is visible rather than hidden.** A user unit
cannot order against the container daemon, which is a system unit, so the unit
expresses that ordering as patience — twenty starts five seconds apart, a
~100 s window in which the daemon can finish coming up after a reboot — and
past it the unit enters `failed`, where `make deploy-status` shows it. That
window is wider than a crash-loop detector wants; the trade is deliberate,
since the boot race happens on every reboot while a bad build is caught by
`make image` before a byte reaches the box — and, since bl-0719, a release that
cannot start is caught by the deploy's own verification within it, because the
beats above wait past one `RestartSec` and then read the container rather than
the unit.

### The image

`make image` builds an OCI image from `Containerfile` — a third route, for a
box that takes images rather than binaries. **The image is the unit of install
and nothing more.** No part of yog uses the container filesystem as a feature,
and no state lives in a layer: the XDG root is the runtime contract and it is
mounted in.

```
make image                   # podman or docker, whichever is on PATH
make image ENGINE=docker
```

It builds under the pinned toolchain (`rust:1.95.0-alpine`, checked against
`rust-toolchain.toml` during the build so the two pins cannot drift) and copies
one static musl binary into an `alpine` runtime layer.

**The runtime layer is what the engine execs**, which is why `FROM scratch` is
wrong here whatever the linking story says. Four things: `git` (every workspace
read and every act is git — `src/git_env.rs` is the crate's one fork),
`openssl` (the wire mint the boot performs when a box has none; without it a
fresh mount comes up with no listener), `sh` (the `$EDITOR` re-entry a §9.3
lineage write performs), and **yog itself** — the world's `PATH` head is
re-exec shims of this binary, so `bl`, `litany` and `bz` need no host binary at
all. System CA roots ride along for the adapter's HTTPS and for an HTTPS git
remote. `lsof` is absent: DESIGN §10's macOS liveness shim is not compiled into
a Linux binary.

#### What mounts where

`XDG_DATA_HOME` is set to `/state`, which puts yog's data root at `/state/yog`
— the extra level is XDG's, not the image's. **That one directory is the whole
mount**, and that is the point: yog composes a nested world under it and
derives `LITANY_HOME` and `XDG_STATE_HOME` onto `world/litany` and
`world/state` for itself and every child, so mounting those separately is
fighting the nesting rather than configuring it.

```
podman run --rm \
  -v ~/yog-state:/state/yog:Z \
  -v ~/work:/work:Z \
  yog:0.0.5 gesture '/attention'
```

Inside that one root: `world/litany` (the nested harness home), `world/state`
(balls' clones and worktrees, and yog's own `ui.json` / `ops.jsonl`),
`world/tools` (the re-exec shims, seeded by the engine), `workspaces/` (the
named workspaces) and `wire/` — the wire material, beside the world rather than
inside it, because the world is a generated artifact yog reseeds and a reseed
must not be a revocation.

Nothing in the image runs a seed. The engine founds what it finds missing on
its own boot, into the mount; writing any of it into a **layer** would put the
one state yog owns where a mount cannot replace it and an upgrade cannot see
it.

There is no `VOLUME` instruction on purpose — and for yog that matters more
than for the other components. A `VOLUME` would let an unmounted run succeed
against an empty anonymous volume, and yog's boot would found a whole world
there and answer as if it were real. Without one, an unmounted run founds its
world inside the container and takes it down with the container, which is at
least visible.

#### What the image deliberately does not contain

- **No wire material.** The CA, its leaves and the `address` file are the
  operator's, minted on the operator's box (`yog wire-certs`, or the engine's
  own boot into the mounted root). REMOTE §1.4 is that certificates arrive out
  of channel by hand, forever; an image that arrived able to present an
  identity would be the in-channel bootstrap that must never exist.
- **No provider credentials, and no world at all.** Both are mounts. A
  credential baked into a layer is a credential published to everyone who can
  pull it.
- **Nothing an agent runs.** A tool call routes to an enrolled thrall over the
  real wire or refuses in band (REMOTE §12's ship-inert posture); the engine
  runs no tool itself. A toolchain in this layer for agents to use would be
  quietly undoing that.
- **No git identity.** yog's substrate commits into the workspaces it drives,
  and git refuses with `Please tell me who you are` against an
  identity-less container. Supply one — `GIT_AUTHOR_NAME` /
  `GIT_AUTHOR_EMAIL` / `GIT_COMMITTER_NAME` / `GIT_COMMITTER_EMAIL`, or a
  mounted `.gitconfig`.
- **No `cargo`, no compiler, no source, no `target/`.** The build stage is
  discarded whole; only the binary crosses.

#### The image-side disclosure gate

`make image` ends in `make image-scan`, and that is a condition of the registry
ruling rather than a convenience (DESIGN §10.1). `make leak-scan` reads the git
**index**; an image is built from inputs no commit has — the build context as
the engine actually receives it, the base layers, the package index, and the
image **config**. The scan reads all three surfaces through the **same rule
table** (`scripts/leak-rules.sh`, sourced and never copied) and runs both
directions: a scratch image with a planted secret in a layer, another in an
`ENV`, and an undeclared binary, all of which must be caught, before the real
image is scanned clean.

What it cannot promise, stated rather than implied: it scans one image, on the
box that built it, before the push. It does not read what is already in the
registry, it cannot un-publish a digest, and whoever runs the build can bypass
it exactly as `--no-verify` bypasses the commit hook.

**Run it under podman today.** `make image ENGINE=docker` refuses at the
self-test: `docker image inspect` exposes no top-level `History`, so the one
`--format` template the scan reads the image **config** with fails execution
and yields a single newline — the `Env` surface goes unscanned, the planted
ENV secret is missed, and the build is refused rather than published. The gate
fails closed, which is the right direction, but it means the docker path builds
nothing until that is fixed (bl-09d4). Podman is the Makefile's default anyway, and it is
what the release workflow names.

**`make image` pushes nothing, and there is no `push` target.** The registry is
`ghcr.io/mudbungie/yog`, pushed only from this repo's release workflow at tag
time; a push is not undoable, and a convenience target for an irreversible act
is how the act happens by accident.

**What does push is the `release-image` job** in
`.github/workflows/release-plz.yml` (bl-6b96). It hangs off the release job's
own output, so an image publishes exactly when a crate version does and behind
the same green-CI gate; it builds with `make image` on the runner, so the
disclosure gate above runs there over the exact bytes it is about to push; and
it pushes ONE name, `ghcr.io/mudbungie/yog:<version>`, recording the manifest
digest in the run summary. The local `:latest` dies with the runner. Its
authority is the workflow's own `GITHUB_TOKEN` with `packages: write` — no PAT,
no registry secret, nothing to rotate.

Three things it deliberately does not do: there is **no manual dispatch door**
(a Release asset can be replaced, a published tag cannot), it **reads the
registry first and refuses to re-push a version that is already there** (a
rebuild is not byte-identical, so a second push would move the tag onto new
bytes), and it publishes **single-arch x86_64** — the runner is amd64 and the
image carries one static musl binary, so an arm64 box gets nothing it can run.
A multi-arch manifest would be its own decision.

**The package needs one operator act, once.** GitHub creates a new package
private whatever the repository's visibility, and every box reads the registry
anonymously by design — so after the first release publishes, flip the package
to public: the repository's **Packages** → `yog` → **Package settings** →
**Danger Zone** → **Change visibility** → **Public**. There is no REST route
for it (DESIGN §10.1 records the probe). Until that flip, a reconcile pass
refuses loudly and names it.

## Publishing

The crate reaches [crates.io](https://crates.io/crates/yog) two ways.

By hand: `make publish` runs `cargo publish --dry-run` unconditionally, and the
real upload only when you opt in with `make publish CONFIRM=yes`.

In CI, hands-off end to end: `.github/workflows/release-plz.yml` keeps ONE open
"release PR" that bumps the version and stages the changelog entry, and
`.github/workflows/release-automerge.yml` merges that PR once its `CI` run is
green. The merge lands on `main`, CI runs there, and only a CI run that
concludes `success` releases — tag, GitHub Release, crates.io upload, then the
linux-gnu binary archive and the ghcr image (see "The image"). One version,
three artifacts, one trigger.

**The gate is the build, not the merge** (bl-1c05). Holding the release PR open
for a hand looked like a control point and was not one: the decision it asked
for had already been made by the work that landed on `main`, and the publish is
gated *after* the merge anyway, so a red build skips the release and leaves the
bump sitting harmlessly on `main`. What auto-merge removes is a hand, not a
check. The merge guards, why the token doing the merging is
`RELEASE_PLZ_TOKEN` and never `GITHUB_TOKEN`, and why this needs no post-merge
dispatch, are written out in that workflow's header.

That release job is the only thing in the repo holding publish authority, so
the boundary around it is written out rule by rule in the header of
`release-plz.yml` (bl-5ae6). In short: no other job is handed the registry
token, the job runs in the `publish` environment so the token can sit behind
branch and reviewer rules, every action is pinned to a full commit SHA, each
job declares its own token scopes under a repo-wide `permissions: {}`, the
release is cut from the exact commit CI proved green
(`workflow_run.head_sha`, not the branch tip that may have moved), a manual
dispatch can only backfill binaries and can never publish, and the one operator
input crosses into shell through an environment variable rather than `${{ }}`
interpolation.
