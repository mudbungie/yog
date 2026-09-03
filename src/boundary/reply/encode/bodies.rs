//! The reply spellings that are **bodies, not rows** — split from
//! [`super`] at §12's cap on the seam that file's own text already draws: the
//! roster there is a match of one-line arms, and an arm that assembles a
//! four-key object is a body pretending to be a row. Five of them, and the
//! roster still names each, so there is exactly one place to learn which reply
//! is said where.

use serde_json::{Value, json};

use super::{ENROLLED, LOGIN, obj_reply};

/// A captured run as its receipt (§7.3): the verb's own exit and streams,
/// with `ok` derived from the exit rather than stored beside it.
///
/// A body rather than a row for the reason [`delivered`] and [`governing`] are
/// bodies beside it — an arm that builds a four-key object is a body, and
/// the roster stops reading as one once any of them is.
pub(super) fn outcome_reply(outcome: &crate::actions::verbs::Outcome) -> Value {
    json!({
        "ok": outcome.ok(), "kind": "outcome", "exit": outcome.exit,
        "stdout": outcome.stdout, "stderr": outcome.stderr,
    })
}

/// The delivery's four identities (V3.2, bl-c2bd). The two options are
/// **absent** rather than null, upstream's own meaning kept: an unmade source
/// ref and a delivery that landed nothing are absences, not empty strings.
pub(super) fn delivered(delivery: &crate::fan::Delivery) -> Value {
    let mut map = obj_reply("delivered");
    map.insert("target".to_owned(), json!(delivery.target));
    map.insert("base".to_owned(), json!(delivery.base));
    if let Some(source) = &delivery.source {
        map.insert("source".to_owned(), json!(source));
    }
    if let Some(commit) = &delivery.commit {
        map.insert("commit".to_owned(), json!(commit));
    }
    Value::Object(map)
}

/// The QR envelope's payload (REMOTE §1.4 as amended, bl-f4e3). Every field is
/// present always — there is no absent case, because a device handed five of
/// the six facts cannot dial, cannot verify, or cannot say who it is — and the
/// three PEMs ride **verbatim**, newlines and all: the envelope measures 1567
/// bytes of compact JSON against a byte-mode QR's 2953, so nothing is
/// re-encoded to buy room it does not need.
pub(super) fn enrolled_reply(enrolled: &crate::registry::enroll::Enrolled) -> Value {
    json!({
        "ok": true, "kind": ENROLLED, "grade": enrolled.grade.word(),
        "name": enrolled.name, "address": enrolled.address,
        "ca": enrolled.ca, "cert": enrolled.cert, "key": enrolled.key,
    })
}

/// The which-config-governs answer (bl-13f9; follow-the-tip, bl-e654). The oid
/// is the **resolved** commit's and rides both ways — short is what a pane
/// labels with, full is what a `git show` outside yog takes — exactly as a
/// lineage row's tip does. `follows` and `diverged_lineages` are the two faces
/// of one enum: a name and `0`, or `null` and the count that held it.
pub(super) fn governing(gov: &crate::config_edit::branch::GoverningConfig) -> Value {
    json!({
        "ok": true, "kind": "governing",
        "oid": gov.oid, "short_oid": gov.short_oid,
        "follows": gov.followed_lineage(),
        "diverged_lineages": gov.diverged_lineages(),
        "files": gov.files,
    })
}

/// The sign-in's standing (REMOTE §8.3, bl-c285) — the envelope, and the body
/// spelled beside its own type (`login::wire`), so one shape serves the act's
/// receipt and every lane frame.
pub(super) fn login_reply(view: &crate::login::LoginView) -> Value {
    let mut map = obj_reply(LOGIN);
    map.append(&mut crate::login::wire::body(view));
    Value::Object(map)
}
