//! litany global config editor — `models.yaml` and `workflows/*.yaml` (§9.2).
//!
//! > Text editor per file; Apply = hash-guard + temp-in-dir + rename (litany
//! > declares these hand-edited; yog is the hand, minus torn writes). yog still
//! > adds no YAML dep […] New workflow = same path, new name; templates
//! > copyable. (DESIGN §9.2)
//!
//! So this surface is the shared [`pipeline`](super::pipeline) verbatim — the
//! same stage → hash-guard → atomic rename brazen uses — and **nothing here
//! judges the YAML**: the operator's risk is `vi`'s, which is the section's
//! original posture and its posture again.
//!
//! **It held one validator between bl-53be and bl-3ffa, and the field it read is
//! why it went.** The gate refused an entry whose `models.<id>.provider` named no
//! brazen row — checkable without a YAML dep, and right while a role's model
//! resolved *through* that declaration. litany retired the `models:` table
//! (bl-35e2) and bl-d9cb re-pointed the picker at `providers.yaml`, leaving the
//! gate judging a field whose only remaining reader was the refusal itself: it
//! could refuse an Apply that was correcting the one line anything reads
//! (`context_window`), on the strength of a dead one. The row judgement now runs
//! only where the live pointer is
//! ([`is_unknown_row`](crate::model_pick::grammar::is_unknown_row), §9.4's pick
//! gate and role marks, §9.5's control over `roles.<r>.provider`).
//!
//! One thing is still deliberately *not* a special case: a new workflow is the
//! general path with an absent load-time snapshot — the guard that refuses a
//! changed file *is* the must-not-exist guard when the file was never there.

use super::{Draft, FileIo};
use crate::xdg::Env;
use std::path::{Path, PathBuf};

/// The litany-global editable surface, rooted at one config root (§9.2). The
/// root is the Y2 fold ([`litany_config_root`](Env::litany_config_root),
/// honoring `LITANY_HOME`); a missing root or empty `workflows/` simply yields
/// no workflows, never an error — absence is a value.
#[derive(Debug, Clone)]
pub struct LitanyGlobal {
    root: PathBuf,
}

impl LitanyGlobal {
    /// Fold the config root from an [`Env`] snapshot (§15 Y2).
    pub fn resolve(env: &Env) -> Self {
        Self {
            root: env.litany_config_root(),
        }
    }

    /// The single `models.yaml` path. Always offered: a missing file is a
    /// new-file edit (its editor reports [`is_new`](Editor::is_new)), not an
    /// error.
    pub fn models(&self) -> PathBuf {
        self.root.join("models.yaml")
    }

    /// The `workflows/` directory that holds the `*.yaml` workflow files.
    pub fn workflows_dir(&self) -> PathBuf {
        self.root.join("workflows")
    }

    /// Every existing `workflows/*.yaml`, sorted by path. Empty when the
    /// directory is absent or holds no `.yaml` file — the missing-root and
    /// empty-dir cases, folded to the same value, never an error.
    pub fn workflows(&self, io: &dyn FileIo) -> std::io::Result<Vec<PathBuf>> {
        let mut files: Vec<PathBuf> = io
            .list_dir(&self.workflows_dir())?
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("yaml"))
            .collect();
        files.sort();
        Ok(files)
    }

    /// The path a new workflow named `name` would occupy, once `name` is a
    /// safe single-file basename (§9.2 "new workflow = new file name"). The
    /// must-not-exist guard is the Apply pipeline's own snapshot guard applied
    /// to the [`Editor::seeded`] absent snapshot, not a check here.
    pub fn new_workflow(&self, name: &str) -> Result<PathBuf, WorkflowNameError> {
        validate_workflow_name(name)?;
        Ok(self.workflows_dir().join(format!("{name}.yaml")))
    }
}

/// Why a proposed workflow name is not a safe single-file basename (§9.2). The
/// sentence rides on the error, so the §11 pane and the §8.5 headless spelling
/// refuse in the same words rather than each phrasing the same three facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WorkflowNameError {
    /// Empty — a workflow must have a name.
    #[error("name required")]
    Empty,
    /// Contains a path separator — a name must name one file, not a path.
    #[error("name must be a single file, no path")]
    Separator,
    /// Leads with a dot — reserved for hidden and relative names.
    #[error("name must not start with a dot")]
    DotLeading,
}

fn validate_workflow_name(name: &str) -> Result<(), WorkflowNameError> {
    if name.is_empty() {
        return Err(WorkflowNameError::Empty);
    }
    if name.starts_with('.') {
        return Err(WorkflowNameError::DotLeading);
    }
    if name.contains('/') || name.contains('\\') {
        return Err(WorkflowNameError::Separator);
    }
    Ok(())
}

/// A raw-text editor over one litany-global file (§9.2): load into a RAM draft,
/// edit, Apply through the shared hash-guard + temp-in-dir + atomic rename
/// pipeline. Apply refuses on exactly one ground — a concurrent edit
/// ([`Conflict`](Saved::Conflict)) — the file's contents being litany's business
/// and the operator's, never this editor's (bl-3ffa).
#[derive(Debug, Clone)]
pub struct Editor {
    draft: Draft,
}

impl Editor {
    /// Load an existing file (or an editable-but-absent one like `models.yaml`)
    /// into a draft. See [`Draft::load`].
    pub fn load(path: PathBuf, io: &dyn FileIo) -> std::io::Result<Self> {
        Ok(Self {
            draft: Draft::load(path, io)?,
        })
    }

    /// Author a brand-new file at `path`, its draft seeded from `seed` bytes —
    /// the new-workflow and copy-from-existing affordances (§9.2 "templates
    /// copyable"). See [`Draft::seeded`]: the guard becomes must-not-exist.
    pub fn seeded(path: PathBuf, seed: &[u8]) -> Self {
        Self {
            draft: Draft::seeded(path, seed),
        }
    }

    /// The file this editor targets.
    pub(crate) fn path(&self) -> &Path {
        self.draft.path()
    }

    /// The RAM draft (§5.3 carve-out) — the text the §9.5 pane derives its
    /// typed rows from and writes back through, and the raw escape's read.
    pub(crate) fn draft(&self) -> &str {
        self.draft.text()
    }

    /// The draft as a mutable buffer — the binding an egui `TextEdit` edits.
    pub(crate) fn draft_mut(&mut self) -> &mut String {
        self.draft.text_mut()
    }

    /// Replace the draft text wholesale.
    pub fn set_draft(&mut self, text: String) {
        self.draft.set(text);
    }

    /// Whether the file was absent at load — a "new file" being authored,
    /// distinct from an existing one (§9.2).
    pub fn is_new(&self) -> bool {
        self.draft.is_new()
    }

    /// Re-read the file into the draft and re-snapshot — the Conflict recovery
    /// ("offer reload") and a plain refresh.
    pub fn reload(&mut self, io: &dyn FileIo) -> std::io::Result<()> {
        self.draft.reload(io)
    }

    /// Follow the file when nothing has been typed into the draft (§9
    /// read-on-demand freshness) — the same rule as §9.1's editor, from the
    /// same predicate. Reports whether it re-read.
    pub fn refresh(&mut self, io: &dyn FileIo) -> std::io::Result<bool> {
        self.draft.refresh(io)
    }

    /// Apply the draft through the shared pipeline. A concurrent change (or an
    /// already-present file when creating) ⇒ [`Saved::Conflict`]; any fs error ⇒
    /// [`Saved::Io`].
    pub fn apply(&mut self, io: &dyn FileIo) -> Saved {
        match self.apply_inner(io) {
            Ok(saved) => saved,
            Err(e) => Saved::Io {
                error: e.to_string(),
            },
        }
    }

    fn apply_inner(&mut self, io: &dyn FileIo) -> std::io::Result<Saved> {
        let staged = self.draft.stage(io)?;
        if self.draft.commit(staged, io)? {
            Ok(Saved::Ok)
        } else {
            Ok(Saved::Conflict)
        }
    }
}

/// The terminal state of an [`Editor::apply`] (§9.2): the concurrent-edit
/// conflict (which is also the new-file must-not-exist refusal), and a
/// filesystem error. There is no content rejection since bl-3ffa — see this
/// module's header for the field the retired one judged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Saved {
    /// Guard passed, renamed into place; the loaded snapshot is updated.
    Ok,
    /// The on-disk file changed since load (or already exists when creating):
    /// refuse rather than blind-LWW. Reload to re-diff ([`Editor::reload`]).
    Conflict,
    /// A filesystem error at any pipeline step.
    Io { error: String },
}

#[cfg(test)]
mod tests;
