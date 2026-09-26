//! How a connection's silence is read.

use super::*;

fn timed_out() -> io::Error {
    io::Error::from(io::ErrorKind::TimedOut)
}

#[test]
fn a_dialled_connection_is_never_pinged() {
    let quiet = Quiet {
        gone: Duration::from_mins(2),
        ping: None,
    };
    assert_eq!(quiet.read_timeout(), Duration::from_mins(2));
    assert!(!quiet.pings_at(Duration::ZERO, &timed_out()));
}

#[test]
fn a_held_connection_is_pinged_until_the_bound() {
    let quiet = Quiet {
        gone: Duration::from_mins(2),
        ping: Some(Duration::from_secs(25)),
    };
    assert_eq!(quiet.read_timeout(), Duration::from_secs(25));
    assert!(quiet.pings_at(Duration::ZERO, &timed_out()));
    assert!(quiet.pings_at(Duration::from_secs(75), &timed_out()));
    assert!(quiet.pings_at(
        Duration::from_secs(75),
        &io::Error::from(io::ErrorKind::WouldBlock)
    ));
    assert!(
        !quiet.pings_at(Duration::from_secs(100), &timed_out()),
        "one more is the bound"
    );
    assert!(
        !quiet.pings_at(
            Duration::ZERO,
            &io::Error::from(io::ErrorKind::UnexpectedEof)
        ),
        "an EOF is the peer's own end"
    );
}

#[test]
fn the_ping_is_one_frame_a_reader_can_name() {
    assert_eq!(ping_frame(), json!({"ping": true}));
}
