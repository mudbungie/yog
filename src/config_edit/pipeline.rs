//! The write pipeline shared by every §9 config editor — the single source of
//! truth for how an in-RAM draft reaches disk without a torn write or a silent
//! last-writer-wins over a concurrent edit.
//!
//! Every editor (brazen §9.1, lernie-global §9.2, config branches §9.3) loads a
//! file into a RAM draft, edits it, then Applies through one discipline: stage
//! the draft to a temp *in the destination's own directory* (so committing is
//! an atomic same-dir rename), optionally let the caller inspect the temp (the
//! brazen `bz` validator gate), guard the on-disk snapshot against a concurrent
//! change since load, then rename. What differs per surface — the validator,
//! the peripheral panes — lives in the editor; the mechanism lives here.
//!
//! The filesystem is the injected [`FileIo`] seam, so Linux tarpaulin drives
//! every arm with an in-memory fake and no real disk.

use crate::ui_state::content_hash;
use std::path::{Path, PathBuf};

/// The filesystem seam every editor and the pipeline read and write through.
/// `read` and `list_dir` map a missing path to the empty case (`None` / no
/// entries), not an error — absence is a value, not a fault. [`RealFileIo`] is
/// `std::fs`; tests inject an in-memory fake.
///
/// [`RealFileIo`]: super::RealFileIo
pub trait FileIo {
    /// File bytes, or `None` when the file does not exist.
    fn read(&self, path: &Path) -> std::io::Result<Option<Vec<u8>>>;
    fn write(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn remove(&self, path: &Path) -> std::io::Result<()>;
    fn exists(&self, path: &Path) -> bool;
    /// The child paths of `dir`, or an empty vec when `dir` is absent. Order is
    /// unspecified; a caller that needs determinism sorts.
    fn list_dir(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>>;
}

/// Read a file into an editor's initial state: the draft text (a missing file
/// is empty text) and the load-time snapshot hash — `None` when the file is
/// absent, so an editor tells "new file" from "existing" and the Apply guard
/// can refuse a file that appeared underneath it. Shared by every editor's
/// `load` and `reload`.
pub(crate) fn load_snapshot(
    io: &dyn FileIo,
    path: &Path,
) -> std::io::Result<(String, Option<u64>)> {
    Ok(match io.read(path)? {
        Some(bytes) => (
            String::from_utf8_lossy(&bytes).into_owned(),
            Some(content_hash(&bytes)),
        ),
        None => (String::new(), None),
    })
}

/// Whether `draft` still holds exactly what the load-time snapshot `loaded`
/// captured — an untouched buffer. This is the whole test behind §9's
/// read-on-demand freshness (§7.1's bl-9130 ruling: config carries no watch
/// root): a pristine editor may follow disk, because re-reading it discards
/// nothing; an edited one may not, because adopting under the operator is the
/// blind LWW §9 rejects.
///
/// An absent snapshot (`None`) loaded as empty text, so an empty draft is
/// pristine there — and a [`seeded`](super::lernie_global::Editor::seeded)
/// new-file draft is not, which is right: its text was authored, never read.
pub(crate) fn is_pristine(draft: &str, loaded: Option<u64>) -> bool {
    match loaded {
        Some(hash) => content_hash(draft.as_bytes()) == hash,
        None => draft.is_empty(),
    }
}

/// The staging temp for a stage: `.<name>.yog-tmp-<pid>` in the destination's
/// own directory (§5.2), so committing is an atomic same-dir rename. A
/// degenerate destination (no file name / no parent) falls back cleanly.
fn temp_path(dest: &Path) -> PathBuf {
    let name = dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config");
    let dir = dest.parent().unwrap_or_else(|| Path::new("."));
    dir.join(format!(".{name}.yog-tmp-{}", std::process::id()))
}

/// The terminal state of a [`Staged::commit`].
pub(crate) enum Commit {
    /// Renamed into place; carries the committed content's hash for the editor
    /// to adopt as its new load-time snapshot.
    Ok(u64),
    /// The on-disk snapshot moved since load — refused (temp removed). The
    /// editor keeps the draft and offers reload.
    Conflict,
}

/// A draft staged to a temp beside its destination, awaiting commit — plain
/// owned data (the [`FileIo`] seam is threaded per operation, as every editor
/// already threads it). The caller may inspect [`temp`](Self::temp) (brazen
/// runs `bz` against it) then [`commit`](Self::commit) it into place, or
/// [`discard`](Self::discard) it (brazen's validator-reject path). Either
/// consumes the handle, so a staged temp is never leaked.
pub(crate) struct Staged {
    dest: PathBuf,
    temp: PathBuf,
    draft_hash: u64,
}

/// Stage `draft` to a temp in `dest`'s directory. The write is the only
/// fallible step here; guard and rename come at [`Staged::commit`].
pub(crate) fn stage(io: &dyn FileIo, dest: &Path, draft: &[u8]) -> std::io::Result<Staged> {
    let temp = temp_path(dest);
    io.write(&temp, draft)?;
    Ok(Staged {
        dest: dest.to_path_buf(),
        temp,
        draft_hash: content_hash(draft),
    })
}

impl Staged {
    /// The staged temp path, for a caller that validates the bytes on disk.
    pub(crate) fn temp(&self) -> &Path {
        &self.temp
    }

    /// Drop the staged temp without committing (the validator-reject path).
    pub(crate) fn discard(self, io: &dyn FileIo) -> std::io::Result<()> {
        io.remove(&self.temp)
    }

    /// Guard `loaded` against the current on-disk snapshot, then atomically
    /// rename the temp over the destination. A snapshot mismatch removes the
    /// temp and yields [`Commit::Conflict`] (this is also the must-not-exist
    /// guard for a new file: `loaded` is `None`, so any file now present is a
    /// mismatch); success yields the committed content's hash.
    pub(crate) fn commit(self, io: &dyn FileIo, loaded: Option<u64>) -> std::io::Result<Commit> {
        let on_disk = io.read(&self.dest)?.map(|b| content_hash(&b));
        if on_disk != loaded {
            io.remove(&self.temp)?;
            return Ok(Commit::Conflict);
        }
        io.rename(&self.temp, &self.dest)?;
        Ok(Commit::Ok(self.draft_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::FakeFs;

    fn dest() -> PathBuf {
        PathBuf::from("/cfg/models.yaml")
    }

    #[test]
    fn temp_path_sits_beside_dest_and_falls_back() {
        let pid = std::process::id();
        assert_eq!(
            temp_path(&dest()),
            PathBuf::from(format!("/cfg/.models.yaml.yog-tmp-{pid}"))
        );
        // A degenerate path (no file name, no parent) falls back cleanly.
        assert_eq!(
            temp_path(Path::new("/")),
            PathBuf::from(format!("./.config.yog-tmp-{pid}"))
        );
    }

    #[test]
    fn load_snapshot_distinguishes_absent_from_present() {
        let fs = FakeFs::seed(&dest(), b"hi");
        assert_eq!(load_snapshot(&fs, &dest()).unwrap().0, "hi");
        assert!(load_snapshot(&fs, &dest()).unwrap().1.is_some());
        let (text, hash) = load_snapshot(&fs, Path::new("/cfg/gone.yaml")).unwrap();
        assert_eq!(text, "");
        assert_eq!(hash, None);
    }

    #[test]
    fn pristine_is_the_untouched_buffer_absent_or_present() {
        // Present file: pristine iff the draft still hashes to the snapshot.
        let loaded = Some(content_hash(b"hi"));
        assert!(is_pristine("hi", loaded));
        assert!(!is_pristine("hi there", loaded));
        assert!(!is_pristine("", loaded));
        // Absent file: it loaded as empty text, so only empty is pristine —
        // a seeded new-file draft was authored, not read.
        assert!(is_pristine("", None));
        assert!(!is_pristine("seed", None));
    }

    #[test]
    fn stage_then_commit_renames_and_reports_hash() {
        let fs = FakeFs::seed(&dest(), b"A");
        let loaded = load_snapshot(&fs, &dest()).unwrap().1;
        let staged = stage(&fs, &dest(), b"B").unwrap();
        let temp = staged.temp().to_path_buf();
        assert!(matches!(
            staged.commit(&fs, loaded).unwrap(),
            Commit::Ok(h) if h == content_hash(b"B")
        ));
        assert_eq!(fs.get(&dest()), Some(b"B".to_vec()));
        assert_eq!(fs.get(&temp), None);
    }

    #[test]
    fn stage_then_discard_removes_the_temp() {
        let fs = FakeFs::seed(&dest(), b"A");
        let staged = stage(&fs, &dest(), b"B").unwrap();
        let temp = staged.temp().to_path_buf();
        staged.discard(&fs).unwrap();
        assert_eq!(fs.get(&temp), None);
        assert_eq!(fs.get(&dest()), Some(b"A".to_vec()));
    }

    #[test]
    fn commit_refuses_a_moved_snapshot() {
        let fs = FakeFs::seed(&dest(), b"A");
        let loaded = load_snapshot(&fs, &dest()).unwrap().1;
        let staged = stage(&fs, &dest(), b"B").unwrap();
        let temp = staged.temp().to_path_buf();
        // Another writer moves the file after load.
        fs.map().insert(dest(), b"C".to_vec());
        assert!(matches!(
            staged.commit(&fs, loaded).unwrap(),
            Commit::Conflict
        ));
        assert_eq!(fs.get(&dest()), Some(b"C".to_vec()));
        assert_eq!(fs.get(&temp), None);
    }
}
