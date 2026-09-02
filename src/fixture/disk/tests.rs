//! Every primitive answers a failure as a sentence naming its own path. None
//! of them may panic: a fixture that aborted mid-lay would leave a half-world
//! a harness then renders as if it were a state.

use super::*;
use tempfile::TempDir;

#[test]
fn every_primitive_refuses_in_words() {
    let tmp = TempDir::new().expect("tmp");
    let file = tmp.path().join("f");
    std::fs::write(&file, "x").expect("write");
    let under = file.join("under");
    assert!(mkdir(&under).expect_err("mkdir").contains("create"));
    assert!(write(&under, "x").expect_err("write").contains("create"));
    assert!(
        stamp(&tmp.path().join("absent"), 1)
            .expect_err("stamp")
            .contains("stamp")
    );
    assert!(
        git(tmp.path(), &["no-such-subcommand"], None)
            .expect_err("git")
            .contains("git")
    );
    // A path with no parent at all takes `write`'s other branch.
    assert!(write(Path::new("/"), "x").is_err());
}

/// The happy half, so both arms of each primitive are spent — and `display`,
/// which is the one that never fails.
#[test]
fn the_primitives_write_what_they_are_asked_to() {
    let tmp = TempDir::new().expect("tmp");
    let deep = tmp.path().join("a/b/c");
    mkdir(&deep).expect("mkdir");
    assert!(deep.is_dir());
    let file = deep.join("d/f");
    write(&file, "body").expect("write");
    assert_eq!(std::fs::read_to_string(&file).expect("read"), "body");
    stamp(&file, 1_000_000_000).expect("stamp");
    let seen = std::fs::metadata(&file)
        .and_then(|m| m.modified())
        .expect("mtime");
    assert_eq!(
        seen,
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000)
    );
    assert_eq!(display(&file), file.display().to_string());
    git(tmp.path(), &["init", "-q", "--bare", "r.git"], Some(1)).expect("git");
    assert!(tmp.path().join("r.git/HEAD").is_file());
}
