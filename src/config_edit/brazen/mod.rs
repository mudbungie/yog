//! brazen `config.toml` editor — a pure view-model (DESIGN §9.1, §5.1
//! rows 19–23).
//!
//! The editor is **raw TOML text**, never form fields: brazen's schema is
//! versionless and full of open valves, so `bz` is the only lawful parser
//! (§9.1). yog therefore adds no TOML dependency — it edits bytes and lets
//! `bz` validate them. Since §16.7 W10 that `bz` is *linked*, so "the only
//! lawful parser" is enforced by the linker rather than by a version gate: the
//! validator and the loops' adapter are one exact-pinned implementation.
//!
//! The file edited is the **focused workspace's own** — since the
//! blast-radius ruling (§16.2) brazen's config lives inside that workspace's
//! wall ([`crate::world::wall`]), so switching workspace switches providers,
//! sign-ins and model cache together. An unfocused surface has no wall and so
//! no paths ([`BrazenPaths::of`] answers `None`); it renders a guard rather
//! than falling back to the machine's own brazen state.
//!
//! The view-model is pure over two injected effects, exactly the
//! [`LockProbe`](crate::git_tree) shape: a [`BzRunner`] (the sole `bz`
//! command surface — validate / effective-dump / provider table / list-models)
//! and the shared [`FileIo`](super::FileIo) seam. A fake pair drives every
//! state transition under Linux tarpaulin; the real [`RealBzRunner`] is a thin
//! shell over [`crate::bz_host`] plus one recorder-covered spawn.
//!
//! Apply is the shared [`pipeline`](super::pipeline) (stage → hash-guard →
//! atomic rename) with brazen's one addition, the `bz` validator gate:
//! ```text
//!   draft ──stage──▶ .config.toml.yog-tmp-<pid>  (temp in the dest dir)
//!         ──gate──▶  bz --config <temp> --dump-config
//!            non-zero exit ─▶ Rejected{stderr}   (draft kept, temp discarded)
//!         ──commit──▶ hash-guard + atomic rename
//!            snapshot moved ─▶ Conflict          (offer reload, temp removed)
//!            else ─▶ Ok                          (loaded snapshot updated)
//!       any fs error at any step ─▶ Io{error}
//! ```
//! `BRAZEN_CONFIG` is never leaked into the child env: the gate passes
//! `--config <temp>` explicitly, which overrides `bz`'s default search path
//! (`bz --help`: "--config <file> … else the default search path").

use super::{Draft, FileIo};
use crate::xdg::Env;
use std::path::{Path, PathBuf};

mod effects;
pub use effects::RealBzRunner;

mod providers;
pub use providers::{ProviderRow, ProviderRowView, provider_rows, row_names, row_views};

#[cfg(test)]
mod tests;

/// The captured result of one `bz` invocation. `success` is exit code 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BzOutcome {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// yog's entire `bz` command surface. Injected so the view-model is driven
/// by a fake in tests; [`RealBzRunner`] answers the three read verbs through
/// the **linked** brazen (§16.7 W10, [`crate::bz_host`]) and keeps a spawn only
/// for the one verb that goes to the network.
pub trait BzRunner {
    /// `bz --config <config> --dump-config` — the Apply validation gate.
    fn dump_config_at(&self, config: &Path) -> BzOutcome;
    /// `bz --dump-config` against the real file/env — the effective view.
    fn dump_config_effective(&self) -> BzOutcome;
    /// The effective provider table, in routing order (`bz --list-providers`,
    /// §5.1 #20/#21): the login rows and the credential-presence rows both read
    /// this one answer, and its `auth` column is the login-capability fact
    /// (§8.3) — see [`ProviderRow`].
    fn providers(&self) -> Vec<ProviderRow>;
    /// `bz --list-models --provider <provider> --json` — cache refresh.
    fn list_models(&self, provider: &str) -> BzOutcome;
}

/// The terminal state of an Apply (§9.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applied {
    /// Validated, hash-guard passed, renamed into place; loaded hash updated.
    Ok,
    /// `bz` rejected the draft; its stderr is surfaced verbatim, the draft
    /// is kept in RAM, the temp deleted. A malformed config never lands.
    Rejected { stderr: String },
    /// The on-disk file changed since load (hash mismatch): refuse rather
    /// than blind-LWW a concurrent edit. The temp is deleted; reload to
    /// re-diff ([`BrazenEditor::reload`]).
    Conflict,
    /// A filesystem error at any pipeline step.
    Io { error: String },
}

/// The static hint that provider rows are compiled into `bz` and never appear
/// in the file or the dump (§5.1 row 21). Rendered beside the effective pane.
/// Deliberately count-free: the number is brazen's, and pinning it here would
/// be a second representation of a fact the crate already owns — the login
/// surface's [`BzRunner::providers`] listing is where they are actually named.
pub const BUILT_IN_ROWS_HINT: &str = "built-in provider rows are compiled into bz and are not shown in this file \
     (the Login provider list shows them)";

/// brazen's three locations **inside one workspace's wall** (§5.1 rows
/// 19/22/23, §16.2 as amended): the layout is yog's own, so it is the same
/// three leaves on every §10 target and there is no per-OS branch left. This
/// struct is that layout's single home — [`crate::bz_host`] hands the very
/// same paths to the linked brazen's seams, so the file yog's editor writes
/// and the file `bz` reads cannot be two different files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrazenPaths {
    pub config: PathBuf,
    pub credentials_dir: PathBuf,
    pub models_cache_dir: PathBuf,
}

impl BrazenPaths {
    /// The three locations under an explicit wall root.
    pub fn in_wall(wall: &Path) -> Self {
        let brazen = wall.join("brazen");
        Self {
            config: brazen.join("config.toml"),
            credentials_dir: brazen.join("credentials"),
            models_cache_dir: brazen.join("models"),
        }
    }

    /// The three locations of the wall `env` names ([`Env::wall`]), or `None`
    /// outside any wall — a seat in no workspace has no providers to read, and
    /// that emptiness is rendered as a guard rather than filled from the
    /// machine's own brazen state (§16.2: nothing is ambient but the roster).
    pub fn of(env: &Env) -> Option<Self> {
        env.wall().map(|w| Self::in_wall(&w))
    }
}

/// Credential presence for a table of rows, against a credentials dir (§5.1
/// #22): does `<dir>/<provider>.json` exist? Booleans only — contents are never
/// read, never written. Free of the editor because the §8.3 Login surface asks
/// the same question without one (bl-402f): it holds no draft and no Apply
/// pipeline, only the folded dir. [`BrazenEditor::credential_presence`] is this
/// function at the editor's own path, so there is one presence read, not two.
pub fn credential_presence(
    dir: &Path,
    rows: &[ProviderRow],
    io: &dyn FileIo,
) -> Vec<(String, bool)> {
    rows.iter()
        .map(|row| {
            let path = dir.join(format!("{}.json", row.name));
            (row.name.clone(), io.exists(&path))
        })
        .collect()
}

/// The raw model-cache document `bz --list-models` wholesale-wrote for
/// `provider` under `dir` (§5.1 row 23), or `None` where it never ran there.
/// Read-only and forgiving — no parse, no schema coupling.
///
/// Free of the editor for the same reason [`credential_presence`] is: the §9.4
/// pick reads this very file to seed the context window it declares (bl-848f)
/// and holds no `config.toml` draft to ask through. One naming of
/// `<provider>.json`, so the file the picker reads and the file the §9.5 pane
/// shows can never be two different files.
pub fn model_cache_at(
    dir: &Path,
    provider: &str,
    io: &dyn FileIo,
) -> std::io::Result<Option<String>> {
    Ok(io
        .read(&dir.join(format!("{provider}.json")))?
        .map(|b| String::from_utf8_lossy(&b).into_owned()))
}

/// The brazen config editor view-model. Holds only the RAM carve-out (the
/// unsent draft, §5.3) and the loaded-content hash for the concurrent-edit
/// guard; every other datum is derived through an injected effect on demand.
#[derive(Debug, Clone)]
pub struct BrazenEditor {
    paths: BrazenPaths,
    draft: Draft,
}

impl BrazenEditor {
    /// Load the file at the folded path into the draft buffer. A missing
    /// file is empty text (fold identity, not an error, §9.1); the load-time
    /// snapshot is `None`, so the Apply guard still refuses if a config
    /// appears underneath the editor. See [`Draft::load`].
    pub fn load(paths: BrazenPaths, io: &dyn FileIo) -> std::io::Result<Self> {
        let draft = Draft::load(paths.config.clone(), io)?;
        Ok(Self { paths, draft })
    }

    /// The RAM draft (§5.3 carve-out). Test-only reader; the shell binds the
    /// mutable buffer through `draft_mut`.
    #[cfg(test)]
    pub(crate) fn draft(&self) -> &str {
        self.draft.text()
    }

    /// The draft as a mutable buffer — the binding an egui `TextEdit` edits.
    pub(crate) fn draft_mut(&mut self) -> &mut String {
        self.draft.text_mut()
    }

    /// Replace the draft text wholesale (e.g. a template paste).
    pub fn set_draft(&mut self, text: String) {
        self.draft.set(text);
    }

    /// Re-read the file into the draft and re-snapshot — the Conflict recovery
    /// ("offer reload") and a plain refresh.
    pub fn reload(&mut self, io: &dyn FileIo) -> std::io::Result<()> {
        self.draft.reload(io)
    }

    /// Follow the file when nothing has been typed into the draft (§9
    /// read-on-demand freshness). The config roots carry no watcher (§7.1,
    /// bl-9130), so the operator's own attention gesture — opening the Config
    /// pane — is the re-read trigger: without it a `config.toml` edited in `vi`
    /// stays invisible to this pane for the whole process lifetime. Reports
    /// whether it re-read.
    pub fn refresh(&mut self, io: &dyn FileIo) -> std::io::Result<bool> {
        self.draft.refresh(io)
    }

    /// Run the §9.1 Apply pipeline. Any filesystem error becomes
    /// [`Applied::Io`]; the logical outcomes are `Ok`, `Rejected`, `Conflict`.
    pub fn apply(&mut self, runner: &dyn BzRunner, io: &dyn FileIo) -> Applied {
        match self.apply_inner(runner, io) {
            Ok(applied) => applied,
            Err(e) => Applied::Io {
                error: e.to_string(),
            },
        }
    }

    fn apply_inner(&mut self, runner: &dyn BzRunner, io: &dyn FileIo) -> std::io::Result<Applied> {
        let staged = self.draft.stage(io)?;
        let gate = runner.dump_config_at(staged.temp());
        if !gate.success {
            staged.discard(io)?;
            return Ok(Applied::Rejected {
                stderr: gate.stderr,
            });
        }
        if self.draft.commit(staged, io)? {
            Ok(Applied::Ok)
        } else {
            Ok(Applied::Conflict)
        }
    }

    /// The read-only effective config: `bz --dump-config` against the real
    /// file/env, rendered verbatim (§5.1 row 20). Pair with
    /// [`BUILT_IN_ROWS_HINT`].
    pub fn effective(&self, runner: &dyn BzRunner) -> BzOutcome {
        runner.dump_config_effective()
    }

    /// Credential presence — booleans only (§5.1 row 22). For each row of the
    /// **effective** table (the linked brazen's answer, §16.7 W10 — not a scan
    /// of the draft, which may be unapplied or malformed), does
    /// `<creds-dir>/<provider>.json` exist? Contents are never read.
    ///
    /// The table is passed in rather than re-queried: the §9.5 pane asks brazen
    /// **once** per open and renders every column of that one answer, so the
    /// rows it names and the credentials it reports can never be two different
    /// tables.
    pub fn credential_presence(
        &self,
        rows: &[ProviderRow],
        io: &dyn FileIo,
    ) -> Vec<(String, bool)> {
        credential_presence(&self.paths.credentials_dir, rows, io)
    }

    /// The model cache for a provider (§5.1 row 23) — [`model_cache_at`] at
    /// this editor's own wall.
    pub fn model_cache(&self, provider: &str, io: &dyn FileIo) -> std::io::Result<Option<String>> {
        model_cache_at(&self.paths.models_cache_dir, provider, io)
    }

    /// Refresh a provider's model cache: `bz --list-models` writes the cache
    /// on disk itself; the caller re-reads via [`model_cache`](Self::model_cache).
    /// The outcome is returned so a failure surfaces verbatim.
    pub fn refresh_models(&self, provider: &str, runner: &dyn BzRunner) -> BzOutcome {
        runner.list_models(provider)
    }
}
