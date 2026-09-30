//! The doctor's answers (bl-28f4, bl-355c): a passing row, whose remedy is
//! ABSENT, and a failing one carrying the act — the one difference a seat
//! renders differently — once with no `rendezvous` object (a process with no
//! listener, or an engine before edition 21) and once with it, every counter
//! distinct so a decoder that transposed two would not pass by luck.

use super::super::super::super::super::Reply;
use crate::doctor::{Report, Row};
use crate::wire::rendezvous::Standing;

pub(super) fn doctor() -> Vec<Reply> {
    let rows = vec![
        Row {
            check: "listener".into(),
            ok: true,
            fact: "listening on 127.0.0.1:7737".into(),
            remedy: None,
        },
        Row {
            check: "address".into(),
            ok: false,
            fact: "127.0.0.1:0 is a request, not an endpoint".into(),
            remedy: Some("`WIRE_HOST=<host> WIRE_PORT=<port> yog wire-certs` states it".into()),
        },
    ];
    vec![
        Reply::Doctor(Report {
            rows: rows.clone(),
            rendezvous: None,
        }),
        Reply::Doctor(Report {
            rows,
            rendezvous: Some(Standing {
                active: true,
                published: 3,
                calls: 5,
                punched: 4,
                served: 6,
                accepted: 2,
                last_poll_unix: 1_700_000_015,
            }),
        }),
    ]
}
