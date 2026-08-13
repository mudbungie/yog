//! The RAM draft every §9 file editor holds: one file's text, the load-time
//! snapshot behind the concurrent-edit guard, and the gestures over them.
//!
//! [`pipeline`](super::pipeline) already owned how an edit *reaches disk*.
//! What each editor still restated for itself was the state that edit is made
//! of — the `{path, text, loaded}` trio and its load / reload / refresh /
//! set gestures. §9.1's [`BrazenEditor`](super::brazen::BrazenEditor) and
//! §9.2's [`Editor`](super::lernie_global::Editor) carried byte-identical
//! bodies for all of them and differed only in their Apply **gate** (`bz`
//! validation vs the provider-row check). Under the 100% floor each copy also
//! bought its own tests for the same three facts.
//!
//! So the gate is the whole difference between the two surfaces, and this is
//! everything else. An editor is now a [`Draft`] plus its gate.

use super::{Commit, FileIo, is_pristine, load_snapshot, stage};
use std::path::{Path, PathBuf};

/// One file's editable text and the snapshot it was read at.
#[derive(Debug, Clone)]
pub(crate) struct Draft {
    path: PathBuf,
    text: String,
    loaded: Option<u64>,
}

impl Draft {
    /// Read `path` into a draft. A missing file is empty text with an absent
    /// snapshot, so [`is_new`](Self::is_new) reports it and the Apply guard
    /// still refuses a file that appears underneath.
    pub(crate) fn load(path: PathBuf, io: &dyn FileIo) -> std::io::Result<Self> {
        let (text, loaded) = load_snapshot(io, &path)?;
        Ok(Self { path, text, loaded })
    }

    /// Author a brand-new file at `path`, seeded from `seed` bytes. A pure
    /// constructor: the snapshot is forced absent, so the Apply guard becomes
    /// a must-not-exist guard. Seed with `b""` for an empty new file.
    pub(crate) fn seeded(path: PathBuf, seed: &[u8]) -> Self {
        Self {
            path,
            text: String::from_utf8_lossy(seed).into_owned(),
            loaded: None,
        }
    }

    /// The file this draft targets.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// The draft text (§5.3 carve-out).
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// The draft as a mutable buffer — the binding an egui `TextEdit` edits.
    pub(crate) fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }

    /// Replace the draft text wholesale.
    pub(crate) fn set(&mut self, text: String) {
        self.text = text;
    }

    /// Whether the file was absent at load — a "new file" being authored.
    /// Flips to `false` once Apply creates it.
    pub(crate) fn is_new(&self) -> bool {
        self.loaded.is_none()
    }

    /// Re-read the file into the draft and re-snapshot — the Conflict recovery
    /// ("offer reload") and a plain refresh.
    pub(crate) fn reload(&mut self, io: &dyn FileIo) -> std::io::Result<()> {
        let (text, loaded) = load_snapshot(io, &self.path)?;
        self.text = text;
        self.loaded = loaded;
        Ok(())
    }

    /// Follow the file when nothing has been typed (§9 read-on-demand
    /// freshness — the config roots carry no watcher, §7.1 bl-9130). An edited
    /// draft is left exactly as typed and learns at Apply from the hash guard;
    /// a [`seeded`](Self::seeded) draft is never pristine, so this can never
    /// discard one. Reports whether it re-read.
    pub(crate) fn refresh(&mut self, io: &dyn FileIo) -> std::io::Result<bool> {
        if !is_pristine(&self.text, self.loaded) {
            return Ok(false);
        }
        self.reload(io)?;
        Ok(true)
    }

    /// Stage the draft to a temp beside its destination, for a gate to inspect
    /// and then [`commit`](Self::commit).
    pub(crate) fn stage(&self, io: &dyn FileIo) -> std::io::Result<super::pipeline::Staged> {
        stage(io, &self.path, self.text.as_bytes())
    }

    /// Hash-guard and rename a staged temp into place, re-snapshotting on
    /// success. `false` is the concurrent-edit conflict — each editor maps it
    /// to its own refusal variant.
    pub(crate) fn commit(
        &mut self,
        staged: super::pipeline::Staged,
        io: &dyn FileIo,
    ) -> std::io::Result<bool> {
        match staged.commit(io, self.loaded)? {
            Commit::Ok(hash) => {
                self.loaded = Some(hash);
                Ok(true)
            }
            Commit::Conflict => Ok(false),
        }
    }
}
