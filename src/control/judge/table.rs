//! The **shipped default table** (VISION §4.11 item 6): what the control says
//! about a class when the operator has said nothing. Its own file beside the
//! vocabulary that names the classes and the fold that reads the operator's
//! own answers — three things, three homes.

use super::Ruling;
use crate::control::classify::Effect;

/// The class → ruling table: **everything passes except loss and credentials**.
/// An unattended drone is there to work, and a shipped hold on open-world made
/// the operator answer for every `python` and every fetch — approving what they
/// were always going to approve. So the four classes that are the job pass, and
/// only irreversible loss and credential access decline in band: those two are
/// what a drone must not decide for itself, and neither is answerable by
/// reflex.
///
/// **Hold is no longer standing policy; it is imposed.** Two mechanisms carry
/// the weight the shipped hold used to, and both aim it at the conversation
/// that earned it rather than at all of them:
///
/// - a workspace that wants the parked default writes one line of
///   `capability.yaml` — `table:` / `  open-world: hold` (see
///   [`Policy`](super::policy::Policy)); severability still runs the right way,
///   with absence the (now permissive) default and the file the override;
/// - the alignment monitor's revoke rung raises a per-conversation floor, under
///   which every class above read holds ([`Answers::floored`]).
///
/// **One exception, and it is not about a reach** (bl-72bd): the seventh class
/// [`Opaque`](Effect::Opaque) holds, because it is what the classifier says
/// when it could not read the invocation at all. bl-1ef1's argument does not
/// reach it — that argument was about parking effects the operator was always
/// going to approve, and this class is the one where nobody knows what is
/// being approved. A workspace that wants the old, open answer writes
/// `table:` / `  opaque: pass`, the same one line, the same way round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table;

impl Table {
    /// This class's ruling.
    pub fn ruling(effect: Effect) -> Ruling {
        match effect {
            Effect::Read | Effect::TargetWrite | Effect::Process | Effect::OpenWorld => {
                Ruling::Pass
            }
            Effect::Destructive | Effect::Secret => Ruling::Refuse,
            // The one shipped hold, and it is not a policy about a reach — it
            // is what the control says when it could not read one (bl-72bd).
            // A refusal would be a claim about the invocation this control has
            // no basis for; a pass is the arm the routed leg fell off into for
            // a year. So it parks, and the operator answers once.
            Effect::Opaque => Ruling::Hold,
        }
    }
}

#[cfg(test)]
mod tests;
