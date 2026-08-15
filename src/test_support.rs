//! Test-only spawn discipline shared by every test module in this binary.
//!
//! Serializes script-write-then-spawn pairs across tests. Without this, a
//! concurrent posix_spawn in another thread inherits the write fd held by
//! `fs::write` in this thread; that fd is CLOEXEC but only closes once the
//! peer's own exec completes. If this thread's exec on the script it just
//! wrote lands while the peer child still holds the inherited write fd,
//! Linux returns ETXTBSY. Holding one lock across write + spawn in every
//! test eliminates the overlap window — it must be a single static for the
//! whole binary: per-module locks do not exclude each other's threads.
//!
//! The lock is **re-entrant per thread** ([`SpawnGuard`]): the exclusion a
//! nested acquisition wants is already held by the outer one, and a plain
//! `Mutex` would self-deadlock instead. That is not hypothetical — a start
//! test holds the guard across its fake-binary write and the flow under test
//! forks `git` through [`spawn_locked`] (the model derives a workspace's
//! snapshot back out of the config commit lernie just authored).

use crate::config_edit::FileIo;
use crate::ui_state::Clock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

pub(crate) static SPAWN_LOCK: Mutex<()> = Mutex::new(());

thread_local! {
    /// How deep this thread already is inside [`spawn_guard`]. The lock is
    /// **re-entrant per thread**: a test holds it across a write+exec pair and
    /// the code under test forks again through [`spawn_locked`] (the start
    /// flow's `git` reads do exactly that), which on a plain `Mutex` is a
    /// self-deadlock. Depth makes the inner acquisition a no-op — the outer
    /// guard already provides the exclusion the inner one wants.
    static DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// The held spawn lock: `Some` at depth 0 (the real guard), `None` for a
/// re-entrant acquisition on the same thread. Dropping it unwinds the depth,
/// then releases the mutex — in that order, so the thread is out of the nest
/// before any peer can enter.
pub(crate) struct SpawnGuard {
    lock: Option<std::sync::MutexGuard<'static, ()>>,
}

impl Drop for SpawnGuard {
    fn drop(&mut self) {
        DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        drop(self.lock.take());
    }
}

/// Acquire `SPAWN_LOCK` poison-immune: a panicking test frees the guard with its
/// `()` intact, so recover rather than cascade-poison peer tests. Recovery stays
/// on one line — a split reads as uncovered under `ignore-panics` (the same
/// discipline as `state::lock_watchset`).
pub(crate) fn spawn_guard() -> SpawnGuard {
    if DEPTH.with(|d| d.replace(d.get() + 1)) > 0 {
        return SpawnGuard { lock: None };
    }
    SpawnGuard {
        lock: Some(SPAWN_LOCK.lock().unwrap_or_else(PoisonError::into_inner)),
    }
}

/// Deterministic [`Clock`] over a shared instant the test advances by hand, so
/// every debounce/sweep branch (§7.2) is exercised without sleeping. Handles
/// cloned via [`FakeClock::handle`] (or `Clone`) share one instant — advancing
/// any handle moves the clock every holder sees (a model and the test both read
/// it, and an [`AppModel`](crate::AppModel) hands one to its sweep schedule).
#[derive(Clone)]
pub(crate) struct FakeClock {
    at: Arc<Mutex<Instant>>,
    /// How far each [`Clock::now`] read moves the clock **by itself**. Zero for
    /// an ordinary fake, where only [`advance`](FakeClock::advance) moves time.
    /// Non-zero makes the work *between* two reads take real time — the one way
    /// to exercise §7.2's late-pass drift without a slow machine, since that
    /// branch is precisely "the clock moved while a pass ran".
    lurch: Duration,
}

impl FakeClock {
    pub(crate) fn new() -> Self {
        Self {
            at: Arc::new(Mutex::new(Instant::now())),
            lurch: Duration::ZERO,
        }
    }

    /// A clock where every read costs `lurch` — a machine under load, in a
    /// deterministic form.
    pub(crate) fn lurching(lurch: Duration) -> Self {
        Self {
            lurch,
            ..Self::new()
        }
    }

    /// A second handle sharing this clock's instant.
    pub(crate) fn handle(&self) -> Self {
        Self {
            at: Arc::clone(&self.at),
            lurch: self.lurch,
        }
    }

    /// This clock as a shared trait object for the `Arc<dyn Clock>` seam
    /// ([`AppModel`](crate::AppModel), `Schedule`). Shares the instant, so
    /// advancing the original still moves the clock the model holds.
    pub(crate) fn arc(&self) -> Arc<dyn Clock> {
        Arc::new(self.handle())
    }

    pub(crate) fn advance(&self, delta: Duration) {
        *self
            .at
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += delta;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        let mut at = self
            .at
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let read = *at;
        *at += self.lurch;
        read
    }

    /// A fixed wall-clock stamp. `ops.jsonl` treats `ts` as opaque (§4.2), so a
    /// constant is the deterministic reading — and it is the literal every
    /// drift/ops assertion in the suite already spells.
    fn stamp(&self) -> String {
        "TS".to_string()
    }
}

/// In-memory [`FileIo`] for editor and pipeline tests: a flat path→bytes map.
/// `fail_write` forces the write step to error (the `Io` Apply arm). Shared by
/// the brazen, lernie-global and pipeline test modules — one fake, one
/// behavior, so the write pipeline is exercised the same way everywhere.
#[derive(Default)]
pub(crate) struct FakeFs {
    pub(crate) files: Mutex<HashMap<PathBuf, Vec<u8>>>,
    pub(crate) fail_write: bool,
}

impl FakeFs {
    /// The backing map, locked (poison-immune — a lock a panicking peer test
    /// poisoned still yields the map, never a second panic).
    pub(crate) fn map(&self) -> std::sync::MutexGuard<'_, HashMap<PathBuf, Vec<u8>>> {
        self.files
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// A fake pre-populated with one file.
    pub(crate) fn seed(path: &Path, bytes: &[u8]) -> Self {
        let me = Self::default();
        me.map().insert(path.to_path_buf(), bytes.to_vec());
        me
    }

    /// The current bytes at `path`, if any.
    pub(crate) fn get(&self, path: &Path) -> Option<Vec<u8>> {
        self.map().get(path).cloned()
    }
}

impl FileIo for FakeFs {
    fn read(&self, path: &Path) -> std::io::Result<Option<Vec<u8>>> {
        Ok(self.get(path))
    }
    fn write(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        if self.fail_write {
            return Err(std::io::Error::other("boom"));
        }
        self.map().insert(path.to_path_buf(), bytes.to_vec());
        Ok(())
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        let bytes = self.map().remove(from).unwrap_or_default();
        self.map().insert(to.to_path_buf(), bytes);
        Ok(())
    }
    fn remove(&self, path: &Path) -> std::io::Result<()> {
        self.map().remove(path);
        Ok(())
    }
    fn exists(&self, path: &Path) -> bool {
        self.map().contains_key(path)
    }
    fn list_dir(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        Ok(self
            .map()
            .keys()
            .filter(|p| p.parent() == Some(dir))
            .cloned()
            .collect())
    }
}

/// Fork + exec `cmd` while holding [`SPAWN_LOCK`], releasing it once the
/// child has exec'd. The single fork-site discipline every test subprocess
/// routes through: the `git_tree` fixture's `run_git` and the production
/// `git_tree::cmd::git` under `cfg(test)`. Because `Command::spawn` returns
/// only after the child has exec'd (CLOEXEC then closes every inherited fd),
/// no fork is ever in flight while a recorder-script test holds a
/// not-yet-closed write fd, so that fd can't leak into a to-be-exec'd script
/// (the ETXTBSY race). The lock is released before the child is waited on, so
/// the subprocesses still run concurrently.
pub(crate) fn spawn_locked(
    cmd: &mut std::process::Command,
) -> std::io::Result<std::process::Child> {
    let _g = spawn_guard();
    cmd.spawn()
}

/// `providers.yaml` exactly as lernie's own `template/providers.yaml` authors
/// it (pinned lernie, `=0.0.8`) — what a materialized `lernie new` commits,
/// worker tool pool included: yog grants nothing on top (§8.1, bl-7fc8).
pub(crate) const TEMPLATE_PROVIDERS: &str = "roles:\n  worker:\n    provider: anthropic\n    \
     model: claude-sonnet-5\n    tools: [apply_patch, bash, cd, dispatch, load_skill, message, \
     multi_tool, read_file]\n  compactor:\n    provider: anthropic\n    model: claude-haiku-4-5\n";

/// The `new)` arm of a fake `lernie`: the workspace lernie ARCH §2.2 describes,
/// authored in shell — a bare `repo.git` whose orphan `config/default` root
/// carries [`TEMPLATE_PROVIDERS`]. Every fake `lernie` a start test drives
/// through shares this one arm.
pub(crate) fn authoring_new_arm() -> String {
    format!(
        "new)\n  mkdir -p \"$2/repo.git\"\n  \
         git init -q --bare -b config/default \"$2/repo.git\"\n  \
         git -C \"$2/repo.git\" worktree add -q --orphan -b config/default \"$2/.author\"\n  \
         printf '%s' '{TEMPLATE_PROVIDERS}' > \"$2/.author/providers.yaml\"\n  \
         git -C \"$2/.author\" add -A\n  \
         git -C \"$2/.author\" -c user.email=t@t.local -c user.name=T \
         -c commit.gpgsign=false commit -q -m 'config: init [config/default]'\n  \
         git -C \"$2/repo.git\" worktree remove \"$2/.author\"\n  ;;\n"
    )
}

/// Re-prime the directory at `path`: **the same path, guaranteed a different
/// inode** (bl-e492).
///
/// The obvious spelling — remove it, create it again — is a coin toss. A
/// filesystem is free to hand the just-freed inode straight back, and the CI
/// runners do: the two watcher tests that assert "a replaced root leaves a deaf
/// watcher" were reproducing nothing there, because the inode they replaced was
/// the inode they started with (`left: Some(9211169)  right: Some(9211169)`).
///
/// So the replacement is **allocated while the original is still linked** — two
/// live directories cannot share an inode, so the new one differs by
/// construction — and then renamed over the top. Nothing is left to the
/// allocator. It is also the truer reproduction: a re-primed clone is
/// materialized beside its target and moved into place, not built in the hole.
pub(crate) fn replace_directory(path: &std::path::Path) {
    let fresh = prepared_replacement(path);
    std::fs::remove_dir_all(path).expect("the original is unlinked");
    std::fs::rename(&fresh, path).expect("the replacement takes the path");
}

/// The first half of [`replace_directory`], for the test that must observe the
/// hole between the two: the replacement directory, created beside `path` while
/// `path` is still linked — which is the whole of what makes its inode differ.
pub(crate) fn prepared_replacement(path: &std::path::Path) -> std::path::PathBuf {
    let mut fresh = path.as_os_str().to_owned();
    fresh.push(".replacement");
    let fresh = std::path::PathBuf::from(fresh);
    std::fs::create_dir_all(&fresh).expect("the replacement is created beside the original");
    fresh
}

/// The hermetic fixture world and the workspace wall it stands in — its own
/// file at §12's cap, on the seam between faking an *effect* and composing a
/// *world* (bl-fcd5).
pub(crate) mod world;
pub(crate) use world::{fixture_workspace, no_wall, no_world, wall_paths, world_under};

/// A real lernie workspace on disk, for the tests that need one (§8.6's control
/// authoring and the start-flow abort it can raise). Its own file: the cap is a
/// tree-wide invariant, and this seeder is a self-contained fixture rather than
/// part of the spawn discipline above.
pub(crate) mod workspace;

/// The §9.5 wire's key material, minted at test runtime by the same
/// out-of-channel act an operator performs (REMOTE §1.4, bl-b6fa) — its own
/// file because a certificate fixture is never committed and the minting is a
/// self-contained seeder, not part of the spawn discipline above.
pub(crate) mod wire;
