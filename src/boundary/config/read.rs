//! The §9 destination's bytes, read (§8.5, bl-0164): split from [`super`] per
//! §12's line budget, mirroring [`write`](super::write). Since bl-dff8 it also
//! holds the two reads that had no boundary spelling at all — the §9.3 lineage
//! browse and the §9.4 model roster — because both are *populating reads asked
//! of the world at the moment they are asked*, which is what this module is.

use super::ConfigFile;
use crate::config_edit::branch::{Lineage, config_file, lineages};
use crate::config_edit::brazen::{BzOutcome, BzRunner, RealBzRunner};
use crate::config_edit::{RealFileIo, load_snapshot};
use crate::model_pick::query::{EMPTY_ROSTER, model_ids};
use std::path::Path;

/// **The §9 config family as ONE populating read** (bl-719a): the five reads
/// whose subject is a workspace's configuration, carried by the family's own
/// type instead of by five rows of the §8.5 query roster.
///
/// The fold `Action` has taken five times (§12), on the seam every layer
/// beneath already draws: [`super`] answers all five, `answer`'s table routes
/// them in one block, `codec::config` spells them, `line::config` reads them.
/// `query.rs` rested at 298 against a 300 wall — the inversion where a file on
/// the wall fires on whoever touches it next — and this is the hatch §12's own
/// row names.
///
/// **The fold is in the carrier, never in the surface.** Each member keeps its
/// slash verb, its envelope `op` and its help page, so no protocol version
/// moves and the corpus regenerates byte-identical; that identity is the check
/// that the move was behaviour-preserving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Read {
    /// One §9 destination's current bytes as text (bl-0164).
    File { file: ConfigFile },
    /// What lineages this workspace has, and what each holds (§9.3, bl-dff8).
    Lineages { workspace: String },
    /// The model ids one provider offers (§9.4, §5.1 #26, bl-dff8), asked in
    /// the named workspace's wall.
    Models { workspace: String, provider: String },
    /// Which branch this agent tracks on (§16.3, bl-0164).
    Marks { workspace: String },
    /// This workspace's effective provider table with each row's credential
    /// fact and tuning capability (§8.3, bl-0164/bl-23bd).
    Providers { workspace: String },
}

impl Read {
    /// The workspace slot REMOTE §8.2's name→path rewrite borrows.
    ///
    /// **`Option`, and that is the honest signature rather than a special case
    /// at the table.** Four members name a workspace outright; [`File`](Self::File)
    /// names a *destination* whose workspace is nested and itself optional
    /// (a §9 file may be the engine's own), which
    /// [`ConfigFile::workspace_slot`] already answers that way. Widening the
    /// carrier to match it keeps one rule where there would otherwise be a
    /// table arm that knows about one member.
    pub(crate) fn workspace_slot(&mut self) -> Option<&mut String> {
        match self {
            Self::File { file } => file.workspace_slot(),
            Self::Lineages { workspace }
            | Self::Models { workspace, .. }
            | Self::Marks { workspace }
            | Self::Providers { workspace } => Some(workspace),
        }
    }
}

/// One file's current bytes as text, or empty for a file that is not there
/// yet — the same [`load_snapshot`] every §9 editor loads through, minus the
/// hash it also returns: a read answers once and holds no draft to guard
/// later.
pub(super) fn text_at(path: &Path) -> Result<String, String> {
    load_snapshot(&RealFileIo, path)
        .map(|(text, _)| text)
        .map_err(|e| e.to_string())
}

/// One file's bytes **out of a config lineage** (§9.3): `git show
/// config/<lineage>:<path>`, the pane's own Load. Lossy-decoded, exactly as the
/// pane decodes it — a config commit holds text, and yog adds no encoding
/// guess. Unlike a file destination there is no empty answer here: git reports
/// a missing ref and a missing blob the same way, so an absent path refuses in
/// git's own words rather than reading back as a new empty file that an Apply
/// would then commit over the real one. [`Lineages`](super::super::Query::Lineages)
/// is the listing that says which paths are there to ask for.
pub(super) fn branch_text(workspace: &Path, lineage: &str, path: &str) -> Result<String, String> {
    config_file(workspace, &format!("config/{lineage}"), path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|e| e.to_string())
}

/// The §9.3 browse (bl-dff8): every lineage with the files its tip holds. A
/// workspace whose repo cannot be read refuses with git's own message — the
/// `work-diff` rule, and for its reason: an unreadable repository shown as an
/// empty list is a lie about the workspace.
pub(super) fn browse(workspace: &Path) -> Result<Vec<Lineage>, String> {
    lineages(workspace).map_err(|e| e.to_string())
}

/// The §9.4 roster (bl-dff8): the model ids `provider` offers, in the order it
/// lists them, read **in this process** through the linked brazen (§16.7 W10's
/// read half). Every seat that reaches the boundary's answer chokepoint is
/// already off-frame — the consumer's own thread, or a `yog gesture` process
/// with nothing else to do — so the picker's streamed spawn buys nothing here
/// and the fork buys a version skew.
pub(super) fn models(wall: &crate::xdg::Env, provider: &str) -> Result<Vec<String>, String> {
    roster(&RealBzRunner::resolve(wall).list_models(provider))
}

/// Fold one `--list-models` run into the roster or the reason there is none —
/// [`RosterView`](crate::model_pick::query::RosterView)'s settle, as a refusal:
/// a non-zero exit answers brazen's own stderr, and an exit-0 run that listed
/// nothing answers [`EMPTY_ROSTER`], never an empty list a seat would render as
/// "your provider has no models".
pub(super) fn roster(out: &BzOutcome) -> Result<Vec<String>, String> {
    if !out.success {
        let stderr = out.stderr.trim();
        if stderr.is_empty() {
            return Err("bz --list-models failed".to_owned());
        }
        return Err(stderr.to_owned());
    }
    let models = model_ids(&out.stdout);
    if models.is_empty() {
        return Err(EMPTY_ROSTER.to_owned());
    }
    Ok(models)
}
