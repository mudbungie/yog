//! **The standing record's rule, mechanically** (REMOTE §3.2): a gain is
//! stamped the next edition and costs nothing; a loss or a re-type is refused
//! under the published major, and across a bump a loss still needs its
//! deprecation; the floor moves once, at the bump.

use serde_json::{Value, json};

use crate::boundary::corpus::ledger::{Ledger, advance};
use crate::boundary::corpus::{Shape, canonical};

fn ack(frame: Value) -> Shape {
    Shape {
        direction: "request",
        name: "ack".to_owned(),
        frames: vec![frame],
    }
}

/// A record at `protocol` with `floor`, holding one `ack` whose paths carry
/// the given stamps.
fn recorded(protocol: u32, floor: u32, stamps: &[(&str, u32)]) -> Ledger {
    let signature: serde_json::Map<String, Value> = stamps
        .iter()
        .map(|(path, at)| ((*path).to_owned(), json!(at)))
        .collect();
    Ledger::read(&canonical(&json!({
        "protocol": protocol,
        "floor": floor,
        "shapes": { "request/ack": { "signature": signature } },
    })))
}

const ACK: &[(&str, u32)] = &[("/op:string", 1), (":object", 1)];

/// **An addition is free, and it is stamped** (REMOTE §3.2): the gained path
/// takes the next edition, the paths that stood keep theirs, and neither the
/// major nor the floor moves.
#[test]
fn a_gained_field_is_stamped_the_next_edition_and_needs_no_bump() {
    let previous = recorded(
        19,
        18,
        &[("/op:string", 1), (":object", 1), ("/late:bool", 18)],
    );
    let next = advance(
        &[ack(json!({ "op": "ack", "late": true, "reason": "why" }))],
        &previous,
        19,
        19,
        &[],
    )
    .expect("additive");
    let stamps = &next.shapes["request/ack"].signature;
    assert_eq!(stamps["/reason:string"], 19, "the gain");
    assert_eq!(stamps["/late:bool"], 18, "what stood");
    assert_eq!(stamps["/op:string"], 1);
    assert_eq!((next.protocol, next.floor), (19, 18));
    assert_eq!(next.edition(), 19);
}

/// A **new** shape is an addition like any other: every path of it is the
/// next edition, so a seat can grey an op the engine it dialled has not got.
#[test]
fn a_new_shape_is_stamped_the_next_edition() {
    let fresh = Shape {
        direction: "request",
        name: "novel".to_owned(),
        frames: vec![json!({ "op": "novel" })],
    };
    let next = advance(
        &[ack(json!({ "op": "ack" })), fresh],
        &recorded(19, 18, ACK),
        19,
        19,
        &[],
    )
    .expect("additive");
    assert!(
        next.shapes["request/novel"]
            .signature
            .values()
            .all(|&at| at == 2)
    );
    assert_eq!(
        Ledger::read("").edition(),
        0,
        "an empty record is edition 0"
    );
}

/// **A loss is a MAJOR bump, and a deprecation first.** Under the published
/// major it is refused outright; with a bump in flight it is still refused
/// until the path is listed; and the sentence names the path, the list, the
/// file and the command.
#[test]
fn a_vanished_field_demands_the_major_and_a_deprecation() {
    let previous = recorded(
        19,
        18,
        &[("/op:string", 1), (":object", 1), ("/gone:bool", 2)],
    );
    let shapes = [ack(json!({ "op": "ack" }))];
    let held = advance(&shapes, &previous, 19, 19, &["request/ack/gone"])
        .expect_err("refused under the major");
    assert!(held.contains("request/ack/gone:bool vanished"), "{held}");
    assert!(held.contains("DEPRECATED"), "{held}");
    assert!(held.contains("PROTOCOL file"), "{held}");
    assert!(held.contains("make corpus"), "{held}");
    let bumping = advance(&shapes, &previous, 20, 19, &[]).expect_err("refused undeprecated");
    assert!(bumping.contains("vanished undeprecated"), "{bumping}");
    // Deprecated by path, and across the bump: lawful, and the record says
    // what was deprecated so a consumer reads it where it reads the rest.
    let next = advance(&shapes, &previous, 20, 19, &["request/ack/gone"]).expect("lawful");
    assert!(next.deprecated.contains("request/ack/gone"));
    assert!(
        !next.shapes["request/ack"]
            .signature
            .contains_key("/gone:bool")
    );
    assert_eq!(
        next.render(),
        Ledger::read(&next.render()).render(),
        "round trip"
    );
}

/// A whole shape that vanished is every path of it vanishing, named.
#[test]
fn a_vanished_shape_is_named_too() {
    let refusal = advance(&[], &recorded(19, 18, ACK), 19, 19, &[]).expect_err("refused");
    assert!(refusal.contains("request/ack:object vanished"), "{refusal}");
    // Deprecated by shape name, every path of it is licensed at once.
    let gone = advance(&[], &recorded(19, 18, ACK), 20, 19, &["request/ack"]).expect("lawful");
    assert!(gone.shapes.is_empty());
}

/// **A re-type is a change, not a gain**: a key in use that starts spelling a
/// second JSON type breaks every reader built against the first, so it is
/// refused under the published major and lawful only across a bump.
#[test]
fn a_second_type_on_a_key_in_use_is_breaking() {
    let previous = recorded(19, 18, ACK);
    let shapes = [ack(json!({ "op": 7 }))];
    let refusal = advance(&shapes, &previous, 19, 19, &[]).expect_err("re-typed");
    assert!(
        refusal.contains("request/ack/op:number re-typed"),
        "{refusal}"
    );
    // Across a bump the old spelling must still be deprecated to vanish, so
    // a nullable widening — both types kept — is the lawful shape of it.
    let widened = [Shape {
        direction: "request",
        name: "ack".to_owned(),
        frames: vec![json!({ "op": "ack" }), json!({ "op": Value::Null })],
    }];
    let next = advance(&widened, &previous, 20, 19, &[]).expect("lawful across a bump");
    assert_eq!(next.shapes["request/ack"].signature["/op:null"], 2);
}

/// **The floor moves at the bump's own regeneration and never after**: it
/// becomes the edition the record is then at, and an addition in a later
/// regeneration lands above it.
#[test]
fn the_floor_is_set_by_the_bump_and_left_by_everything_else() {
    let bumped = advance(
        &[ack(json!({ "op": "ack" }))],
        &recorded(18, 0, ACK),
        19,
        17,
        &[],
    )
    .expect("the bump");
    assert_eq!((bumped.protocol, bumped.floor), (19, 1));
    let later = advance(
        &[ack(json!({ "op": "ack", "more": 1 }))],
        &bumped,
        19,
        19,
        &[],
    )
    .expect("additive");
    assert_eq!(later.floor, 1, "unmoved");
    assert_eq!(
        later.shapes["request/ack"].signature["/more:number"], 2,
        "post-floor"
    );
}
