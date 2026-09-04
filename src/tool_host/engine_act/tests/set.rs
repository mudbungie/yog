//! **The closed set, and the two facts that keep a machine out of it**
//! (bl-fe43, bl-81cc): the enumeration itself, an enrolled thrall that
//! advertises an engine act and still never hears about the call, and the
//! invocation id every spawn carries.

use super::*;

/// The set is closed and enumerated in one place, and every member goes
/// through it. The eight rows are the three subject-locality families: the
/// compactor's procedure pair, the conversation-subject worker grants, and the
/// two acts on the agent's own record and history (`python`'s inner
/// invocations land under the in-flight step and re-enter the front door;
/// `search_history` reads the workspace's `agents/*` refs). The worktree names
/// (`bash`, `read_file`, `apply_patch`) stay out, for the reason the roster's
/// doc states — their subject is the working tree, which a consenting machine
/// may hold.
#[test]
fn the_eight_names_are_engine_acts_and_nothing_else_is() {
    assert_eq!(
        NAMES,
        [
            "write_summary",
            "mark_for_deletion",
            "dispatch",
            "message",
            "load_skill",
            "cd",
            "python",
            "search_history",
        ]
    );
    assert!(NAMES.iter().copied().all(is));
    for machine_work in ["bash", "read_file", "apply_patch", "Bash", "Python"] {
        assert!(!is(machine_work), "{machine_work} is not an engine act");
    }
    assert!(!is("write_summary_2"));
}

/// **Compaction never reaches a thrall's mailbox**, even with one enrolled and
/// its tools loaded: the pair's subject is the conversation, so no invocation
/// is queued at the engine at all. The loaded set is present precisely so the
/// beat can say the router had a machine to route to and did not use it.
#[test]
fn an_engine_act_never_reaches_an_enrolled_thralls_mailbox() {
    let root = TempDir::new().expect("tmp");
    loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[loaded::Entry {
            client: "laptop".to_owned(),
            tool: tool("Bash"),
        }],
    )
    .expect("loaded");
    let door = front_door(root.path(), "printf marked");
    let input = json!({"path": "messages/004-user.md"});
    let stop = AtomicBool::new(false);

    let capture = Injection::new(
        root.path().to_path_buf(),
        door,
        budget(),
        budget(),
        FakeClock::new().arc(),
    )
    .route(act!("mark_for_deletion", root.path(), &input, &stop));

    assert_eq!(capture.exit_code, 0);
    assert_eq!(capture.stdout, b"marked");
    assert!(
        deposit::pending(root.path()).is_empty(),
        "an engine act queues nothing at the engine"
    );
}

/// **A foot never sees `python` or `search_history`** (bl-fe43, bl-81cc), and a
/// thrall that advertises both by those very names does not change it: each
/// one is performed at the engine's own front door, and the engine is asked
/// **nothing** — not the roster read the worktree lane opens with, and not the
/// invoke gesture a routing leg would queue. That is the whole difference the
/// two rows buy. Routed, `python` would compose its inner invocations against
/// a step record the foot does not hold, and `search_history` would run its
/// pickaxe over a box with no repository on it and answer *nothing found*.
#[test]
fn a_foot_never_sees_python_or_search_history() {
    let root = TempDir::new().expect("tmp");
    loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[
            loaded::Entry {
                client: "laptop".to_owned(),
                tool: tool("python"),
            },
            loaded::Entry {
                client: "laptop".to_owned(),
                tool: tool("search_history"),
            },
        ],
    )
    .expect("loaded");
    let door = front_door(root.path(), "printf '%s|%s' \"$1\" \"$2\"");
    let stop = AtomicBool::new(false);
    for (name, input) in [
        ("python", json!({"program": "print(1)"})),
        ("search_history", json!({"pattern": "the wedge"})),
    ] {
        let capture = Injection::new(
            root.path().to_path_buf(),
            door.clone(),
            budget(),
            budget(),
            FakeClock::new().arc(),
        )
        .route(act!(name, root.path(), &input, &stop));

        assert_eq!(capture.exit_code, 0, "{name}");
        assert_eq!(
            String::from_utf8_lossy(&capture.stdout),
            format!("tool|{name}"),
            "{name} is performed at the engine's own front door"
        );
        assert!(
            deposit::pending(root.path()).is_empty(),
            "{name} queued something at the engine"
        );
    }
}

/// **The invocation's own id rides the child's environment** (litany's §3.3
/// stdio contract, upstream bl-e8d7): every spawn this router makes for a call
/// carries `LITANY_TOOL_ID`, so the id the child reads and the
/// `steps/<agent-id>/<NNN>/tools/<tool-id>/` directory it writes beside cannot
/// disagree. `python` is the act that cannot work without it — it names each
/// inner invocation's record directory from that variable.
#[test]
fn every_spawn_carries_the_invocations_own_id() {
    let root = TempDir::new().expect("tmp");
    let door = front_door(root.path(), "printf '%s' \"$LITANY_TOOL_ID\"");
    let input = json!({"program": "print(1)"});
    let stop = AtomicBool::new(false);

    let capture = perform(
        &door,
        budget().span(),
        &act!("python", root.path(), &input, &stop),
    );

    assert_eq!(capture.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&capture.stdout), "toolu_9");
}
