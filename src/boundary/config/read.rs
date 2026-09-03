//! The §9 destination's bytes, read (§8.5, bl-0164): split from [`super`] per
//! §12's line budget, mirroring [`write`](super::write). Since bl-dff8 it also
//! holds the two reads that had no boundary spelling at all — the §9.3 lineage
//! browse and the §9.4 model roster — because both are *populating reads asked
//! of the world at the moment they are asked*, which is what this module is.

use super::ConfigFile;
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::{ConfigView, Reply};
use crate::config_edit::branch::{Lineage, config_file, lineages};
use crate::config_edit::brazen::{BzOutcome, BzRunner, RealBzRunner, row_names};
use crate::config_edit::form::{self, Control, Row, schema_for};
use crate::config_edit::litany_global::LitanyGlobal;
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
    /// **What this workspace's roles are actually set to** (§9.4, §5.1 #27;
    /// bl-2410): every role its config lineage declares, with the provider row
    /// and model id bound to it and the two §9.4 tuning knobs it carries.
    ///
    /// A different subject from [`Providers`](Self::Providers), which is why it
    /// is a different member: that one is per *provider row* and says what a
    /// row is **capable** of, this is per *role* and says what has been
    /// **chosen**. REMOTE §9.14 carries the rest of the argument — why it is
    /// not a raw-text read, where it is read from, and why nothing set is an
    /// answer rather than a refusal.
    Roles { workspace: String },
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
            | Self::Providers { workspace }
            | Self::Roles { workspace } => Some(workspace),
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

/// Read one §9 destination (§8.5, bl-0164): [`apply`](super::apply)'s read-only
/// twin, and the file editors' Reload spelled headless. A file destination that
/// is not there yet answers empty text — the same "new file" reading every
/// editor's own load already gives — so only a real I/O failure refuses. A
/// **lineage** answers the pane's own Load (bl-dff8): `git show
/// config/<lineage>:<path>`, the very bytes an Apply on that destination would
/// be diffed against. It carries the write's `origin` and ignores it, because
/// where the next commit lands is not where the current bytes are;
/// [`Query::Lineages`](crate::boundary::Query::Lineages) is the browse that
/// says which paths a lineage holds.
///
/// **The answer is the text AND the file's typed settings** (§9.5, bl-dc3f):
/// the same bytes read twice, once verbatim and once through the schema, so one
/// answer serves the raw editor and the controls pane and the file stays the
/// single fact. It is a fold over the text just returned — never a second read
/// of disk, which would let the two halves of one answer disagree. A file yog
/// has no grammar for answers an **empty** list, not an absent field: §9.5's
/// three raw-text fallbacks are *"the general path with empty input, not a
/// branch"*, and a seat with no settings to show is already showing the raw
/// editor.
pub(crate) fn file(deps: &Deps, ws: &Path, dest: &ConfigFile) -> Result<Reply, String> {
    let text = match dest {
        ConfigFile::Brazen { .. } => text_at(&super::brazen_paths(deps, ws).config)?,
        ConfigFile::LitanyModels => text_at(&LitanyGlobal::resolve(&deps.world).models())?,
        ConfigFile::LitanyWorkflow { name } => text_at(
            &LitanyGlobal::resolve(&deps.world)
                .new_workflow(name)
                .map_err(|e| e.to_string())?,
        )?,
        ConfigFile::Cadence => text_at(&super::cadence_path(&deps.world))?,
        ConfigFile::Branch { lineage, path, .. } => branch_text(ws, lineage, path)?,
    };
    let settings = settings(deps, ws, dest, &text);
    Ok(Reply::Config(ConfigView { text, settings }))
}

/// The file's schema applied to the text it just answered (§9.5) — the settings
/// table, the bounds each control judges at input, and the provider judgement
/// `is_unknown_row` makes, all of which §9.5 rules must reach the surface and
/// none of which crossed before bl-dc3f.
///
/// **brazen is asked only where a control needs it.** The provider fault is the
/// one judgement here that is not a fact of the text, and only a schema
/// declaring a [`Control::Provider`] field can carry it — so a `cadence.yaml`
/// read spawns nothing, and the `Deps` contract ("asked, never stored", read
/// inside the wall the gesture names — bl-fcd5) is kept where it does. An
/// unanswerable brazen is an empty table, which faults nothing: no surface may
/// refuse on the strength of a question that went unanswered.
fn settings(deps: &Deps, ws: &Path, dest: &ConfigFile, text: &str) -> Vec<Row> {
    let Some(schema) = schema_for(&dest.file_name()) else {
        return Vec::new();
    };
    let providers = if schema.fields.iter().any(|f| f.control == Control::Provider) {
        row_names(&RealBzRunner::resolve(&super::wall_env(deps, ws)).providers())
    } else {
        Vec::new()
    };
    form::read(&schema, text, &providers)
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
