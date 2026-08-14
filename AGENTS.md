# yog — Agent Operating Guide

You are working in **yog**, a single published binary crate: an egui desktop
window that drives `lernie` loops over `balls` tasks. Two authorities govern
your work and they do not overlap:

- **`docs/DESIGN.md` is the architecture authority** — what yog *is*, its
  invariants, module map (§12), and the world substrate it composes (§16). When
  code and DESIGN disagree, one of them is a bug; never invent a third answer.
  Do not implement a deviation silently — fix the doc (see the global AGENTS.md
  guidance: "Nothing is set in stone").
- **This file is the code-style authority** — the machine-enforced rules below
  and the repo discipline that surrounds them.

yog composes a **nested world** (DESIGN §16): it overrides `LERNIE_HOME` and
`XDG_STATE_HOME` under `$XDG_DATA_HOME/yog/world` and hands that env to every
child it spawns, so yog's `bl`/`lernie` substrate never collides with the
user's ambient tools (brazen's config/credentials/cache resolve **per
workspace** since the blast-radius ruling — nothing brazen-shaped
stays ambient, §16.2). If you touch spawn paths, env folds (`src/xdg`), or
`src/world/*`, read §16 first — the nesting is the point.

---

## Code-style rules (Rust Bootstrap v3, adapted to yog)

The house standard is **contained Rust**: complexity lives in function bodies
(local, compiler-caught), not in type signatures (viral). Prefer
clones / `Arc` / `Box<dyn Trait>` / enums over borrow-based APIs — the perf given
up was never why we chose Rust. The rules are flat-numbered so they are
mechanical to follow; most are machine-enforced by a pinned ast-grep rule
(`rules/*.yml`), the clippy manifest (`Cargo.toml [lints]`), or `cargo-deny`.
**Where yog deviates from the standard, the deviation and its reason are
recorded verbatim in the rule — read it before assuming a rule is absolute.**

1. **No named lifetimes.** `'a`/`'ctx`/… on a signature, struct, or impl leaks
   internal storage into the interface. Borrow on the way IN (elided), hand back
   OWNED on the way OUT. `'static` and `'_` are fine (they name nothing).
   Enforced: `rules/no-named-lifetimes.yml`.

2. **A `pub fn` returns an owned, concrete type** — never `&T`/`&mut T`/`&[T]`
   nor an opaque `impl Trait` (edition-2024 implicit capture makes `impl Trait`
   smuggle borrows invisibly). Return `String`/`Vec<T>`/a named struct, or an
   index the caller resolves. If the accessor is internal, demote to
   `pub(crate)` rather than clone-to-own. Enforced:
   `rules/no-pub-borrow-return.yml`.

3. **`unsafe` is confined, not forbidden. yog ADAPTATION:** the standard's
   `unsafe_code = "forbid"` is replaced by an ast-grep *location* rule pinning
   every `unsafe` to `src/cli_outbound/sys.rs` — one irreducible SIGTERM syscall
   in `Stream`'s drop. `forbid` is unoverridable and reaches test code, and a
   `nix`/`rustix` dependency or a crate split for ~10 lines of FFI is worse than
   a confinement rule. Enforced: `rules/unsafe-outside-sys.yml`.

4. **No panic paths outside tests.** `unwrap`/`expect`/`panic!`/`todo!`/
   `unimplemented!`/`dbg!` and unchecked `indexing_slicing`/`string_slice` are
   `deny` in the manifest; `assert!`/`assert_eq!`/`assert_ne!` are banned in
   prod (a `panic!` in disguise); `debug_assert!` is fine. Locks use
   `unwrap_or_else(PoisonError::into_inner)`; fallible reads use `.get()`/`?`.
   Tests get carve-outs via `clippy.toml` (`allow-*-in-tests`) and
   `rules/no-assert-outside-tests.yml`.

5. **No `#[allow]` in prod.** An inline `#[allow]`, a `#![allow]`, or a
   `#[cfg_attr(…, allow(…))]` hides a warning where it fires. Policy lives in
   the manifest (`Cargo.toml [lints]`) where it is reviewable and justified in
   one place — **that manifest is the only home for a suppression.** Test code
   may relax a lint. Enforced: `rules/no-lint-suppression.yml`.

6. **Dependencies are pre-approved + `cargo-deny`. yog ADAPTATION (amended
   §16.7 W10):** the standard's "rustls-only, no openssl" is now yog's rule
   **verbatim**. The former adaptation — "no TLS surface at all", justified by
   yog being a native GL desktop app with no network — died when the
   batteries-included wave embedded `brazen`, the LLM network adapter: yog's
   own process now makes the HTTPS calls, so `ureq`/`rustls`/`ring`/
   `webpki-roots` are load-bearing, not incidental. `deny.toml` still bans
   `openssl-sys` AND `native-tls`, which was always the standard's point — a C
   toolchain dep and a non-portable system bridge, either of which breaks the
   single-binary musl/macOS/Windows story rustls keeps. The license allow-list
   is exhaustive over the committed `Cargo.lock`; transitive advisories may be
   ignored only with a reason recorded beside the entry, and **`deny.toml` is
   the authority for which ones and why — no advisory, count or rationale is
   restated here**, an entry that stops matching being a warning telling you to
   drop it. **Zero new dependencies without explicit user approval.**
   Sources are **registry-only, with no exception in force**: since
   bl-89a4 all three embedded substrate crates (`balls`, `brazen`, `lernie`)
   are plain crates.io pins — **`Cargo.toml` is the pin authority and no
   version is restated here**, the restatement having gone stale once already —
   `deny.toml` has no `allow-git` list, and `make publish` works. The phase-2
   ruling (DESIGN §16.7) still permits ONE interim exception — an embedded substrate crate
   pinned `version = "=x.y.z"` **plus** an exact `git`/`rev` while an upstream
   publish is in flight — but taking it re-blocks `make publish` (crates.io
   refuses git deps) and re-exposes yog to a rewritten upstream history
   orphaning the rev, so it is a last resort with a named exit. The pin must
   always be exact and lockfile-fixed; a `path` dependency is never lawful.

7. **`Mutex`/`RwLock` only in `src/state.rs`; no `Rc`/`RefCell` anywhere. yog
   ADAPTATION:** the lock chokepoint (`state.rs`) has three sanctioned
   carve-outs — the test scaffolding locks (`SPAWN_LOCK`/`ENV_LOCK` in
   `test_support`), `src/git_tree/probe_cache.rs` (a macOS 2 s TTL cache whose
   `Mutex` is uncontended single-thread interior mutability), and
   `src/fs_watcher/hub.rs` (the process's one `notify` instance and its fan-out
   registry, `OnceLock` singletons that are never dropped or handed out —
   bl-908c). Both code carve-outs share one reason: folding them into
   `state.rs` breaks llvm-cov's per-line coverage there, shifting the file's
   byte offsets and mis-attributing phantom uncovered regions onto its `impl`
   headers and type aliases. `Rc`/`RefCell` are banned everywhere, tests
   included (bare `Cell` counters are fine). Enforced:
   `rules/locks-outside-state.yml`, `rules/no-rc-refcell.yml`.

8. **Async is tokio-only, `#[async_trait]` mandatory. yog ADAPTATION:** yog runs
   **no async and no tokio today** — it is a synchronous egui frame loop over
   subprocess spawns. This rule is installed but **vacuous**; honor it if async
   is ever introduced, do not add tokio to satisfy a rule that currently matches
   nothing.

9. **No trait bounds on a `pub` item** — no `pub struct S<C: Clock>`,
   `pub fn f<T: Into<String>>`, nor a bounded `where`. A bound on the public
   surface forces monomorphization onto every consumer. Dissolve with a trait
   object or a concrete param; demote an internal bounded helper to
   `pub(crate)`. An *unbounded* `pub fn f<T>(x: T)` is fine. **yog ADAPTATION:**
   the shared time source is `Arc<dyn Clock>`, not `Box<dyn Clock>` — the caller
   that injects it (a test advancing a `FakeClock`, `main.rs` handing over a
   `SystemClock`) keeps a handle while the §7.2 sweep schedule holds its own, so
   it must be shared, not owned. Enforced: `rules/no-pub-generic-bounds.yml`.

10. **`thiserror` in libs, `anyhow` in the app. yog ADAPTATION:** `thiserror` is
    in place for the error enums; **`anyhow` is NOT a dependency** — `main.rs` is
    a thin entry with no error-plumbing layer that would justify it. Do not add
    anyhow.

11. **One crate per module boundary. yog ADAPTATION:** yog is a **single
    published binary crate**, not a workspace. The module tree plus the 300-line
    cap (below) already contain complexity; a crates split buys nothing here and
    would fight the 100% coverage floor. No `[workspace]`.

**What "pub" means for rules 2 and 9:** the `pub` surface is the *real* library
surface consumed by the `tests/` integration crate and `src/main.rs`. Anything
internal is `pub(crate)` — the ast-grep rules scan only bare `pub`, so an honest
demotion removes an internal API from the boundary's obligations. Reach for
`pub(crate)` before cloning-to-own or de-generifying a purely internal type.

---

## Repo discipline

- **Task tracking is `bl` (balls).** Run `bl skill` before using it. Session
  start is `bl prime --as YOUR_IDENTITY`, then `bl list`.
- **Claim → work → close, in the worktree.** `bl claim <id> --as ID` prints a
  `work/<id>` worktree; **every edit goes there**, never on `main`. `bl close
  <id> --as ID` folds `main` in, runs the pre-commit gate, squash-delivers, and
  tears the worktree down. Always pass `--as ID` — never let the model invent a
  name. A stray edit on `main` is invisible to the squash and is left behind.
- **`main` is the only durable line, and the remote says so.** Four kinds of ref
  may exist on `origin`, each with an owner; anything else is a defect, and
  which one it is should be answerable from this list alone (bl-7066).
  - `main` — the trunk. **Work lands ONLY by `bl close` squashing a claim
    worktree onto it.** A pushed branch is never how work lands.
  - `balls/tasks` — the task store: structural, owned by `bl`, not a line of
    development. It publishes with the source (see "What may never enter a ball
    body").
  - `release-plz-*` — machine-owned and machine-pruned. release-plz pushes a new
    timestamped branch on every refresh, and `prune-release-branches`
    (release-plz.yml) deletes every one with no open PR right afterwards. Leave
    them alone; do not hand-delete the one with the open release PR.
  - **anything else is a PROBE, and the agent that pushed it deletes it.** The
    only reason to push a branch at all is to buy a **runner verdict** — a
    `workflow_dispatch` needs a ref that is already on the remote, and some
    defects exist only there (bl-e492: 1969/1969 locally, five failures on CI).
    That is legitimate; leaving it is not. Read the verdict, land through `bl
    close`, then delete the branch and close its PR in the same breath.

  **A probe is not free, and deleting it does not make it free.** GitHub keeps
  `refs/pull/<n>/head` forever: the public repo still serves the head refs of
  two branches that no longer exist anywhere. Whatever you push is a permanent
  public artifact of that tree even after the branch is gone — so push a probe
  when you need a verdict, and not to "save work somewhere".
- **Every child process is spawned through `git_env::command`.** `git` exports
  `GIT_DIR`/`GIT_INDEX_FILE` into every process it starts, and those OUTRANK
  `-C <repo>` and `current_dir` — so a child that inherits them forks its *own*
  `git` against the hook's repo. bl-0dff closed that for yog's direct `git`
  forks; bl-916a moved the scrub to the **spawn boundary**, where one
  `env_remove` clears the whole descendant process tree (`bl`, `lernie`, `bz`,
  an `$EDITOR` shim, the suite's fake substrate scripts). A bare `Command::new`
  outside `src/git_env.rs` is an error — `rules/no-bare-command.yml`.
  **`git commit` inside a `work/<id>` worktree is safe again:** the hook runs
  the suite, and the suite no longer writes to the outer repo (regression:
  `tests/git_env_scrub.rs`; verified by running the whole suite with
  `GIT_DIR`/`GIT_INDEX_FILE` pointed at a decoy repo, which stays untouched).
  Two residuals, both narrow: `make rules-audit` scans `src` only, so a bare
  `Command::new` added under `tests/` is on you; and a test binary that drives
  the embedded substrate **in-process** (`multiplex::dispatch`) must scrub its
  own process env from `git_env::INHERITED`, as `tests/multiplex_bl.rs` and
  `tests/multiplex_lernie.rs` do — no spawn boundary exists to do it for them.
- **300-line hard cap on every source file, inline tests included.** Docs and
  config (`.md`/`.toml`/`.yml`/`.json`/lock, `Makefile`, `LICENSE`) are exempt.
  Anything projected ≥200 is pre-split at design time (DESIGN §12), not at the
  cap — 200 is the aspiration the tree was swept to (bl-52f8), 300 the wall. **`make line-cap` is the one definition of the cap and of the exempt
  set**; the pre-commit hook and `make lint` both call it, neither restates it.
  It scans the **whole tree**, not the staged diff — the hook once checked only
  the files you happened to touch, which made the cap a sampling rather than an
  invariant (`src/app/balls.rs` rode at 308 lines undetected until an unrelated
  task edited it, bl-12dc). Over the cap? Split along a real seam and add the
  row to DESIGN §12; never shave lines to duck the limit.
- **100% test coverage, `cargo-tarpaulin` pinned 0.35.2.** `tarpaulin.toml`
  holds the config (excludes `src/main.rs` and `src/shell/*`); the hook and CI
  both run `--fail-under 100`. If it can't be tested, it mustn't be built.
- **The clippy pedantic allow-list lives ONLY in the manifest.** `Cargo.toml
  [lints.clippy]` runs `pedantic = deny` with a justified allow-list (currently
  13 entries in three tiers: the bootstrap five, six empirically-warranted for
  this egui GUI, and two site-specific false positives). Each entry carries a
  one-line justification. Never inline a suppression to dodge it (rule 5) — add
  a justified manifest entry via review instead.
- **`docs/DESIGN.md` is the architecture authority.** Amend the doc when reality
  diverges; do not code around a stale doc.
- **Never credit AI or tooling** in commit messages, code, or docs.

---

## The local gate

`make check` is the complete local gate and mirrors CI exactly:

    fmt-check → lint (line-cap + leak-scan + clippy + ast-grep scan + cargo-deny) → coverage

- `make lint` — `make line-cap` (sub-second, so it fails first), then
  `make leak-scan` (~5s), then `cargo clippy --all-targets -- -D warnings`
  (picks up the manifest `[lints]`), then `make rules-audit`, then
  `cargo deny check`.
- `make line-cap` — the 300-line cap over every tracked non-exempt file. Prints
  every offender at once, and fails if it enumerates *nothing* (a broken
  pattern must not pass silently — the same two-direction discipline as
  `rules-audit`'s fixtures). The cap is a parameter, so **`make line-cap
  LINE_CAP=199` lists the ≥200 pre-split band** — run that before you extend a
  module, not after. It is deliberately *not* a gate: 94 of 395 source files sit
  in that band today. A warning that fires on a fifth of the tree is noise, and a
  gate there is just the cap moved to 200. ≥200 is a **design-time projection
  rule** — it fires on the author about to add to a file, not on the file's
  existing state. The band is the aspiration, not the limit: bl-52f8 swept the
  tree so nothing rides the 300 wall, and nothing has since — **ask `make
  line-cap LINE_CAP=n` for today's census rather than trusting a count written
  here** (this line has been wrong before: it named `transcript/rows.rs` at 266
  as the tree's one ≥250 file long after bl-2335 split that file in two).
- `make leak-scan` — the disclosure gate (bl-fd5a, reworked bl-167d).
  `scripts/leak-rules.sh` is the one definition of what may not be committed:
  private keys, vendor API tokens, credential assignments, routable
  IPv4/IPv6/MAC addresses, **absolute paths under any home root on any
  platform** (`/home/…`, `/Users/…`, `C:\Users\…` — the house synthetic roots
  `/home/u`, `/home/op`, `/home/x` are the only account names that pass),
  email addresses outside the reserved documentation space, dialogue behind a
  speaker label, agent-session artifacts (vendor resource ids, Claude Code
  transcript keys), credential-shaped file paths, and **content no rule can
  read**. `scripts/leak-scan.sh` is the mechanism; findings are truncated to 12
  characters, because a finding must LOCATE a leak, never reprint it into a
  terminal or a CI log.

  **It reads index BLOBS, not the worktree.** `git checkout-index`
  materializes the index into a scratch tree and the scan reads that, so the
  bytes scanned are the bytes committed. The index rather than the diff, for
  the same reason `line-cap` reads it. Until bl-167d the scan enumerated `git
  ls-files` — path NAMES — and grepped the WORKTREE files they pointed at, so a
  leak that was `git add`ed and then overwritten with a clean copy on disk was
  committed unread.

  **Unreadable is rejected, not skipped.** `grep -I` silently passes binary
  files, which is the class most likely to carry a dump (archives, databases,
  PDFs, HAR captures, screenshots, executables). A tracked binary must be a
  regenerable derivation with a byte-for-byte test — `BINARY_ALLOWED` names
  exactly `assets/yog-*.png`, which `make icon` emits and
  `src/theme/icon/tests/artifacts.rs` pins.

  **The scan is never cached.** `scripts/pre-commit` runs it BEFORE consulting
  bl-speculate's verdict cache, so no stored verdict — including one imported
  from the remote builder — can let a leak through unread. (The cache's gate
  fingerprint is a fixed file list compiled into `bl-speculate` that cannot
  name the scanner; not being cacheable dissolves that rather than waiting on
  upstream.)

  Its regression half is `--self-test`, which the target runs first and which
  is stricter than `rules-audit`'s: every rule owns a fixture in
  `scripts/leak-fixtures/` where **every non-comment line** must be flagged
  **by that rule** — so one dead alternative inside a nine-way pattern cannot
  hide behind the eight still working — and must carry `FIXTURE_MARKER`
  (`notreal`), because no regex can tell a real secret from a fabricated one
  and only the value can say so. Plus `clean.txt` / `clean-paths.txt`,
  near-misses that must NOT be flagged. Both directions, because a leak gate
  dies by matching nothing, and a noisy one dies by being bypassed.
  `tests/leak_gate.rs` holds the other half: seven tests that drive the real
  scanner over throwaway repositories.

  **There is no allowlist and no per-rule path exemption, and nothing is
  exempt from the tree scan.** There was one — `docs/drive-logs/` was exempt
  from the home-path rule, on the argument that a drive log is evidence of a
  run on a real box and the path *is* the evidence. bl-244f burned those logs
  instead. bl-167d then removed the last two skips: the scanner and its rule
  table are scanned (they stay clean because **no pattern may match its own
  text** — see `leak-rules.sh` on the one that did), and each fixture is
  scanned by every rule EXCEPT the one it is the fixture of, a structural
  exemption keyed to the file's own name rather than an allowlist. **Fix the
  rule, not the coverage.**

  Writing a drive log is still QUALITY.md §3 step 6, and it stays lawful
  because `scripts/drive/logskel.sh` folds `$HOME` to `~` in every path it
  emits, at the one place the text is written. Hand-finish a log the same way.

  `.githooks/commit-msg` runs the same scanner over the commit MESSAGE, which
  `pre-commit` never sees. Run `make install-hooks` once to seat it.

  **The scanner scans the tree it is RUN IN, which need not be this repo**
  (bl-1043). It resolves its rule table from its own directory and the tree
  from `git rev-parse` in the working directory, so `cd <any git checkout> &&
  <repo>/scripts/leak-scan.sh` judges that checkout's index by this table. That
  is how the task store is gated below without a second copy of the rules.
- `make rules-audit` — `ast-grep scan src` (must be clean) AND a negative check
  that `ast-grep scan rules/fixtures` *fails* (proving every rule still bites).
- `make coverage` — pinned tarpaulin, `--fail-under 100`.

**Tool pins (must match, or the gate/CI is not reproducible):** rustc `1.95.0`
(`rust-toolchain.toml`), ast-grep `0.44.1` (`sgconfig.yml`), cargo-deny `0.20.2`
(`deny.toml`), cargo-tarpaulin `0.35.2` (`tarpaulin.toml`). Bump a pin only as a
deliberate, isolated change.

---

## What may never enter a ball body

`bl` keeps this project's tasks in a **separate git repository** — `tasks/*.md`
on the `balls/tasks` branch, pushed to the *same remote as the source*. A ball
body is therefore published text on a ref that goes public with the crate
(operator ruling 2026-08-13, bl-dd1d: the store is **scrubbed and published**,
not moved to a private remote). Nothing you write in one is private, and the
source gate has never seen a byte of it — `make leak-scan` reads the index of
*this* tree, and the store is not in it.

Write the reasoning; leave out the identity, the chronology and the machine
state — the same editorial rule bl-2368 applied to the source tree. None of the
following may enter a task title, body, comment or `-m` note:

- **Other people's names, handles and addresses.** Third parties, other
  operators, anyone who did not publish themselves. **The maintainer's own
  `mudbungie` identity and `mudbungie@gmail.com` are explicitly permitted** —
  that is the companion ruling on bl-dd1d, not an oversight: the handle is
  already public in `LICENSE`, `Cargo.toml`, the README and the release-plz
  owner guard, and `leak-rules.sh`'s `personal-email` rule excepts that one
  address on purpose. Every *other* address is a leak.
- **Verbatim transcript prose.** Operator dialogue, model output, an agent's
  own reply pasted back in. Cite the conclusion and the ball it came from — a
  conversation is content somebody said, and quoting it publishes them.
- **Live machine state.** Process ids, load figures, absolute paths under a
  real home (`/home/<account>`, `/Users/<account>`, `C:\Users\<account>`),
  workspace and wall names off a live world, host and device names. Cite the
  *shape*, not the instance: `/home/u` is a house synthetic root and passes.
- **Provider auth state.** Who is signed in to what, which credential exists or
  does not, billing and account-status text quoted from a provider. "The
  account cannot run jobs" is the fact; the provider's sentence about it is
  disclosure.
- **Conversation and session ids.** Vendor resource ids, Claude Code transcript
  keys, and the identifiers of a specific run on a specific box.

The gate below enforces the mechanical half of this list. The half no regex can
reach — an unlabelled paragraph of somebody's conversation, a third party named
in ordinary prose — is yours, and it is the half that actually leaked: bl-dd1d's
audit recovered no credential of any kind. What it recovered was private
context.

### The task store gate

`scripts/yog-leak-gate` is a **balls plugin** that runs `scripts/leak-scan.sh`
— the same script and the same `leak-rules.sh` table as `make leak-scan`,
because two copies of the rules drift within a week — over the store checkout,
and exits non-zero. A non-zero exit is the balls protocol's abort: the op is
refused and the plugins that already ran roll back in reverse, so the store
commit is un-sealed before `bl-tracker` can push it.

It hangs at `<op>.post`, not `pre`. A `pre` plugin runs **before** `bl` writes
the task file, so it would scan the previous state and wave through the very
body being added; `post` is the one window in which the ball exists and has not
yet been published.

Wiring is one act per checkout, and this repo cannot perform it — the plugin
schedule lives in the balls landing (`balls/config`), not in yog's tree:

    bl install --bin yog-leak-gate=<repo>/scripts/yog-leak-gate
    for op in create update claim unclaim close drop; do
      bl conf prepend $op.post yog-leak-gate
    done

`prepend`, never `append`: plugins run in list order and only the irreversible
belongs last, so the gate must sit ahead of `bl-tracker` (which pushes) and
`bl-delivery` (which squashes). Those six ops are exactly the ones this
landing runs `bl-tracker` on — *the gate goes immediately before the publisher,
everywhere the publisher runs*. It is severable: `bl conf remove <op>.post
yog-leak-gate` deletes config, not code.

**Wire it only once the store scrub (bl-dd1d) has landed.** When this gate was
written the store tip tripped `home-path` on nine live task files — bl-dd1d's
own body among them, because it cites the paths as evidence — so wiring it
first would refuse every `bl` op on the box, the scrub's own edits included.

**What it cannot do.** It stops the accident, not the author. The same agent can
`bl conf remove` it, or commit and push inside the store clone by hand, exactly
as `git commit --no-verify` defeats the source hook. There is no unbypassable
*preventive* placement to move it to: a git hook inside the store clone is
strictly worse (untracked, per-clone, re-founded by `bl prime`, absent on every
other box and silently so), and GitHub cannot interpose a check on a direct
push to `balls/tasks` — there is no pull request to require a status check on,
and a server-side hook is not a repo artifact, the same boundary bl-167d drew
for protected refs. The check the author *cannot* switch off is
`.github/workflows/store-scan.yml`, which scans the published ref daily, on
dispatch, and whenever the rule table itself changes. It runs after the push,
so it **detects**; the remedy for a hit is a history rewrite. Prevention is
local and bypassable, enforcement is remote and late, and stating that is worth
more than a gate that implies otherwise.

---

## Before making the repo or a crate version public

**A commit hook scans one tree.** `make leak-scan` reads the index it is about
to commit, and that is the whole of what it can promise. Everything below is
outside any hook's reach, cannot be made an invariant by one, and is therefore
a checklist run by a person once per publication — not a gate, and not
something the gate should imply it has covered (bl-167d).

Nothing here is automated on purpose: each item is a one-time judgement whose
remedy is destructive (a history rewrite, a yank, a rotation), so a green
checkbox would be a worse answer than a person looking.

This list was RUN, once, on 2026-08-13 (bl-4f96). What it found and what that
cost is recorded per item; the notes are the checklist's evidence that it works.

1. **History.** The gate has only ever seen the tip. Sweep every reachable
   commit for the same material before the first public push —
   `git log -p --all` through `scripts/leak-scan.sh FILE...` on a checkout of
   each commit, or a purpose-built history scanner. A hit means a rewrite
   (`git filter-repo`) *and* rotation of whatever it named; the commit is
   public the moment the repo is.

   The 2026-08-13 sweep read all 13,801 objects and hit: the operator address
   in 241 blobs of `docs/DESIGN.md`, operator-home paths in 562 places over 22
   files, and the drive logs bl-244f had deleted from the tip. **A filter was
   the wrong remedy and a squash was the right one**, because the same blobs
   also held verbatim operator dialogue and a third party's name — prose no
   pattern can find, so a filtered history would have been unverifiable. A
   squashed initial commit publishes exactly the tree the gate certifies. Its
   one real cost is that balls binds a task to the repo's ROOT COMMIT: a new
   root orphans every ball, `bl list` answers nothing while `bl list
   --everywhere` answers in full, and every live `tasks/*.md` needs its
   `root_commit` rewritten. Closed balls keep the old binding in store history.
2. **Other refs.** Tags, `speculation/**` branches left by a crashed
   `scripts/speculate-gate` driver, and any `work/bl-*` branch that was pushed.
   Delete what should never have been pushed rather than scanning it twice.
   **`balls/tasks` and `balls/config` are on this same remote and publish with
   it** — the task store is a ref, not a private sidecar (bl-dd1d). Run
   `store-scan` by `workflow_dispatch` rather than trusting the schedule was
   alive, and note that both the workflow and the store gate only ever see the
   TIP: the store's history is item 1's problem too, and rewriting it
   invalidates every existing clone.

   **A rewrite of an existing GitHub repository does not clean it, so do not
   plan one.** GitHub keeps `refs/pull/<n>/head` forever: closing the pull
   request and deleting its branch leaves the ref, and every object it reaches
   stays fetchable. Objects orphaned by a force-push stay reachable by sha as
   well. The only publication that can be certified is into a **fresh object
   store** — rename the private repository to an archive (nothing is deleted,
   the old refs and branches stay where they are), create the public one under
   the original name so `repository =` in Cargo.toml and balls' `task-remote`
   stay correct, and push only the refs that were scanned.

   Nothing about a repository travels with its NAME. The new one starts with no
   secrets (`CARGO_REGISTRY_TOKEN` must be set again or the release job cannot
   publish), and with "Allow GitHub Actions to create and approve pull
   requests" off, which 403s release-plz's PR job.
3. **Commit messages.** `.githooks/commit-msg` covers messages written with the
   hook seated (`make install-hooks`). Messages older than the hook, and any
   written elsewhere, are not covered.
4. **Repository text nobody committed.** Pull-request titles and bodies, issue
   text, review comments, release notes, and the crates.io description and
   metadata. None of it is in the tree.
5. **Actions logs and artifacts.** A failed gate prints paths, hostnames and
   sometimes the offending line into a public log; `.github/workflows/
   speculate.yml` also uploads a `verdicts` artifact. Both survive the run.
   A fresh repository starts with no run history, which is the other reason
   item 2's rename-and-recreate is cheaper than it looks.
6. **Already-published versions.** `cargo publish` is irreversible: a yanked
   version stays downloadable. Audit the packaged file list
   (`cargo package --list`) before `make publish CONFIRM=yes`, not after.

   This is the item that was learned the hard way. **0.0.1 was published on
   2026-07-26 carrying everything item 1 found** — the operator address, eleven
   operator-home paths across `src/world/mod.rs`, `src/opslog/tests.rs`,
   `src/binding/tests.rs` and `docs/DESIGN.md`, and three `docs/drive-logs/`
   files — because `Cargo.toml` declares no `include`/`exclude` and so packages
   the whole tree. It was yanked on 2026-08-13, which changes nothing about who
   can download it. A private repository is not a private crate: the moment a
   version ships, this checklist's items 1 and 6 are the same deadline.

## The merge queue — speculative closes, builds on GitHub Actions

The gate above costs minutes and closes serialize on it. yog rides balls'
speculative merge queue (balls `docs/design/bl-24e7-speculative-merge-queue.md`,
adopted in bl-1a5b): the gate consults a tree-keyed **verdict cache** first —
`scripts/pre-commit` exits in seconds when this exact worktree tree already
passed this exact gate — and speculative builds warm that cache ahead of the
queue, on GitHub Actions (`.github/workflows/speculate.yml`), so the local
machine never pays the build. After your last commit in the claim worktree:

    bl-speculate enqueue bl-XXXX                      # seal work/bl-XXXX into the queue
    bl-speculate run --gate scripts/speculate-gate    # builds run on GH Actions
    bl close bl-XXXX --as YOU                         # cache hit → seconds

Facts the queue derives from (do not fight them):

- **Sealing is the tag** `merging/bl-XXXX` on your tip. Any new commit unseals
  you; re-running `enqueue` re-seals at the *bottom* of the queue. That is the
  whole eviction mechanism.
- **A conflict or FAIL verdict ahead of you stops the chain** — the fix belongs
  to that branch's owner. Your close still works; it just pays the stock gate.
- **Everything degrades to the stock local gate.** No binary, no verdict, no
  network, no runner: the cache misses honestly and `bl close` builds locally.
  Never wait on the remote to close.
- **The gate fingerprint is content**: `scripts/pre-commit`,
  `scripts/check-line-lengths.sh`, `scripts/check-coverage.sh`, `Makefile`,
  plus local `rustc -V`. Editing any of them invalidates every stored verdict
  (deliberately). `rust-toolchain.toml` pins the toolchain on both sides —
  bump it in lockstep or remote verdicts silently stop matching.
- A crashed `scripts/speculate-gate` can strand a `speculation/<sha>` branch
  on origin; sweep with `git push origin --delete speculation/<sha>`.
