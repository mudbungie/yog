//! **The acceptance world's `World`** (§11): the populated fixture the tests
//! render, and its derivation driven by hand.
//!
//! Split from [`super`] per §12's budget, on the seam the two halves already
//! had: the builder there mints the world's bytes once, this holds what a test
//! then does to it. **The wire standing behind it** — the questions and the
//! acts the frame's own ends carry, and the fixed point they settle to — is
//! [`super::wire`], split off at the same budget for the reason its doc gives.

use super::super::super::ShellState;
use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::git_tree::tests::fixture::Fixture;
use crate::wire::link::LinkEnd;
use crate::wire::post::Outbox;
use std::path::PathBuf;
use tempfile::TempDir;

/// A workspace populated across every inspector surface (transcript, steps +
/// tool i/o, inbox), symlinked under the lernie workspaces root so the model
/// enumerates it.
pub(in crate::shell::acceptance) struct World {
    pub(super) _root: TempDir,
    /// The workspace's own git fixture. Reachable across the whole acceptance
    /// tree because a beat sometimes needs a config the shipped world does not
    /// carry — the §9.4 role fault wants a lineage naming a provider row brazen
    /// does not have, and lernie's pinned template names a live one (bl-d9cb).
    pub(in crate::shell::acceptance) fx: Fixture,
    /// Every extra sphere [`World::add_workspace`] created, held only so their
    /// temp dirs outlive the test — the §3.1 wall boundary is unobservable with
    /// one workspace, so a wall drive needs a second.
    pub(super) spheres: Vec<Fixture>,
    pub(in crate::shell::acceptance) model: AppModel,
    /// The §7.2 derivation, driven by hand: in the app a `Worker` thread runs
    /// it, and the frame renders only what it publishes (bl-ee0a).
    pub(super) deriver: crate::app::Deriver,
    pub(in crate::shell::acceptance) state: ShellState,
    pub(in crate::shell::acceptance) ws: PathBuf,
    /// yog's own data root — where the §16 nested world and the §3.1 names root
    /// live. A raise mints under it, so a test that drives one needs to seed the
    /// world here and read the raised sphere back.
    pub(in crate::shell::acceptance) yog_data: PathBuf,
    /// Where a second sphere is symlinked from ([`World::add_workspace`]).
    pub(in crate::shell::acceptance) lernie_workspaces: PathBuf,
    /// The frame's end of the read path — the standing questions a painted
    /// surface declared, taken exactly as the asker takes them.
    pub(in crate::shell::acceptance) link: LinkEnd,
    /// The frame's end of the act path — what its gestures posted, taken
    /// exactly as the poster takes them.
    pub(in crate::shell::acceptance) outbox: Outbox,
    /// **A §8.2 entry's end**, once [`World::attach_entry`] has given this
    /// window one: a second channel claiming one leaf, standing in for a
    /// workspace held on another box. `None` is every other fixture, which is
    /// §8.2's zero-entry shape — byte for byte what a window did before entries
    /// existed.
    pub(super) entry: Option<(String, LinkEnd)>,
    /// **Every act this world has answered**, in the order it took them — what
    /// a beat asserts a gesture *posted*, which is the only half of a routed
    /// act a window decides. Which channel it then goes down is
    /// `wire::channels`' question and is proven there.
    pub(in crate::shell::acceptance) acted: Vec<crate::boundary::Action>,
    /// The follow lane's engine end (bl-73e7), taken exactly as
    /// [`Lane`](crate::wire::lane::Lane) takes it.
    pub(in crate::shell::acceptance) tail: crate::wire::lane::TailEnd,
    /// The held read standing behind it: the conversation it is on, and the
    /// incremental fold it has reached. Minted when the subject changes and
    /// dropped when the stream ends, which is what the lane's thread does with
    /// a connection.
    pub(in crate::shell::acceptance) followed: Option<(String, crate::boundary::follow::Follow)>,
    /// **The engine's own substrate binaries** — what a posted act actually
    /// spawns. They are the engine's and never a seat's (REMOTE §9.8: a seat
    /// carries the gesture and nothing else), so they live here rather than on
    /// the driver. Deliberately absent by default, on `Screen::new`'s doctrine:
    /// a drive that has not said which fake it wants forks nothing real.
    pub(super) lernie: Cli,
    pub(super) bl: Cli,
}

impl World {
    /// One derivation pass and the frame's take of it — what the smoke test
    /// does whenever it has just changed something on disk. It answers the
    /// acts in flight first: a gesture the frame posted has to have *happened*
    /// before a derivation over the world it changed means anything.
    pub(in crate::shell::acceptance) fn converge(&mut self) {
        self.acts();
        self.deriver.step();
        self.model.refresh();
    }

    /// Point the engine's spawns at this drive's fakes — what
    /// `Screen::with_lernie` is asking for, said once where the acts run.
    pub(in crate::shell::acceptance) fn substrate(&mut self, lernie: &Cli, bl: &Cli) {
        self.lernie = lernie.clone();
        self.bl = bl.clone();
    }

    /// Mint a **second sphere** under the same lernie root: another workspace
    /// with one conversation, symlinked where the model enumerates it. Its §3.1
    /// leaf names its own wall (§16.2 as amended), so this is what a wall drive
    /// switches to. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn add_workspace(
        &mut self,
        name: &str,
        agent: &str,
    ) -> PathBuf {
        let fx = Fixture::new();
        fx.build_agent(agent, name);
        let ws = self.lernie_workspaces.join(name);
        std::os::unix::fs::symlink(&fx.path, &ws).unwrap();
        self.spheres.push(fx);
        ws
    }

    /// Fork a nameless descent child off `parent_id` (§2.3) — the bl-63a1
    /// chained-id shape: no name blob, no goal on disk, no step record, so the
    /// §3.3 ladder bottoms out at its floor. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn add_child(&self, parent_id: &str, child_id: &str) {
        self.fx.build_child(parent_id, child_id);
    }

    /// A **second root** conversation (§2.3) wearing `name` as its §3.3 name
    /// fact — a second row in the list, so a test can compare two rows of the
    /// same list rather than two worlds. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn add_root(&self, conv_id: &str, name: &str) {
        self.fx.build_agent(conv_id, name);
        self.fx.name_agent(conv_id, name);
    }

    /// **Advance the workspace's config lineage past every conversation in
    /// it** (§9.4 drift): one ordinary config edit on `config/default`,
    /// carrying `providers_yaml`. Every agent forked the lineage's previous
    /// head, so the governing commit and the tip part the moment this lands —
    /// which is the whole condition the drift clause and its exits are offered
    /// under. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn advance_config(&self, providers_yaml: &str) {
        self.fx.commit_other("providers.yaml", providers_yaml);
    }

    /// Mark `conv_id` **abandoned** — §6's will-not-retry assertion, the one
    /// gate that suppresses rule 2 (`attention::rest_evidence`). It is how a
    /// fixture gets a settled row bearing **no** attention beside one that
    /// does, without focusing anything and so without spending an ack.
    pub(in crate::shell::acceptance) fn quiet(&self, conv_id: &str) {
        self.fx
            .mark_ref(&format!("refs/lernie/abandoned/{conv_id}"));
    }
}
