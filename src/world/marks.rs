//! **The agent's balls space** (DESIGN §16.3, the per-agent ruling):
//! where one agent's task tracking lives, and the one var that carries it.
//!
//! The ruling: by default each agent gets its own balls branch for tracking;
//! an agent's branch can be set at launch; subagents are passed their parent's
//! space by default; and an agent can amend its own branch to change its
//! config. A **space** is the whole of that: balls' state home (the
//! clone bundle — landing, store checkout, worktrees) and balls' config home
//! (the §4 layer-2 `config.toml` that names the store branch), together.
//!
//! **One var carries it: `YOG_MARKS`**, layered onto an agent's spawn exactly
//! as [`YOG_WALL`](super::wall::YOG_WALL) is, so the whole descendant tree
//! inherits it — that IS the subagent clause, with no mechanism of its own.
//!
//! - **Absent = the world's space** ([`Space::world`]): balls' state stays
//!   `<world>/state` (where every clone yog's board reads already lives) and
//!   its config home becomes `<world>/config`. yog's own `bl` verbs, the §5.1
//!   #2 board reads, and every agent *pointed at a project* run here, so the
//!   project's board is one store and stays instantly consistent.
//! - **Present = the agent's own space** ([`Space::own`], `<wall>/marks`): a
//!   private clone bundle AND a private balls config home, so that agent's task
//!   churn shares nothing with the project's board — the default the ruling
//!   asks for.
//!
//! **`<world>/config` is not cosmetic — it closes a real leak.** balls reads
//! `$XDG_CONFIG_HOME/balls/config.toml` (a layer that OUTRANKS the landing, so
//! it decides `tasks_branch`) and `$XDG_CONFIG_HOME/balls/default-config/` (the
//! template `bl prime` founds a landing from). §16.2 nested balls' state and
//! left `XDG_CONFIG_HOME` ambient, so both resolved to the operator's own
//! `~/.config/balls` — and a stale `default-config` there (one naming the
//! retired `tracker` plugin) made every landing yog founded prune its whole
//! plugin schedule: no `bl-tracker` at any phase, so yog's stores never fetched
//! and never pushed. Nesting the config home is what puts yog outside that
//! blast radius (bl-e47b's investigation).
//!
//! **The branch is written into the space's own `config.toml`, in balls'
//! schema, and read back from it.** It cannot be `bl conf set task-branch`:
//! that write is scope-keyed to the LANDING, and a landing is per *clone* —
//! i.e. per invocation path — so it binds a project, not an agent, and an agent
//! that runs `bl` in three directories would need three writes. balls' layer-2
//! config is the one place a value covers every clone in a space, which is what
//! "the agent's branch" means. It stays balls' own file, balls' own key and
//! balls' own precedence — yog stores no setting of its own shape, and `bl
//! conf` remains the authority on what resolved (it reports the winning layer
//! by name, `xdg`). Severability holds: deleting the space deletes the policy.

use std::io;
use std::path::{Path, PathBuf};

use crate::opslog::{self, OpEntry};
use crate::xdg::Env;

/// The one var naming an agent's own balls space (§16.3). Absent = the world's
/// space, which is the project's board — the space every agent pointed at a
/// project uses, and the one yog's own verbs and reads run in.
pub const YOG_MARKS: &str = "YOG_MARKS";

/// balls' default store branch — the project's stable contract (`balls::
/// DEFAULT_TASKS_BRANCH`), and what a space with nothing written reads as.
pub const SHARED_BRANCH: &str = "balls/tasks";

/// balls' §4 layer-2 config file under a config home: `<home>/balls/config.toml`
/// (`balls::layout::Xdg::user_config`, reproduced as path algebra so the fold is
/// pure). The single home of a space's store branch.
pub fn config_file(config_home: &Path) -> PathBuf {
    config_home.join("balls").join("config.toml")
}

/// balls' two home directories for one space (§16.3) — state (the clone bundle)
/// and config (the layer-2 `config.toml` + the seed template). Owned and
/// concrete; every consumer hands them to `balls::layout::Xdg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    /// `$XDG_STATE_HOME` as balls sees it — clones, worktrees, op logs.
    pub state: PathBuf,
    /// `$XDG_CONFIG_HOME` as balls sees it — `balls/config.toml`, `balls/default-config/`.
    pub config: PathBuf,
}

impl Space {
    /// The world's space: balls' state stays exactly where §16.2 put it (so
    /// every clone yog already founded is still the one it reads), and its
    /// config home nests under the world root beside it.
    pub fn world(world_root: &Path, state_home: &Path) -> Space {
        Space {
            state: state_home.to_path_buf(),
            config: world_root.join("config"),
        }
    }

    /// An agent's own space: one root serving as both homes, so the whole of
    /// that agent's balls existence is one directory — deleting it is the whole
    /// severability story, and §3.6's unmaking takes it with the wall.
    pub fn own(root: &Path) -> Space {
        Space {
            state: root.to_path_buf(),
            config: root.to_path_buf(),
        }
    }

    /// The store branch this space tracks on: `tasks_branch` from balls' own
    /// layer-2 config, else balls' default. An unreadable or malformed file
    /// reads as the default — balls itself surfaces a parse error on every op,
    /// so this stays quiet rather than double-reporting.
    pub fn branch(&self) -> String {
        let text = std::fs::read_to_string(config_file(&self.config)).unwrap_or_default();
        parse_branch(&text).unwrap_or_else(|| SHARED_BRANCH.to_owned())
    }
}

/// The `tasks_branch` value in a balls layer-2 config body, if it names one.
/// The body is balls' TOML, but yog authors it and [`lawful`] confines a branch
/// to characters no TOML escape can reach — so the read is the same one line
/// back, not a parser yog would have to take a dependency for.
fn parse_branch(text: &str) -> Option<String> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == "tasks_branch")
        .map(|(_, value)| value.trim().trim_matches(QUOTE).to_owned())
        .filter(|v| !v.is_empty())
}

/// Is `branch` a lawful store branch to write (§16.3)? A git ref name with no
/// whitespace and no TOML-escaping character, and never balls' own landing
/// branch — which balls refuses outright ("one branch cannot back two
/// checkouts"), so refusing it here states the same invariant one step earlier,
/// at the field, in §3.1's idiom.
pub fn lawful(branch: &str) -> bool {
    !branch.is_empty()
        && branch != LANDING_BRANCH
        && !branch.contains(char::is_whitespace)
        && !branch.contains([QUOTE, '\\'])
}

/// balls' landing branch (`balls::LANDING_BRANCH`) — the one name a store
/// branch may never take.
const LANDING_BRANCH: &str = "balls/config";

/// The double quote, spelled by codepoint: a bare char literal of it opens a
/// string as far as the §12 citation sweep's scanner is concerned, and one
/// escape hatch is cheaper than teaching that scanner char literals.
const QUOTE: char = '\u{22}';

/// An agent's own space root under its workspace wall: `<wall>/marks`. The wall
/// is already the sphere's one private layer (§16.2), and the §3.1 name that
/// keys it is the same name that is the ball claimant (§3.2) — so the claimant
/// and the space it claims into are one fact, never two that can disagree.
pub fn own_root(world: &Env, workspace: &Path) -> PathBuf {
    super::wall::root_of(world, workspace).join("marks")
}

/// The space standing in `env` (§16.3): `YOG_MARKS` when an agent's own space
/// is layered on, else the world's. The one resolution, used by every read and
/// by the embedded `bl` arm alike, so the space yog reads and the space an
/// agent's `bl` writes are one answer.
pub fn space(env: &Env) -> Space {
    match env.var(YOG_MARKS) {
        Some(root) => Space::own(Path::new(&root)),
        None => Space::world(&super::layout(env).root, &env.balls_state_home()),
    }
}

/// The spawn layer for an agent launched onto its **own** space (§16.3): the
/// `(var, value)` pairs layered on top of the world's overrides and the wall's,
/// through the same [`Cli::and_env`](crate::cli_outbound::Cli::and_env) seam.
/// Empty for a launch pointed at a project — that agent's `bl` is the board's
/// own, and an absent var is exactly that.
pub fn pairs(world: &Env, workspace: &Path, own: bool) -> Vec<(String, String)> {
    if !own {
        return Vec::new();
    }
    vec![(
        YOG_MARKS.to_owned(),
        own_root(world, workspace).to_string_lossy().into_owned(),
    )]
}

/// Read a workspace's tracking branch (§16.3) — the state `/marks` reports and
/// the pane renders. Never spawns: the value's one home is the space's own
/// config file, and an unfounded project (or no project at all) is no obstacle,
/// which is what makes the launched-then-pointed-at-a-project case answerable.
pub fn read(world: &Env, workspace: &Path) -> Space {
    Space::own(&own_root(world, workspace))
}

/// Point a workspace's own space at `branch` (§16.3): write `tasks_branch` into
/// balls' layer-2 config for that space, log the write to `ops.jsonl` (§4.2, the
/// mutation-logging discipline), and hand back the branch **re-read** — what
/// landed, never an echo of what was asked.
pub fn apply(space: &Space, state_root: &Path, ts: &str, branch: &str) -> io::Result<String> {
    let path = config_file(&space.config);
    let outcome = if lawful(branch) {
        write_branch(&path, branch)
    } else {
        Err(io::Error::other(REFUSAL))
    };
    log_op(state_root, ts, &path, branch, &outcome)?;
    outcome?;
    Ok(space.branch())
}

/// The refusal an unlawful branch earns, said once — the grammar states it
/// before dispatch and [`apply`] states it again at the write, so a typed line
/// and a forced call cannot word the same fact differently.
pub const REFUSAL: &str =
    "name a store branch: one word, no quotes, and not balls' own landing branch (balls/config)";

/// Write `tasks_branch = "<branch>"` as the space's whole layer-2 config. The
/// file is yog's to author in full: a space is one agent's, and its only balls
/// config is the branch — so a merge-preserving edit would be machinery for a
/// key nothing else ever writes.
fn write_branch(path: &Path, branch: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, body(branch))
}

/// The config body, said once — the write emits it and [`parse_branch`] reads
/// it back. One key, quoted; [`lawful`] has already refused anything a quote
/// would have to escape.
pub fn body(branch: &str) -> String {
    format!("tasks_branch = \"{branch}\"\n")
}

/// Append the write's outcome to `ops.jsonl` (§4.2). A file write is not a
/// spawn, so it rides the §4.2 non-spawn step shape the start flow's own
/// `yog-step` rows use — the path is the subject, the exit says whether it
/// landed.
fn log_op(
    state_root: &Path,
    ts: &str,
    path: &Path,
    branch: &str,
    outcome: &io::Result<()>,
) -> io::Result<()> {
    let (exit, stderr) = match outcome {
        Ok(()) => (0, String::new()),
        Err(e) => (-3, e.to_string()),
    };
    opslog::append(
        state_root,
        &OpEntry {
            ts: ts.to_owned(),
            argv: vec!["yog-step".to_owned(), "marks".to_owned(), branch.to_owned()],
            cwd: path.display().to_string(),
            exit,
            stdout: String::new(),
            stderr,
            // The §16.3 knob's own pane states this outcome in place (§7.3,
            // bl-48f8), so no banner elsewhere repeats it.
            origin: crate::opslog::Origin::World,
        },
    )
}

#[cfg(test)]
mod tests;
