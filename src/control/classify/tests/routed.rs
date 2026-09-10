//! The **fail-closed lane** (bl-72bd, bl-1772): a routed tool's command line,
//! read exactly as the engine's own `bash` is and against a machine none of
//! whose paths this control vouches for.

use super::super::Effect;
use super::{effect, judged};
use serde_json::json;

/// Ruling 2 of the round-1 triage, first half: a routed tool whose input
/// carries a command line is classified by that line exactly as the engine's
/// own `bash` is. The drive that filed bl-72bd deleted 180 MB through the
/// first of these and was not asked a question.
#[test]
fn a_routed_shell_is_classified_by_its_command_line() {
    let destructive = judged(
        "box2_shell",
        json!({"command": "find /srv/data/blobs -mindepth 1 -delete"}),
    );
    assert_eq!(destructive.effect, Effect::Destructive);
    assert!(
        destructive.why.contains("/srv/data/blobs"),
        "{}",
        destructive.why
    );
    assert_eq!(
        effect("box2_shell", json!({"command": "env"})),
        Effect::Secret
    );
    // …and the same table, so a read on the foot is still a read.
    assert_eq!(
        effect("box2_shell", json!({"command": "ls -la"})),
        Effect::Read
    );
    // The engine's own bash answers identically — that is what "exactly as"
    // means, and the two are one call into one ruleset.
    assert_eq!(
        judged(
            "bash",
            json!({"command": "find /srv/data/blobs -mindepth 1 -delete"})
        ),
        destructive
    );
}

/// Ruling 2, second half: a tool this control cannot read is HELD, never
/// passed. The class it lands in is the one the shipped table parks.
#[test]
fn a_tool_this_control_cannot_read_is_opaque_and_never_a_passing_class() {
    for (name, input) in [
        ("litany-tool-deploy", json!({})),
        ("box2_install_package", json!({"name": "curl"})),
        ("box2_rotate_log", json!({"path": "/var/log/x"})),
        // A `command` key that is not a string, and an empty one: neither is a
        // command line, so neither may borrow the shell's classification.
        ("box2_shell", json!({"command": 7})),
        ("box2_shell", json!({"command": "   "})),
    ] {
        let c = judged(name, input);
        assert_eq!(c.effect, Effect::Opaque, "{name}");
        assert!(c.why.contains(name), "{}", c.why);
        assert_eq!(
            crate::control::judge::Table::ruling(c.effect),
            crate::control::judge::Ruling::Hold,
            "{name}"
        );
    }
}

/// bl-1772: the `cd` the drive found. A relative operand used to resolve
/// against the agent's own cwd — on the engine, inside the writable root — so
/// `cd <dir> && rm -f -- *` read as a target write while the identical
/// `rm -f <dir>/*` read as loss. This control vouches for no path on a foot, so
/// the routed leg's writable set is empty and the two spellings are one act.
#[test]
fn a_cd_into_the_target_classifies_as_the_direct_form_does() {
    let direct = judged("box2_shell", json!({"command": "rm -f /srv/data/blobs/*"}));
    assert_eq!(direct.effect, Effect::Destructive);
    let chained = judged(
        "box2_shell",
        json!({"command": "cd /srv/data/blobs && rm -f -- *"}),
    );
    assert_eq!(chained.effect, Effect::Destructive, "{}", chained.why);
    // A path inside the ENGINE's writable root is on the other machine too, and
    // is judged by that: nothing routed lands in the root by spelling.
    assert_eq!(
        effect("box2_shell", json!({"command": "rm -rf /w/agent/build"})),
        Effect::Destructive
    );
    // The engine's own bash is untouched — its root is the one this control can
    // actually vouch for, and work inside it is still the job.
    assert_eq!(
        effect("bash", json!({"command": "rm -rf /w/agent/build"})),
        Effect::TargetWrite
    );
    // Reads on a foot stay reads: emptying the root moves the rows that judge
    // by operand, and nothing else.
    assert_eq!(
        effect("box2_shell", json!({"command": "ls -la /srv/data"})),
        Effect::Read
    );
}

/// bl-c6f0: the third answer. A routed tool with no command line and a `url`
/// input reaches the network, and network egress is what the open-world class
/// means — so the fetch tool `thrall mcp pin` seats (`{"url": …, "max_length":
/// …, "raw": …}`) passes on the shipped table instead of parking every call.
/// The same reading as a command line, on the other operand a routed schema
/// carries.
#[test]
fn an_input_naming_a_url_is_the_open_world_class() {
    let c = judged("box2_fetch", json!({"url": "https://example.invalid/x"}));
    assert_eq!(c.effect, Effect::OpenWorld);
    assert!(c.why.contains("box2_fetch"), "{}", c.why);
    assert!(c.why.contains("url"), "{}", c.why);
    // …and the shipped table passes it, which is the point of the class.
    assert_eq!(
        crate::control::judge::Table::ruling(c.effect),
        crate::control::judge::Ruling::Pass
    );
    // The batched spelling reads the same, and so does a whole pinned schema.
    assert_eq!(
        effect("box2_fetch", json!({"urls": ["https://a.invalid"]})),
        Effect::OpenWorld
    );
    assert_eq!(
        effect(
            "box2_fetch",
            json!({"url": "https://a.invalid", "max_length": 5000, "raw": false})
        ),
        Effect::OpenWorld
    );
    // A command line still outranks it: a shell that happens to carry a `url`
    // field is classified by the line, exactly as before.
    assert_eq!(
        effect(
            "box2_shell",
            json!({"command": "rm -rf /srv/data", "url": "https://a.invalid"})
        ),
        Effect::Destructive
    );
}

/// The other direction: an input that names no address is the hold it was. A
/// blank string, an empty list, a list of blanks and a non-string all name
/// nothing — an off-schema value is an operand this control cannot read, which
/// is the park rather than a guess.
#[test]
fn an_input_naming_no_address_stays_opaque() {
    for input in [
        json!({}),
        json!({"name": "curl"}),
        json!({"url": ""}),
        json!({"url": "   "}),
        json!({"url": 7}),
        json!({"urls": []}),
        json!({"urls": ["  ", 7]}),
        json!({"url_prefix": "https://a.invalid"}),
    ] {
        assert_eq!(
            effect("box2_fetch", input.clone()),
            Effect::Opaque,
            "{input}"
        );
    }
}
