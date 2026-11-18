//! **A capture is N ≥ 1 posts, at the boundary** (REMOTE §5.7, bl-d414).

use super::*;
use crate::registry::mailbox::{Offsets, Piece};

/// **The streamed form, through the same four arms** (REMOTE §5.7): a piece
/// posted while the run goes on rides back to an asker holding offsets under
/// `partial`, and the capture that ends the run carries the remainder.
#[test]
fn a_piece_crosses_while_the_run_is_live_and_the_capture_is_the_remainder() {
    let root = tempdir().expect("tmp");
    let mail = Mailbox::holding(2, Duration::from_millis(1));
    let laptop = Client::parse("laptop").expect("identity");
    tools::store(root.path(), &laptop, &[tool("bz")]).expect("advertise");
    let driver = deps(root.path(), crate::registry::LOCAL, &mail);
    let host = deps(root.path(), "laptop", &mail);
    let id = handle(invoke(&driver, "100", &call("laptop", "bz")));
    assert!(invocations(&host).is_ok());

    let piece = Piece {
        stdout: "hel".to_owned(),
        stderr: String::new(),
    };
    let posted = route(
        &host,
        "101",
        &Verb::Complete(Completion {
            invocation: id.clone(),
            post: Post::Partial(piece.clone()),
        }),
    );
    assert_eq!(
        posted,
        Ok(Reply::Routed {
            invocation: id.clone(),
            standing: Standing {
                partial: Some(Piece::default()),
                capture: None,
            },
        })
    );
    let since = Some(Offsets::default());
    assert_eq!(
        capture(&driver, 101, &id, since),
        Ok(Reply::Routed {
            invocation: id.clone(),
            standing: Standing {
                partial: Some(piece),
                capture: None,
            },
        })
    );
    let done = Completion {
        invocation: id.clone(),
        post: Post::Capture(ran()),
    };
    assert!(complete(&host, "102", &done).is_ok());
    let rest = Some(Offsets {
        stdout: 3,
        stderr: 0,
    });
    assert_eq!(
        capture(&driver, 102, &id, rest),
        Ok(Reply::Routed {
            invocation: id,
            standing: over(),
        })
    );
}
