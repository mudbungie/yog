//! Start-flow tests (§15 M6 Z3), split by concern for the 300-line cap: [`plan`]
//! (the pure planner tables per rung), [`goal`] (preambles, prefills, the driver
//! cwd, the preview), [`identity`] (the §3.3 stamp, its inverses and the mint), [`exec`] (the `bl`-facing executors + their
//! non-spawn aborts), [`ensure`] (the `lernie new` ensure, the mint and the
//! worktree ladder), and [`run`] ([`prepare`] end-to-end per rung). Shared
//! fixtures live here.

mod control;
mod ensure;
mod exec;
mod goal;
mod identity;
mod plan;
mod prompt;
mod run;

use crate::opslog::{self, OpEntry};
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, StartInputs};
use crate::test_support::authoring_new_arm;
use lernie::mint::SplitMix64;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

/// Write an executable script at `dir/name` (0755) and return its path.
pub(super) fn write_exec(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

/// A fake `bl` printing `id` for `create` and `worktree` for `claim`, exit 0.
pub(super) fn fake_bl(dir: &Path, id: &str, worktree: &Path) -> PathBuf {
    let body = format!(
        "#!/bin/sh\ncase \"$1\" in\ncreate) printf '%s\\n' '{id}';;\nclaim) printf '%s\\n' '{wt}';;\nesac\nexit 0\n",
        wt = worktree.display(),
    );
    write_exec(dir, "bl", &body)
}

/// A fake `lernie` that **materializes** its markers like the real one, so a
/// re-plan converges on disk (§8.1): `prime` writes `$LERNIE_HOME/models.yaml`
/// (the seed marker) and `new` authors the ARCH §2.2 workspace through the
/// shared [`authoring_new_arm`]. Other verbs exit 0.
pub(super) fn fake_lernie(dir: &Path) -> PathBuf {
    write_exec(
        dir,
        "lernie",
        &format!(
            "#!/bin/sh\ncase \"$1\" in\nprime) [ -n \"$LERNIE_HOME\" ] && mkdir -p \"$LERNIE_HOME\" \
             && : > \"$LERNIE_HOME/models.yaml\";;\n{}esac\nexit 0\n",
            authoring_new_arm()
        ),
    )
}

/// A fake binary that exits non-zero with `msg` on stderr — the failed-verb path.
pub(super) fn fake_fail(dir: &Path, name: &str, msg: &str) -> PathBuf {
    write_exec(
        dir,
        name,
        &format!("#!/bin/sh\nprintf '%s\\n' '{msg}' 1>&2\nexit 3\n"),
    )
}

/// The hermetic effect world: a dir for fake binaries, a yog state root for
/// `ops.jsonl`, the balls-state + yog-data + home roots the path formulas derive
/// from, and a real project dir (the `bl` cwd).
pub(super) struct World {
    pub(super) bin: TempDir,
    pub(super) state: TempDir,
    pub(super) balls: TempDir,
    pub(super) yog: TempDir,
    pub(super) home: TempDir,
    pub(super) project: TempDir,
}

impl World {
    pub(super) fn new() -> Self {
        Self {
            bin: tempdir().unwrap(),
            state: tempdir().unwrap(),
            balls: tempdir().unwrap(),
            yog: tempdir().unwrap(),
            home: tempdir().unwrap(),
            project: tempdir().unwrap(),
        }
    }

    /// The logged `ops.jsonl` entries, oldest-first.
    pub(super) fn ops(&self) -> Vec<OpEntry> {
        opslog::tail(self.state.path(), 16)
    }

    /// The `argv[1]` verb of each logged op, oldest-first — the order assertion.
    pub(super) fn verbs(&self) -> Vec<String> {
        self.ops()
            .into_iter()
            .filter_map(|e| e.argv.get(1).cloned())
            .collect()
    }

    /// The materializing fake `lernie` as a `Cli` standing the world's
    /// `LERNIE_HOME`, so `prime` writes the seed marker where [`seed::seeded`]
    /// reads it — a re-plan then converges (`prime`/`new` skip) exactly as in
    /// production.
    pub(super) fn lernie(&self) -> crate::cli_outbound::Cli {
        let home = crate::world::layout_under(self.yog.path()).lernie;
        crate::cli_outbound::Cli::new(fake_lernie(self.bin.path())).with_env(vec![(
            "LERNIE_HOME".to_owned(),
            home.to_string_lossy().into_owned(),
        )])
    }

    /// A [`StartInputs`] over this world carrying `payload` into the workspace
    /// named `name` under this world's names root (§3.1: the name is the leaf).
    pub(super) fn inputs(&self, name: &str, payload: Payload) -> StartInputs {
        StartInputs {
            workspace: crate::binding::workspace_path(self.yog.path(), name),
            payload,
            home: self.home.path().to_path_buf(),
            yog_data_root: self.yog.path().to_path_buf(),
            balls_state_root: self.balls.path().to_path_buf(),
            conversation_names: Vec::new(),
        }
    }
}

/// A fixed RNG seed for deterministic conversation mints across a test's preview
/// and its fire (§3.3).
pub(super) fn rng() -> SplitMix64 {
    SplitMix64::from_seed(0x5eed)
}

/// An existing-ball payload at the given join state.
pub(super) fn ball(project: &Path, id: &str, join: JoinState) -> Payload {
    Payload::Ball {
        project: project.to_path_buf(),
        ball: BallSpec::Existing {
            id: id.to_owned(),
            title: "T".to_owned(),
            body: "B".to_owned(),
            join,
        },
    }
}
