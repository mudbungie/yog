//! The stamp: written at a connection's rate, read as "never" when it is not
//! there.

use super::*;
use tempfile::tempdir;

fn client() -> Client {
    Client::parse("laptop").expect("identity")
}

/// It round-trips, and the last write wins — a client that speaks again is
/// later, not twice.
#[test]
fn the_stamp_round_trips_and_the_latest_write_stands() {
    let root = tempdir().expect("tmp");
    assert_eq!(read(root.path(), &client()), None, "never connected");
    mark(root.path(), &client(), 1_700_000_000);
    assert_eq!(read(root.path(), &client()), Some(1_700_000_000));
    mark(root.path(), &client(), 1_700_000_060);
    assert_eq!(read(root.path(), &client()), Some(1_700_000_060));
}

/// **Never is never, whatever the file says.** Missing, unreadable and
/// unparseable are one answer: a zero would be a date, and a client that has
/// not connected has no date.
#[test]
fn an_unusable_stamp_reads_as_never() {
    let root = tempdir().expect("tmp");
    let dir = crate::registry::dir(root.path(), &client());
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join(SEEN), b"whenever").expect("write");
    assert_eq!(read(root.path(), &client()), None);
    std::fs::remove_file(dir.join(SEEN)).expect("remove");
    std::fs::create_dir_all(dir.join(SEEN)).expect("a directory in its place");
    assert_eq!(read(root.path(), &client()), None);
}

/// A stamp that cannot be written is not an error: the caller is a request
/// being answered, and the roster losing a column is not worth refusing a
/// gesture over.
#[test]
fn a_stamp_that_cannot_be_written_is_silent() {
    let root = tempdir().expect("tmp");
    std::fs::write(root.path().join(crate::registry::CLIENTS), b"file").expect("write");
    mark(root.path(), &client(), 1_700_000_000);
    assert_eq!(read(root.path(), &client()), None);
}
