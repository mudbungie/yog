//! The `world/tools/` shim's process body: one request in on stdin, one
//! verdict out on stdout, and every way the protocol can break failing closed.

use super::*;
use std::io::Write;

#[test]
fn the_shim_reads_one_request_and_prints_one_verdict() {
    let w = World::new();
    let env = crate::xdg::Env::from_pairs([
        ("HOME", w.dir.path().join("home").display().to_string()),
        (
            "XDG_STATE_HOME",
            w.dir.path().join("state").display().to_string(),
        ),
    ]);
    let mut out: Vec<u8> = Vec::new();
    let code = run(
        &mut request("bash", json!({"command": "ls"})).as_bytes(),
        &mut out,
        &env,
        &w.workspace(),
    );
    assert_eq!(code, 0);
    assert_eq!(String::from_utf8(out).unwrap(), "{\"verdict\":\"pass\"}\n");
    // A refusal is an answer, so it still exits 0 — the seam fails closed on a
    // non-zero exit, and a decline is not a fault.
    let mut out: Vec<u8> = Vec::new();
    let code = run(
        &mut request("bash", json!({"command": "bz --login"})).as_bytes(),
        &mut out,
        &env,
        &w.workspace(),
    );
    assert_eq!(code, 0);
    assert!(String::from_utf8(out).unwrap().contains("refuse"));
}

#[test]
fn an_unreadable_request_or_a_closed_stdout_fails_closed() {
    let w = World::new();
    let env = crate::xdg::Env::from_pairs([("HOME", "/home/op")]);
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(
        run(&mut "not json".as_bytes(), &mut out, &env, &w.workspace()),
        UNREADABLE
    );
    assert!(out.is_empty(), "nothing is answered for an unreadable ask");
    // Bytes that are not UTF-8 at all.
    let mut out: Vec<u8> = Vec::new();
    assert_eq!(
        run(&mut &[0xffu8][..], &mut out, &env, &w.workspace()),
        UNREADABLE
    );
    // A stdout that will not take the verdict is the same failure — including
    // when it is the flush that fails.
    assert!(Closed.flush().is_ok());
    assert_eq!(
        run(
            &mut request("bash", json!({"command": "ls"})).as_bytes(),
            &mut Closed,
            &env,
            &w.workspace(),
        ),
        UNREADABLE
    );
}
