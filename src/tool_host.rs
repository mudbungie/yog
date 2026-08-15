//! **yog's tool injection** (REMOTE §5, bl-c907): the object the `yog lernie`
//! arm hands lernie at `Fx::tool_injection`, so an agent can see this
//! workspace's client machines and drive the tools they advertise.
//!
//! lernie 0.0.9's seam (its `docs/DESIGN_TOOL_INJECTION.md`) is **one object
//! carrying both halves** — the definitions prompt assembly and the grant gate
//! read, and the router the executor consults ahead of binary resolution — so a
//! tool declared and not permitted, or permitted and not declared, is
//! unrepresentable. yog fills it with:
//!
//! - [`clients`] — ONE tool in the stable prefix, always. Its subject is the
//!   roster, and `load` is the act that makes a host's tools callable.
//! - [`loaded`] — the agent's durable loaded set, read at every assembly, each
//!   entry surfacing as an **individually named** tool. Never a multiplexer:
//!   lernie's `docs/DESIGN_MCP_BRIDGE.md` §6 ruling binds a host too, so the
//!   grant gate, the tool control and any future policy keep seeing one name
//!   per capability.
//!
//! **It runs in the driver, and the driver is a child process.** Presence is
//! engine RAM by ruling (REMOTE §5), so the roster is asked for through the
//! deposit inbox REMOTE §3 reserves for the world's own residents ([`ask`]) —
//! no new verb, no new transport, and the same `Query::Clients` every seat
//! reads. [`Injection::tools`], by contrast, touches nothing but disk: a prefix
//! that changed when an engine was slow would be a connectivity-rate fact
//! inside the model's cached context, which is what REMOTE §5 was amended to
//! exclude.
//!
//! **Adjudication is untouched and still runs first.** `yog tool-control`
//! (DESIGN §8.6) is consulted before the executor routes anything, so a routed
//! invocation is judged exactly as a local one — and lernie is honest that what
//! happens on the far machine is beyond the adjudicator's reach (REMOTE §5).
//!
//! **The residual, stated where it is felt.** The leg that carries an
//! invocation down a tool host's live connection is not built (REMOTE §9 step
//! 7, filed as bl-024b). A loaded name is declared, adjudicated and routed —
//! and the routing answers a non-zero in-band capture naming the missing leg,
//! which is the shape a vanished endpoint already had to produce (lernie's
//! §3.3: *"A vanished endpoint is an in-band error result, never a hang"*).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ::lernie::cmd::{InjectedTool, RoutedCall, RoutedCapture, ToolInjection};

use crate::ui_state::{Clock, iso8601_extended};

/// The deposit round trip a driver asks its engine through.
pub mod ask;
/// The `clients` tool: the roster, and the act that loads from it.
pub mod clients;
/// The agent's durable loaded set.
pub mod loaded;
/// The dated observations the tool appends to context.
pub mod render;

/// The loaded-set root's leaf under yog's state root, and the workspace/agent
/// an answer is about — everything a `clients` op needs beyond the invocation
/// itself.
///
/// It is built per call from [`RoutedCall`]'s own workspace and agent, so the
/// router never depends on what the process was launched to drive.
pub struct Site {
    /// Where the gestures inbox and the loaded sets live (`<world>/state/yog`).
    pub state_root: PathBuf,
    /// The workspace's name (§3.1: its directory leaf).
    pub workspace: String,
    /// The calling agent's id (§2.3).
    pub agent: String,
    /// How long an engine ask may take.
    pub budget: ask::Budget,
    /// The observation stamp's source.
    pub clock: Arc<dyn Clock>,
}

impl Site {
    /// The instant this answer is true at, in the crate's one human spelling.
    /// An unparseable stamp reads as the epoch rather than refusing: a wrong
    /// date on an observation is worth less than the observation.
    pub fn observed(&self) -> String {
        iso8601_extended(self.clock.stamp().parse().unwrap_or(0))
    }
}

/// yog's injection, as installed at `Fx::tool_injection`.
pub struct Injection {
    state_root: PathBuf,
    budget: ask::Budget,
    clock: Arc<dyn Clock>,
    /// The `(workspace, agent)` this driver process was launched to drive, when
    /// its verb names one. [`ToolInjection::tools`] has no call to read them
    /// off — the seam is per-process, not per-agent (lernie's §7) — so the
    /// binding reads them out of its own argv and hands them over. A verb that
    /// names no agent (`prompt`, which mints one, and every operator verb)
    /// declares the `clients` tool and nothing else, which is exactly what a
    /// conversation with no loads reads as.
    driving: Option<(String, String)>,
}

impl Injection {
    /// Install an injection for a driver process.
    pub fn new(
        state_root: PathBuf,
        budget: ask::Budget,
        clock: Arc<dyn Clock>,
        driving: Option<(String, String)>,
    ) -> Self {
        Self {
            state_root,
            budget,
            clock,
            driving,
        }
    }

    /// The site one invocation is answered at.
    fn site(&self, workspace: &Path, agent: &str) -> Site {
        Site {
            state_root: self.state_root.clone(),
            workspace: crate::naming::leaf(workspace),
            agent: agent.to_owned(),
            budget: self.budget,
            clock: Arc::clone(&self.clock),
        }
    }
}

/// The `clients` tool's own definition — the one entry in the stable prefix.
fn declaration() -> InjectedTool {
    InjectedTool {
        name: clients::NAME.to_owned(),
        input_schema: clients::schema(),
        description: Some(clients::DESCRIPTION.to_owned()),
    }
}

/// A [`Result`] in the stdio vocabulary lernie's executor already speaks
/// (`docs/DESIGN_TOOL_INJECTION.md` §3.1): a product on stdout at exit 0, or
/// the reason on stderr at exit 1. The model cannot tell this from a local
/// tool, and neither can the transcript.
fn capture(name: &str, answered: Result<String, String>) -> RoutedCapture {
    match answered {
        Ok(product) => RoutedCapture {
            stdout: product.into_bytes(),
            stderr: Vec::new(),
            exit_code: 0,
        },
        Err(reason) => RoutedCapture {
            stdout: Vec::new(),
            stderr: format!("{name}: {reason}\n").into_bytes(),
            exit_code: 1,
        },
    }
}

/// What a loaded remote name earns today (REMOTE §9 step 7, bl-024b): the
/// refusal a vanished endpoint would earn, with the honest reason. The
/// declaration, the adjudication and the load are all real; the leg is not.
fn unrouted(entry: &loaded::Entry) -> Result<String, String> {
    Err(format!(
        "this engine cannot yet carry an invocation to client {:?}. \
         The advertisement, the roster and the load are in place; the leg that \
         takes an invocation down a tool host's live connection and brings the \
         capture back is not built yet (yog task bl-024b). \
         Nothing on {:?} was contacted.",
        entry.client, entry.client
    ))
}

impl ToolInjection for Injection {
    /// The `clients` tool, always, plus this agent's loaded set — read off
    /// disk, so assembly never waits on an engine and never varies with one.
    fn tools(&self) -> Vec<InjectedTool> {
        let mut out = vec![declaration()];
        if let Some((workspace, agent)) = &self.driving {
            out.extend(
                loaded::read(&self.state_root, workspace, agent)
                    .into_iter()
                    .map(|entry| InjectedTool {
                        name: entry.presented(),
                        input_schema: entry.tool.input_schema,
                        description: Some(entry.tool.description),
                    }),
            );
        }
        out
    }

    /// Answer the `clients` tool and every loaded remote name; decline
    /// everything else, so a pool tool resolves exactly as it would with no
    /// injection installed.
    fn route(&self, call: RoutedCall<'_>) -> Option<RoutedCapture> {
        let site = self.site(call.workspace, call.agent);
        if call.name == clients::NAME {
            return Some(capture(
                call.name,
                clients::answer(&site, call.input, call.stop),
            ));
        }
        let entry = loaded::read(&site.state_root, &site.workspace, &site.agent)
            .into_iter()
            .find(|entry| entry.presented() == call.name)?;
        Some(capture(call.name, unrouted(&entry)))
    }
}

#[cfg(test)]
mod tests;
