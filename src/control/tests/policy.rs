//! The per-workspace capability policy, read at the live tip by the real
//! process body, and the bounded input summary the hold mark carries.

use super::*;

/// The workspace's own policy, read at the live tip by the shim itself: the
/// same `curl` that passes under the shipped table parks under a workspace
/// that says so, and the same `python` that passes unmatched is classified by
/// an operator row. Severability, end to end and through the real process
/// body — nothing here is a test seam.
#[test]
fn the_shim_reads_the_workspace_s_own_policy_at_its_live_tip() {
    let w = World::new();
    let env = crate::xdg::Env::from_pairs([
        ("HOME", w.dir.path().join("home").display().to_string()),
        (
            "XDG_STATE_HOME",
            w.dir.path().join("state").display().to_string(),
        ),
    ]);
    let verdict = |w: &World, command: &str| {
        let mut out: Vec<u8> = Vec::new();
        assert_eq!(
            run(
                &mut request("bash", json!({ "command": command })).as_bytes(),
                &mut out,
                &env,
                &w.workspace(),
            ),
            0
        );
        String::from_utf8(out).unwrap()
    };
    // Shipped: an unmatched program and a network reach both pass.
    assert!(verdict(&w, "curl x").contains("pass"));
    assert!(verdict(&w, "python go.py").contains("pass"));
    w.policy(concat!(
        "table:\n",
        "  open-world: hold\n",
        "rules:\n",
        "  python: destructive\n",
        "secrets:\n",
        "  - .kube\n",
    ));
    // The table override parks the open-world reach the shipped table let by…
    assert!(verdict(&w, "curl x").contains("hold"));
    // …the operator's own row classifies what no shipped row named…
    assert!(verdict(&w, "python go.py").contains("refuse"));
    // …and the added secret fragment outranks the program's read row.
    assert!(verdict(&w, "cat .kube/config").contains("refuse"));
}

/// The reason is what the operator reads off the hold mark, so it names what
/// the invocation was about to do — bounded, because a mark is read at a
/// glance and a model can write a megabyte of `command`.
#[test]
fn the_reason_carries_a_bounded_input_summary() {
    let w = World::new();
    // Under a workspace that parks the open world — the sentence is what a
    // parked operator reads, so it is written where a hold exists.
    let parked = Consult {
        policy: Policy::parse("table:\n  open-world: hold\n"),
        ..w.consult()
    };
    let short = adjudicate(
        &parked,
        &Request::parse(&request("bash", json!({"command": "curl x"}))).unwrap(),
    );
    let Verdict::Hold(reason) = short else {
        panic!("the override holds open-world");
    };
    assert!(reason.contains("curl x"), "{reason}");
    assert!(reason.contains("open-world"), "{reason}");

    let long = adjudicate(
        &parked,
        &Request::parse(&request(
            "bash",
            json!({"command": "curl ".to_owned() + &"x".repeat(500) }),
        ))
        .unwrap(),
    );
    let Verdict::Hold(reason) = long else {
        panic!("the override holds open-world");
    };
    assert!(reason.contains('…'), "a runaway input is cut: {reason}");
    assert!(reason.len() < 400, "and stays readable: {}", reason.len());
}
