//! **What a gesture addresses, one noun down** (REMOTE §8 as amended by
//! bl-49bc, DESIGN §8.5): the *conversation* table over [`Action`] and
//! [`Query`], and the one resolution that turns the needle it answers into an
//! agent **id**. Its own file beside [`super`]'s two nouns, on the seam that
//! module's doc already draws — one table per noun, standing once ahead of each
//! chokepoint's match.
//!
//! **One addressing vocabulary: an id, or the unique stored name a living agent
//! wears.** A `/prompt` receipt answers `{"kind":"started","conversation":
//! "<minted-name>"}` — the minted §3.3 name is all the fire knows, since the
//! root has no id until the detached driver writes `agents/<id>` — so before
//! this resolution existed, that handle composed with `message` (litany's one
//! name-resolving verb) and with nothing else: `/agent` read `present:false`,
//! `/steps` and `/transcript` answered empty rows, `/stop` and `/retarget`
//! refused, and — the dangerous half — `/floor`, `/flag` and `/delete-agent`
//! *succeeded*, writing yog's own policy rows against a string no conversation
//! answers to. A receipt whose handle only one verb accepts is not a receipt.
//!
//! **Names and ids are disjoint spaces, so the resolution never guesses.**
//! Every agent id opens with litany's compact `YYYYMMDDTHHMMSSZ` stamp
//! (ARCH §2.3) and litany refuses a *name* that reads like one at creation, so
//! [`id_shaped`] is a total discriminator on the needle itself. That buys three
//! things at once. An id costs nothing — no enumeration, no existence check, so
//! every seat that already spells one (the window, the inspector, a peer
//! following a `/conversations` row) is untouched, and `delete-agent` keeps
//! admitting the id no ref answers to that litany's §9.2 debris cleanup needs. A
//! name is resolved or **refused**, never passed through as though it were an
//! id — which is the whole defect. And the vocabulary is litany's own, not a
//! second one yog invented (`workspace::agent_name::resolve`: "an exact id match
//! first, else the unique living agent wearing that name"), so a handle that
//! addresses here addresses there.
//!
//! **A legacy display-only name refuses, and that is correct** (bl-8068): the
//! §3.3 ladder's rung two is a `You are <x>.` goal-stamp parse with no stored
//! `name` blob behind it, so no ref answers to it. It renders as a title and it
//! has never been an address; the seats that paint one already hover that fact.

use crate::app::Snapshot;
use crate::boundary::{Action, Query};
use std::path::Path;

impl Action {
    /// The **conversation** this gesture names (§3.3), or `None` when it names
    /// none. `Fork` answers with its `parent` — the agent it dispatches *from*
    /// is the agent it addresses — and the monitor's family answers through its
    /// own verb, exactly as the workspace table does.
    pub(crate) fn agent(&self) -> Option<String> {
        match self {
            Action::Message { agent, .. }
            | Action::Stop { agent, .. }
            | Action::Interrupt { agent, .. }
            | Action::Nudge { agent, .. }
            | Action::Retarget { agent, .. }
            | Action::DeleteAgent { agent, .. }
            | Action::MarkSeen { agent, .. }
            | Action::AnswerHold { agent, .. }
            | Action::Floor { agent, .. } => Some(agent.clone()),
            Action::Fork { parent, .. } => Some(parent.clone()),
            Action::Monitor(verb) => verb.agent(),
            // Everything else addresses a workspace, a project, a client or the
            // world. A start names no conversation *because it makes one* — the
            // name it mints is the receipt, and the receipt is what the next
            // gesture addresses.
            Action::Prepare { .. }
            | Action::Prompt { .. }
            | Action::Scan { .. }
            | Action::Close { .. }
            | Action::Assign { .. }
            | Action::Release { .. }
            | Action::Create { .. }
            | Action::Update { .. }
            | Action::Fan(_)
            | Action::DeleteWorkspace { .. }
            | Action::Fleet(_)
            | Action::Ack
            | Action::ClearTrail
            | Action::ApplyConfig { .. }
            | Action::SetMarks { .. }
            | Action::Advertise { .. }
            | Action::Enroll(_)
            | Action::Route(_)
            | Action::PickModel { .. } => None,
        }
    }
}

impl Query {
    /// The **conversation** this read is aimed at (§3.3) — the §11 inspector
    /// family plus the seat's own [`Query::Agent`] — or `None` for the reads
    /// aimed at a workspace, at the world, or at the interface. The mirror of
    /// the action table above, and it stands ahead of
    /// [`answer`](crate::boundary::answer::answer) for the same reason.
    pub(crate) fn agent(&self) -> Option<String> {
        match self {
            Query::Transcript { agent, .. }
            | Query::Follow { agent, .. }
            | Query::Steps { agent, .. }
            | Query::Step { agent, .. }
            | Query::Files { agent, .. }
            | Query::Governing { agent, .. }
            | Query::Rail { agent, .. }
            | Query::Inbox { agent, .. }
            | Query::Agent { agent, .. } => Some(agent.clone()),
            Query::Conversations { .. }
            | Query::Science { .. }
            | Query::WorkDiff { .. }
            | Query::Lineages { .. }
            | Query::Models { .. }
            | Query::Marks { .. }
            | Query::Providers { .. }
            | Query::WorkspaceBalls { .. }
            | Query::Clients { .. }
            | Query::Workspaces
            | Query::Balls
            | Query::Board
            | Query::Attention
            | Query::Ops { .. }
            | Query::Search { .. }
            | Query::Help { .. }
            | Query::ReadConfig { .. }
            | Query::Invocations
            | Query::Capture { .. } => None,
        }
    }
}

/// Resolve the conversation a gesture names to the **agent id** every executor
/// and every derivation keys on, or refuse naming the token.
///
/// `needle` is the table above's answer, so `None` — a gesture that names no
/// conversation — resolves to nothing and no arm reads it: the general path with
/// no input, folded in here rather than spelled at both chokepoints.
///
/// Three rungs, each answering a question the one above it cannot.
///
/// 1. An **id-shaped** needle is an id (module doc): returned untouched, with no
///    enumeration and no existence claim — so every seat that already spells one
///    pays nothing, and `delete-agent` keeps admitting the id no ref answers to
///    that litany's §9.2 debris cleanup needs.
/// 2. Otherwise the **published derivation** ([`Snapshot`]) is asked, because it
///    is the very set every boundary *read* answers from: addressing and
///    answering must not be able to disagree about which conversations there
///    are. It holds foreign and hand-made ids — the ones litany's stamp grammar
///    does not recognize, which rung one cannot see — beside every stored name.
/// 3. Otherwise **disk**, for the conversation no derivation has swept yet
///    (bl-6c9e's barrier, one noun down): a `/prompt` fire returns the instant
///    its detached driver is launched, so the name its receipt hands back is
///    addressable before the next §7.2 pass has read the branch. Reached only
///    when rung two has no answer at all, so the steady state never pays for it.
///
/// An unknown needle refuses and so does an ambiguous one, in the resolver's own
/// words — never a pass-through, which is what let a display name reach `floor`,
/// `flag` and `delete-agent` as though it addressed a conversation.
pub(crate) fn resolve_agent(
    snap: &Snapshot,
    workspace: &Path,
    needle: Option<String>,
) -> Result<String, String> {
    let Some(needle) = needle else {
        return Ok(String::new());
    };
    if id_shaped(&needle) {
        return Ok(needle);
    }
    if let Some(id) = within(&derived(snap, workspace), &needle)? {
        return Ok(id);
    }
    if let Some(id) = within(&crate::git_tree::living_agents(workspace), &needle)? {
        return Ok(id);
    }
    Err(format!("unknown conversation {needle:?}"))
}

/// What `needle` addresses **within one enumerated conversation set**: the id it
/// already is, else the id of the unique agent wearing it as a stored name.
/// `Ok(None)` is *this set has no answer* — ask the next rung — and `Err` is one
/// name worn by two living agents, which refuses rather than guessing exactly as
/// two workspace roots sharing a leaf do.
///
/// One rule read over both sets, so the derivation and the disk behind it cannot
/// come to mean different things by the same word.
fn within(set: &[(String, Option<String>)], needle: &str) -> Result<Option<String>, String> {
    if set.iter().any(|(id, _)| id == needle) {
        return Ok(Some(needle.to_owned()));
    }
    let mut worn = set
        .iter()
        .filter(|(_, name)| name.as_deref() == Some(needle));
    let Some((id, _)) = worn.next() else {
        return Ok(None);
    };
    match worn.next() {
        None => Ok(Some(id.clone())),
        Some(_) => Err(format!("ambiguous conversation {needle:?}")),
    }
}

/// This workspace's conversations as the **published derivation** holds them
/// (§7.2): `(id, stored name)`, the shape [`crate::git_tree::living_agents`]
/// answers in. The stored [`Agent::name`](crate::git_tree::Agent) blob rather
/// than the §3.3 ladder's fold, because the ladder's second rung is a goal-stamp
/// *title* no ref answers to (bl-8068) — it renders, it has never addressed. A
/// workspace with no derived tree answers with none: the general path with empty
/// inputs.
fn derived(snap: &Snapshot, workspace: &Path) -> Vec<(String, Option<String>)> {
    snap.trees
        .get(workspace)
        .into_iter()
        .flat_map(|tree| {
            tree.agents
                .iter()
                .map(|a| (a.agent_id.clone(), a.name.clone()))
        })
        .collect()
}

/// Does `needle` read as an agent id — its first `-` segment being litany's
/// compact `YYYYMMDDTHHMMSSZ` stamp (ARCH §2.3)?
///
/// The grammar is [`crate::nav::convs::is_stamp`], the tree's one definition of
/// what an agent id is: the §3.3 ladder's floor reads it, the acceptance naming
/// scan reads it, and now the boundary's addressing does — so what counts as an
/// id cannot come to differ between the seat that paints one and the resolver
/// that accepts one.
fn id_shaped(needle: &str) -> bool {
    needle
        .split('-')
        .next()
        .is_some_and(crate::nav::convs::is_stamp)
}

#[cfg(test)]
mod tests;
