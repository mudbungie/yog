//! Test-only spawn discipline for this binary — **one lock, held by the crate's
//! one fork** ([`crate::git_env::spawn`]), and by nothing else.
//!
//! `fs::write` on a fixture script holds a write fd. A `fork` in another thread
//! copies that fd into a child, which keeps it until its own `exec` completes;
//! an `exec` of the script inside that window is ETXTBSY, so a test that writes
//! a fake binary is reddened by a peer test three modules away. The lock is one
//! static for the whole binary because per-module locks do not exclude each
//! other's threads.
//!
//! It is taken **around the fork** (bl-6397). The older discipline asked every
//! test that wrote a script to bracket its own write and exec, which is a
//! contract each new test must be told about and most were not — the two tests
//! that flaked were one of each kind: a victim that held the guard correctly,
//! and an unguarded sign-in fixture that was both victim and cause. Guarding
//! the fork alone is *sufficient*: a peer fork cannot land inside anyone's
//! write window without holding the lock, and it returns the lock only once its
//! child has exec'd. Measured with writes left entirely unguarded, 8
//! write-then-exec threads against an 8-thread fork storm, ~9,600 pairs: zero
//! ETXTBSY, against 8.3% with the fork unguarded.
//!
//! The write-side brackets tests used to hold are gone with the contract, and
//! they were not merely redundant: a test that holds the lock across a body
//! whose WORKER THREAD forks starves that fork for the whole test — the wire
//! host's tool span, which this change caught the moment the fork started
//! taking the lock. Write the script, exec it: the fork boundary holds, and
//! nothing in a test body needs to know it is there.

use crate::config_edit::FileIo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

pub(crate) static SPAWN_LOCK: Mutex<()> = Mutex::new(());

/// Acquire `SPAWN_LOCK` poison-immune: a panicking fork frees the guard with
/// its `()` intact, so recover rather than cascade-poison peer tests. Recovery
/// stays on one line — a split reads as uncovered under `ignore-panics` (the
/// same discipline as `state::lock_watchset`).
pub(crate) fn spawn_guard() -> std::sync::MutexGuard<'static, ()> {
    SPAWN_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// In-memory [`FileIo`] for editor and pipeline tests: a flat path→bytes map.
/// `fail_write` forces the write step to error (the `Io` Apply arm). Shared by
/// the brazen, litany-global and pipeline test modules — one fake, one
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
    fn list_dir(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        Ok(self
            .map()
            .keys()
            .filter(|p| p.parent() == Some(dir))
            .cloned()
            .collect())
    }
}

/// `providers.yaml` exactly as litany's own `template/providers.yaml` authors
/// it (the pinned engine) — what a materialized `litany new` commits,
/// worker tool pool included: yog grants nothing on top (§8.1, bl-7fc8).
pub(crate) const TEMPLATE_PROVIDERS: &str = "roles:\n  worker:\n    provider: anthropic\n    \
     model: claude-sonnet-5\n    tools: [apply_patch, bash, cd, dispatch, load_skill, message, \
     multi_tool, read_file]\n  compactor:\n    provider: anthropic\n    model: claude-haiku-4-5\n";

/// The `new)` arm of a fake `litany`: the workspace litany ARCH §2.2 describes,
/// authored in shell — a bare `repo.git` whose orphan `config/default` root
/// carries [`TEMPLATE_PROVIDERS`]. Every fake `litany` a start test drives
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

/// A real litany workspace on disk, for the tests that need one (§8.6's control
/// authoring and the start-flow abort it can raise). Its own file: the cap is a
/// tree-wide invariant, and this seeder is a self-contained fixture rather than
/// part of the spawn discipline above.
pub(crate) mod workspace;

/// The §9.5 wire's key material, minted at test runtime by the same
/// out-of-channel act an operator performs (REMOTE §1.4, bl-b6fa) — its own
/// file because a certificate fixture is never committed and the minting is a
/// self-contained seeder, not part of the spawn discipline above.
/// **The suite's own seat** — the client half of the wire, which the crate no
/// longer ships (bl-7942) and its own tests must still speak to prove the
/// listener.
pub(crate) mod engine;
pub(crate) mod seat;
pub(crate) mod wire;

/// The §11 accessories that crossed with bl-296f — the altitude-0 chrome and
/// the selection's own detail — asked through the boundary, there being no
/// model accessor left to ask instead.
pub(crate) mod chrome;

/// The deterministic [`Clock`] every debounce and sweep branch is exercised
/// against — its own file at §12's cap, on the seam this file already had:
/// faking a *value* the crate reads, rather than the spawn discipline above.
pub(crate) mod clock;
pub(crate) use clock::FakeClock;
