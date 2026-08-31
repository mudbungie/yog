//! **yog's tool injection** (REMOTE §5, bl-c907): the object the `yog litany`
//! arm hands litany at `Fx::tool_injection`, so an agent can see this
//! workspace's client machines and drive the tools they advertise.
//!
//! litany's seam (its `docs/DESIGN_TOOL_INJECTION.md`) is **one object carrying
//! both halves** — the definitions prompt assembly and the grant gate read, and
//! the router the executor answers through — so a tool declared and not
//! permitted, or permitted and not declared, is unrepresentable. Since litany
//! 0.0.2 that router is **total**: while an injection is installed it answers
//! every invocation the agent makes, and no binary resolution stands behind it.
//! yog fills it with:
//!
//! - [`clients`] — ONE tool in the stable prefix, always. Its subject is the
//!   roster, and `load` is the act that makes a host's tools callable.
//! - [`loaded`] — the agent's durable loaded set, read at every assembly, each
//!   entry surfacing as an **individually named** tool. Never a multiplexer:
//!   litany's `docs/DESIGN_MCP_BRIDGE.md` §6 ruling binds a host too, so the
//!   grant gate, the tool control and any future policy keep seeing one name
//!   per capability.
//! - [`engine_act`] — the compactor's procedure pair, which is not a host
//!   injection at all: litany injects it from the calling role's own procedure,
//!   and since the seam inverted it reaches this router like every other name.
//!   Its subject is the conversation, which yog holds, so yog answers it
//!   itself, at the engine's own front door (REMOTE §5.4, bl-dfce).
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
//! **A name the router does not own is a refusal it renders.** Nothing resolves
//! a binary behind the injection any more, so there is nothing to hand a name
//! back to: an unowned name earns a non-zero capture saying so, which is the
//! shape an absent binary produced anyway. A conversation with nothing loaded
//! therefore refuses every ordinary call in band — REMOTE §12's ship-inert
//! posture working, not an error state.
//!
//! **Adjudication is untouched and still runs first.** `yog tool-control`
//! (DESIGN §8.6) is consulted before the executor routes anything, so a routed
//! invocation is judged exactly as a local one — and litany is honest that what
//! happens on the far machine is beyond the adjudicator's reach (REMOTE §5).
//!
//! **A loaded remote name runs where it lives** (REMOTE §9 step 7, bl-024b).
//! [`remote`] is the driver's end of the routing leg: two ordinary gestures
//! through the same inbox door — one that queues the call in the engine's
//! mailbox and one that polls for what came back — and the far machine's own
//! stdout, stderr and exit code passed through untouched, so the model cannot
//! tell a routed tool from a local one. Only a *transport* failure is a
//! sentence of yog's own, and it is in band and non-zero, which is the shape a
//! vanished endpoint already had to produce (litany's §3.3: *"A vanished
//! endpoint is an in-band error result, never a hang"*).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ::litany::cmd::{InjectedTool, RoutedCall, RoutedCapture, ToolInjection};

use crate::ui_state::{Clock, iso8601_extended};

/// The deposit round trip a driver asks its engine through.
pub mod ask;
/// The `clients` tool: the roster, and the act that loads from it.
pub mod clients;
/// The compactor's procedure pair, performed as engine acts.
pub mod engine_act;
/// The agent's durable loaded set.
pub mod loaded;
/// The driver's end of the routing leg (REMOTE §5, bl-024b).
pub mod remote;
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
    /// How long a *tool* on another machine may take (REMOTE §5, bl-024b) — a
    /// second bound because it measures a second thing: an engine that has not
    /// answered is down, a tool that has not answered is working.
    pub patience: ask::Budget,
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
    /// The path litany's own third hop addresses as `<driver_target> tool
    /// <name>` — the world's `litany` shim, which re-executes yog under that
    /// namespace. [`engine_act`] performs the compactor pair through it, so the
    /// acts stay the engine's own and yog restates none of their semantics.
    driver_target: PathBuf,
    budget: ask::Budget,
    patience: ask::Budget,
    clock: Arc<dyn Clock>,
}

impl Injection {
    /// Install an injection for a driver process. Two bounds, because they
    /// measure two things: `budget` is how long the engine may take to answer
    /// one deposit, `patience` how long a tool on another machine may take to
    /// run ([`remote::patience`]).
    pub fn new(
        state_root: PathBuf,
        driver_target: PathBuf,
        budget: ask::Budget,
        patience: ask::Budget,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            state_root,
            driver_target,
            budget,
            patience,
            clock,
        }
    }

    /// The site one invocation is answered at.
    fn site(&self, workspace: &Path, agent: &str) -> Site {
        Site {
            state_root: self.state_root.clone(),
            workspace: crate::naming::leaf(workspace),
            agent: agent.to_owned(),
            budget: self.budget,
            patience: self.patience,
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

/// **What a name nobody offers earns.** The router is total, so this is a
/// refusal yog renders rather than a hand-back — and it is the *whole* of the
/// ship-inert posture: a server with no machine enrolled, or an agent that has
/// loaded nothing, refuses every ordinary call in band and the model steps on.
/// It names the way out rather than the rule, because its reader is a model.
const UNLOADED: &str = "no tool of that name is loaded in this conversation; \
     use the clients tool to see this workspace's machines and load what one advertises";

/// A [`Result`] in the stdio vocabulary litany's executor already speaks
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

/// **A loaded remote name, run where it lives** (REMOTE §9 step 7, bl-024b):
/// the routing leg's capture, passed through exactly as it came back — the
/// far machine's own stdout, stderr and exit code, so the model cannot tell a
/// routed tool from a local one and neither can the transcript. A *transport*
/// failure is the in-band refusal [`capture`] spells, which is the shape a
/// vanished endpoint already had to produce (litany's §3.3).
fn routed(site: &Site, entry: &loaded::Entry, call: &RoutedCall<'_>) -> RoutedCapture {
    match remote::invoke(site, entry, call.input, call.stop) {
        Ok(got) => RoutedCapture {
            stdout: got.stdout.into_bytes(),
            stderr: got.stderr.into_bytes(),
            exit_code: got.exit_code,
        },
        Err(reason) => capture(call.name, Err(reason)),
    }
}

impl ToolInjection for Injection {
    /// The `clients` tool, always, plus this agent's loaded set — read off
    /// disk, so assembly never waits on an engine and never varies with one.
    ///
    /// **The driven agent is the seam's own fact since litany bl-ddaa**
    /// (yog bl-fd24): assembly asks *for* an agent, and the answer is that
    /// agent's document — so a `prompt` driver, whose verb mints its agent
    /// and whose argv therefore names none, declares its loads exactly as a
    /// resumed driver does. Before the amendment this read a
    /// binding-supplied `(workspace, agent)` that was `None` for every
    /// minting verb, and the conversation's whole first driver could load
    /// but never call.
    fn tools(&self, workspace: &Path, agent: &str) -> Vec<InjectedTool> {
        let mut out = vec![declaration()];
        out.extend(
            loaded::read(&self.state_root, &crate::naming::leaf(workspace), agent)
                .into_iter()
                .map(|entry| InjectedTool {
                    name: entry.presented(),
                    input_schema: entry.tool.input_schema,
                    description: Some(entry.tool.description),
                }),
        );
        out
    }

    /// Answer every name, because the router is total (litany's inverted seam):
    /// the `clients` tool, the compactor's two engine acts, every loaded remote
    /// name — and a refusal, rendered here, for anything else.
    fn route(&self, call: RoutedCall<'_>) -> RoutedCapture {
        let site = self.site(call.workspace, call.agent);
        if call.name == clients::NAME {
            return capture(call.name, clients::answer(&site, call.input, call.stop));
        }
        if engine_act::is(call.name) {
            return engine_act::perform(&self.driver_target, self.patience.span(), &call);
        }
        match loaded::read(&site.state_root, &site.workspace, &site.agent)
            .into_iter()
            .find(|entry| entry.presented() == call.name)
        {
            Some(entry) => routed(&site, &entry, &call),
            None => capture(call.name, Err(UNLOADED.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests;
