//! The production [`FileIo`] impl — the thin `std::fs` shell behind every
//! editor's pure view-model. Covered the way `cli_outbound`/`lock_probe` are:
//! a real tempdir, no fakes.

use super::FileIo;
use std::path::{Path, PathBuf};

/// `std::fs`-backed filesystem seam. Missing paths fold to the empty case
/// (`read` ⇒ `None`, `list_dir` ⇒ empty) and a missing destination *directory*
/// is made by the write that needs it, matching the trait contract.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFileIo;

impl FileIo for RealFileIo {
    fn read(&self, path: &Path) -> std::io::Result<Option<Vec<u8>>> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        // The place is made by the write that fills it (bl-8c06) — the same
        // move brazen's own two wall leaves already make
        // (`bz_host::store`'s credential store and model cache both
        // `create_dir_all` at `put`). A `<wall>/brazen/` that no act has
        // written yet does not exist, so §9.1's stage — a temp in the
        // destination's own directory — was an `ENOENT` with nothing but
        // `os error 2` to say, and the one act that can put a provider row in
        // a newborn wall could never land.
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, bytes)
    }

    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::rename(from, to)
    }

    fn remove(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_file(path)
    }

    fn list_dir(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        match std::fs::read_dir(dir) {
            // An unreadable individual entry is dropped, not fatal — the same
            // forgiveness `read` gives a missing file.
            Ok(entries) => Ok(entries.flatten().map(|e| e.path()).collect()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_write_rename_remove_roundtrip() {
        let dir = tempdir().unwrap();
        let io = RealFileIo;
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        // read of a missing file → None (fold identity), which is also how
        // absence is asked now that the seam carries no `exists` (bl-dba3: its
        // one caller was the credential probe, and the probe was a second
        // derivation of a column brazen already answers).
        assert_eq!(io.read(&a).unwrap(), None);
        io.write(&a, b"hello").unwrap();
        assert_eq!(io.read(&a).unwrap(), Some(b"hello".to_vec()));
        io.rename(&a, &b).unwrap();
        assert_eq!(io.read(&a).unwrap(), None);
        assert_eq!(io.read(&b).unwrap(), Some(b"hello".to_vec()));
        io.remove(&b).unwrap();
        assert_eq!(io.read(&b).unwrap(), None);
    }

    /// The write makes the place it writes into (bl-8c06) — absence on the
    /// write side folds the way absence on the read side already did, which is
    /// what lets a §9.1 apply land in a wall no act has filled yet.
    #[test]
    fn write_founds_the_destinations_directory() {
        let dir = tempdir().unwrap();
        let io = RealFileIo;
        let deep = dir.path().join("walls/home/brazen/config.toml");
        io.write(&deep, b"[[provider]]").unwrap();
        assert_eq!(io.read(&deep).unwrap(), Some(b"[[provider]]".to_vec()));
        // And a directory that cannot be made is still a fault, not a fold: an
        // ancestor that is a regular file is `NotADirectory`, never `NotFound`.
        let blocked = dir.path().join("file");
        io.write(&blocked, b"x").unwrap();
        assert!(io.write(&blocked.join("under/it"), b"x").is_err());
    }

    #[test]
    fn read_surfaces_non_notfound_errors() {
        // Reading a directory as a file is an error that is NOT NotFound —
        // it must propagate, not fold to None.
        let dir = tempdir().unwrap();
        assert!(RealFileIo.read(dir.path()).is_err());
    }

    #[test]
    fn list_dir_lists_children_and_absent_is_empty() {
        let dir = tempdir().unwrap();
        let io = RealFileIo;
        // An absent directory folds to an empty listing, not an error.
        assert!(io.list_dir(&dir.path().join("nope")).unwrap().is_empty());
        io.write(&dir.path().join("a.yaml"), b"1").unwrap();
        io.write(&dir.path().join("b.txt"), b"2").unwrap();
        let mut got = io.list_dir(dir.path()).unwrap();
        got.sort();
        assert_eq!(
            got,
            vec![dir.path().join("a.yaml"), dir.path().join("b.txt")]
        );
    }

    #[test]
    fn list_dir_surfaces_non_notfound_errors() {
        // Listing a regular file (ENOTDIR) is an error, not NotFound.
        let dir = tempdir().unwrap();
        let f = dir.path().join("f");
        RealFileIo.write(&f, b"x").unwrap();
        assert!(RealFileIo.list_dir(&f).is_err());
    }
}
