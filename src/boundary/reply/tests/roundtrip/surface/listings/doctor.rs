//! The doctor's answers (bl-28f4): a passing row, whose remedy is ABSENT, and
//! a failing one carrying the act — the one difference a seat renders
//! differently.

use super::super::super::super::super::Reply;
use crate::doctor::Row;

pub(super) fn doctor() -> Vec<Reply> {
    vec![Reply::Doctor(vec![
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
    ])]
}
