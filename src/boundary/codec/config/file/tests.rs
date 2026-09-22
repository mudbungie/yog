//! The destination's own table (bl-edfc): every parameter of every §9
//! destination, read three ways — present, absent, and present but not a
//! string — because the two fault shapes are one sentence and a table that
//! only ever spelled the absent one would prove half of it.
//!
//! The subject is the **sentence**, not the decode: `decode_file`'s successes
//! already round-trip through `codec::config`'s parity table. What is asserted
//! here is that a refusal names the object it read (`target`), the destination
//! it was reading for, the field it wanted, and the fact no field name can
//! state — that this family carries its parameters a level down.

use super::{PLACE, decode_file};
use crate::boundary::config::ConfigFile;
use serde_json::{Value, json};

/// The five destinations and the parameters each takes — the table the two
/// refusal tests below are both driven from, so a destination that grows a
/// parameter grows both directions at once.
fn destinations() -> Vec<(&'static str, Value, Vec<&'static str>)> {
    vec![
        (
            "brazen",
            json!({ "file": "brazen", "workspace": "ws" }),
            vec!["workspace"],
        ),
        ("litany-models", json!({ "file": "litany-models" }), vec![]),
        (
            "litany-workflow",
            json!({ "file": "litany-workflow", "name": "review" }),
            vec!["name"],
        ),
        ("cadence", json!({ "file": "cadence" }), vec![]),
        (
            "branch",
            json!({ "file": "branch", "workspace": "ws", "lineage": "default",
                    "origin": "fork", "source": "base", "path": "providers.yaml" }),
            vec!["workspace", "lineage", "path", "origin", "source"],
        ),
    ]
}

/// Whole, every destination reads — the control arm, without which the two
/// refusal tests could pass over envelopes that were malformed to begin with.
#[test]
fn every_destination_reads_when_its_parameters_are_present() {
    for (file, whole, _) in destinations() {
        assert!(decode_file(&whole).is_ok(), "{file}: {whole}");
    }
}

/// The ball's own gesture: the workspace stated correctly, in the wrong place.
/// The old sentence was `missing or non-string field "workspace"` — true of
/// the object read and a flat contradiction of the envelope in front of the
/// operator, who has written a `workspace` and can see it.
#[test]
fn a_workspace_beside_target_instead_of_inside_it_is_told_where_it_belongs() {
    let err = decode_file(&json!({ "file": "brazen" })).unwrap_err();
    assert_eq!(
        err,
        "config: target for file \"brazen\": missing or non-string field \
         \"workspace\" — the config family carries its parameters INSIDE target"
    );
}

/// Absent and non-string, over every parameter of every destination: one
/// sentence, and it names the place in both fault shapes.
#[test]
fn every_destination_parameter_refuses_by_place_as_well_as_by_name() {
    for (file, whole, keys) in destinations() {
        for key in keys {
            for broken in [None, Some(json!(7))] {
                let mut obj = whole.as_object().cloned().unwrap_or_default();
                match broken {
                    None => obj.remove(key),
                    Some(v) => obj.insert(key.to_owned(), v),
                };
                let err = decode_file(&Value::Object(obj)).unwrap_err();
                assert!(err.contains(&format!("file {file:?}")), "{key}: {err}");
                assert!(err.contains(&format!("field {key:?}")), "{key}: {err}");
                assert!(err.ends_with(PLACE), "{key}: {err}");
            }
        }
    }
}

/// The two refusals about the target itself keep their own hand-written
/// sentences: they already name the place, because the place is all they are
/// about — and so does the unknown-token pair, which has a token to name and
/// no field.
#[test]
fn the_targets_own_refusals_are_unchanged() {
    assert_eq!(
        decode_file(&json!("brazen")).unwrap_err(),
        "config: target is not an object"
    );
    assert_eq!(
        decode_file(&json!({ "file": "enhance" })).unwrap_err(),
        "config: unknown target file \"enhance\""
    );
    let branch = json!({ "file": "branch", "workspace": "ws", "lineage": "d",
                         "origin": "rebase", "path": "p" });
    assert_eq!(
        decode_file(&branch).unwrap_err(),
        "config: unknown origin \"rebase\""
    );
}

/// The discriminant is read before any destination is known, so it cannot be
/// read by the place-naming wrapper — there is no file to name yet.
#[test]
fn a_target_naming_no_file_refuses_by_the_bare_field() {
    let err = decode_file(&json!({ "workspace": "ws" })).unwrap_err();
    assert_eq!(err, "missing or non-string field \"file\"");
}

/// The fork's `source` is the one parameter a sibling mode does not take, so
/// the advance and the orphan must still read with it absent.
#[test]
fn the_modes_that_take_no_source_read_without_one() {
    for origin in ["advance", "orphan"] {
        let v = json!({ "file": "branch", "workspace": "ws", "lineage": "d",
                        "origin": origin, "path": "p" });
        assert!(
            matches!(decode_file(&v), Ok(ConfigFile::Branch { .. })),
            "{v}"
        );
    }
}
