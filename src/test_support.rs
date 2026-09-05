//! Test-only scaffolding for this binary: the fake effects and the fixture
//! world — and the suite's door onto the executable-file discipline
//! ([`fixture::write_exec`], a panic over [`crate::git_env::write_exec`]).
//!
//! There used to be a spawn lock here too, and the story of why it is gone is
//! the discipline. `fs::write` on a script holds a write fd; a `fork` in
//! another thread copies it into a child that keeps it until its own `exec`
//! completes; an `exec` of the script inside that window is ETXTBSY. bl-6397
//! answered from the fork side — one process-wide lock across
//! [`crate::git_env::spawn`], measured at zero against 8.3% unguarded — and
//! that held only while every fork in the process was yog's own. It was not:
//! the linked `balls`/`litany`/`brazen` fork `git` themselves and took no lock
//! of ours, so a beat driving one in-process reopened the window (bl-6bf5
//! measured 8 failures, bl-fd28 another 2).
//!
//! bl-fd28 moved the exposure instead of scheduling around it, and bl-e6c9
//! found the same hazard standing in the ENGINE — `world::tools::ensure_shim`
//! wrote a shim yog then exec'd — so the helper is production's now and this
//! module only spells it without a `Result`. Everything about the hazard, the
//! two measurements and the shapes that were rejected is stated once, in
//! [`crate::git_env::write_exec`]'s module doc.
//!
//! No lock is left in this module's own right: the `Mutex` below is a test
//! double's interior mutability, not serialization (`rules/locks-outside-state.yml`).

use crate::config_edit::FileIo;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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
     python, read_file, search_history]\n  compactor:\n    provider: anthropic\n    model: claude-haiku-4-5\n";

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
pub(crate) use world::{fixture_workspace, no_wall, no_world, signed, wall_paths, world_under};

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

/// The suite's spelling of [`crate::git_env::write_exec`] (bl-fd28, bl-e6c9),
/// plus the two narrow mode helpers for the "this file cannot be rewritten"
/// fixture — the one location `rules/no-hand-chmod.yml` still lets a mode bit
/// be set from.
pub(crate) mod fixture;
pub(crate) use fixture::write_exec;

/// The deterministic [`Clock`] every debounce and sweep branch is exercised
/// against — its own file at §12's cap, on the seam this file already had:
/// faking a *value* the crate reads, rather than the spawn discipline above.
pub(crate) mod clock;
pub(crate) use clock::FakeClock;
