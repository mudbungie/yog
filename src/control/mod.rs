//! The **capability control** (VISION §4.11, DESIGN §8.6): the executable
//! lernie's tool-control seam consults before every granted tool invocation
//! executes, and everything it reads to answer.
//!
//! **The enforcement point already shipped upstream.** Pinned lernie carries the
//! seam (its ARCH §3.3 *Tool control*): `workflow.yaml`'s `tool_control:` names
//! one binary, hands it the `tool_use` block plus the calling role and agent id
//! on stdin, and reads one verdict — `pass`, `refuse`, `hold` — off its stdout,
//! failing closed. yog's whole job is to *be* that binary. No new primitive is
//! asked of anyone, and role grants stay exactly as lernie ships them: grants
//! are lernie's structure, this is yog's policy.
//!
//! Two moves per consult, and nothing else:
//!
//! 1. **Classify** the invocation into the effect vocabulary ([`classify`]) —
//!    invocations, never tool names, because `bash` is every class at once.
//! 2. **Judge** the class by the shipped table folded with the operator's own
//!    answers ([`judge`]).
//!
//! **It writes nothing, ever.** A hold is released by re-adjudication on the
//! next drive, so a consult with a side effect would answer differently the
//! second time; and every fact it needs already has a durable home elsewhere —
//! lernie's hold mark, yog's ops trail, the shipped defaults. That is also why
//! it never calls stop: a stop mid-tool-window wedges the branch permanently
//! (lernie's own bl-b98d), so a refusal is an in-band decline the model steps
//! past and an "ask" is a park that costs no process and no tokens. There is no
//! modal in either frontend, and no attended/unattended split — the attention
//! item is answered in seconds or in hours, so attendance is latency, not a
//! mode.
//!
//! **What this is not.** Not confinement. The ambient `PATH` rides beneath the
//! world's prepend, the network is unconfined, and brazen's credentials are
//! shared by deliberate ruling (§16.2). Rule classification bounds accident and
//! drift; adversarial evasion is the OS layer's problem, later and
//! platform-explicit.

use balls::layout::Xdg as BallsXdg;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::opslog;
use crate::xdg::Env;

pub mod author;
pub mod bash;
pub mod classify;
pub mod hold;
pub mod judge;
pub mod lex;
pub mod policy;
pub mod root;
pub mod rules;
pub mod wire;

use classify::Classified;
use judge::Answers;
use policy::Policy;
use root::Root;
use wire::{Request, Verdict};

/// The multi-call subcommand the `world/tools/` shim re-execs yog under
/// (§8.6). Not a multiplex namespace and not something a human types: lernie
/// spawns the control with **no argv at all**, so this word is the whole of its
/// command line. `main.rs` answers it at the process edge beside the other
/// multi-call subcommands, binding the process's real stdin and stdout to
/// [`run`] — the same shape `--editor-apply` and the two world hatches take,
/// and the reason [`run`] takes its streams rather than reaching for them.
pub const SUBCMD: &str = "tool-control";

/// Exit code for a request the control cannot read at all. The seam fails
/// closed on a non-zero exit, which is the right answer to a broken protocol:
/// an invocation nobody could adjudicate must not execute.
const UNREADABLE: i32 = 2;

/// Everything one consult reads, resolved. Owned so the judgment is a pure
/// function of it — the seam of every test below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consult {
    /// The workspace root — the control's own cwd, per lernie's contract.
    pub workspace: PathBuf,
    /// balls' own layout — where the bl-delivery formula is rooted, and where
    /// the §4.10 attempt formula places a fan candidate's worktree. balls'
    /// value, not a mirror of it (§16.7 W8), so both formulas are asked of the
    /// crate that owns them.
    pub balls: BallsXdg,
    /// yog's state root, holding the `ops.jsonl` this fold reads.
    pub state_root: PathBuf,
    /// `$HOME`, for `~` in operands.
    pub home: PathBuf,
    /// The agent's working directory as lernie's own mark reports it; `None`
    /// for an agent that never moved, whose tools run in its worktree.
    pub cwd: Option<PathBuf>,
    /// The workspace's standing policy at its **live** config tip — the
    /// operator's overrides of the shipped table, ruleset and secret list.
    /// Default is the shipped state, so a workspace that declares nothing is
    /// adjudicated exactly as one that has never been edited (§8.6
    /// severability).
    pub policy: Policy,
}

impl Consult {
    /// Resolve a consult from the composed world env and the workspace lernie
    /// named. Pure — the two disk reads (the cwd mark, the policy file) are the
    /// caller's.
    pub fn new(env: &Env, workspace: &Path, cwd: Option<PathBuf>, policy: Policy) -> Consult {
        Consult {
            workspace: workspace.to_path_buf(),
            balls: env.balls_layout(),
            state_root: env.yog_state_root(),
            home: env.home_dir(),
            cwd,
            policy,
        }
    }

    /// The writable root and cwd for `agent_id`, over the trail's claim rows.
    fn root(&self, agent_id: &str, entries: &[opslog::OpEntry]) -> Root {
        let agent = root::agent_worktree(&self.workspace, agent_id);
        let claimant = self
            .workspace
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut writable = vec![agent.clone()];
        writable.extend(root::bound_worktrees(
            entries,
            &self.balls.state_dir(),
            &claimant,
        ));
        // …and the §4.10 fan's candidates beside it: with N > 1 the work does
        // not happen in `work/<id>` at all, it happens in each candidate's own
        // attempt worktree, so a root that named only the claim would refuse
        // every write a fanned drone makes. Derived from the same trail and the
        // same claim — yog's own rows, never the agent's mark (`root`).
        writable.extend(root::candidate_worktrees(
            entries,
            &self.balls,
            &self.workspace,
            &claimant,
        ));
        Root {
            cwd: self.cwd.clone().unwrap_or(agent),
            writable,
            home: self.home.clone(),
        }
    }
}

/// Adjudicate one invocation: classify it, then judge the class against the
/// shipped table folded with the operator's own answers. Pure over `consult`
/// plus one read of the ops trail.
pub fn adjudicate(consult: &Consult, request: &Request) -> Verdict {
    let entries = opslog::tail(&consult.state_root, usize::MAX);
    let root = consult.root(&request.agent_id, &entries);
    let classified = classify::classify(request, &root, &consult.policy);
    let ruling = Answers::fold(&entries).ruling(
        &request.id,
        &request.agent_id,
        classified.effect,
        &consult.policy,
    );
    ruling.verdict(&reason(request, &classified))
}

/// How many `char`s of the invocation's input the reason carries. Enough to
/// recognise the command; bounded because the sentence rides a git blob an
/// operator reads at a glance.
const SUMMARY_MAX: usize = 160;

/// The sentence a hold hands the operator and a refusal hands the model: the
/// tool, **what it was about to do**, the class it landed in, and the evidence
/// that put it there. Never a section number — the reader has the window, not
/// the document.
///
/// The input summary lives here rather than at the attention item because the
/// control is the only thing that sees the invocation: the mark carries the
/// sentence, so the parked drone's whole story is one fact with one home, and
/// the operator never opens a transcript to learn what is waiting.
pub fn reason(request: &Request, classified: &Classified) -> String {
    format!(
        "{} {} classified {} ({})",
        request.name,
        clip(&request.input.to_string()),
        classified.effect.word(),
        classified.why,
    )
}

/// `text` bounded to [`SUMMARY_MAX`] chars, saying so when it was cut.
fn clip(text: &str) -> String {
    let flat = text.replace(['\n', '\r'], " ");
    if flat.chars().count() > SUMMARY_MAX {
        flat.chars().take(SUMMARY_MAX).chain("…".chars()).collect()
    } else {
        flat
    }
}

/// The `world/tools/` shim's process body: read one request, answer one verdict.
/// Exit 0 on any answer — including a refusal, which is an answer — and
/// [`UNREADABLE`] only when the request itself could not be read, which the seam
/// then fails closed on.
pub fn run(stdin: &mut dyn Read, stdout: &mut dyn Write, env: &Env, workspace: &Path) -> i32 {
    let mut raw = String::new();
    if stdin.read_to_string(&mut raw).is_err() {
        return UNREADABLE;
    }
    let Some(request) = Request::parse(&raw) else {
        return UNREADABLE;
    };
    let cwd = root::agent_cwd(workspace, &request.agent_id);
    // The policy is read **here**, per consult, off the config lineage's live
    // tip: the control acts for the operator, so a revocation written a second
    // ago binds on the very next invocation (§8.6).
    let policy = Policy::read(workspace);
    let verdict = adjudicate(&Consult::new(env, workspace, cwd, policy), &request);
    if writeln!(stdout, "{}", verdict.json()).is_err() {
        return UNREADABLE;
    }
    0
}

/// The workspace the control was consulted about: lernie's own env var, else the
/// process cwd — which lernie also sets to the workspace root, so the fallback
/// is the same fact by its other spelling.
pub fn workspace_of(env: &Env) -> PathBuf {
    env.lernie_conv_repo()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

#[cfg(test)]
mod tests;
