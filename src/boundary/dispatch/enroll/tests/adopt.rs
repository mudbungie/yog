//! **A leaf that already exists is adopted, not refused** (bl-bd48, bl-6b14) —
//! [`stance`](super::super::stance)'s four states, through the act that spends
//! them.

use super::*;

/// **The dead end this closes.** `yog wire-certs WIRE_LEAF=devbox WIRE_FOOT=1`
/// is the act the binary's own help teaches for a second machine, and it
/// registers the leaf in nothing: the foot dials, advertises into the empty set
/// and is useless. `/enroll` was the only registrar and it refused that name,
/// offering a CA rotation. It adopts now — the same certificate, the same
/// identity, one registration seated and the material handed over.
#[test]
fn a_leaf_the_mint_already_issued_is_adopted_and_seated() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    provision::issue(&dir, "devbox", Grade::Foot).expect("the operator's own WIRE_LEAF");
    let minted = read(&dir.join("devbox.pem")).expect("leaf");

    let answer = enrolled(enroll(&deps, "7", &request("devbox", Grade::Foot)).expect("adopted"));
    assert_eq!(answer.name, "devbox");
    assert_eq!(answer.grade, Grade::Foot);
    assert_eq!(
        answer.cert, minted,
        "the certificate already carried away is the one adopted — nothing is re-issued"
    );
    assert!(answer.key.contains("PRIVATE KEY"), "and its key comes back");
    assert!(
        crate::registry::registered(
            &deps.state_root,
            &Client::parse("devbox").expect("identity")
        )
        .contains("alba"),
        "the registration the leaf was missing"
    );
    assert!(
        !dir.join("devbox.key").exists(),
        "shredded, as any enrollment"
    );
}

/// **A grade is minted into a subject and registering cannot change it**
/// (REMOTE §4.2). An adoption grants what the certificate already carries and
/// never a word more, so the mismatch is refused naming both grades — the one
/// on disk and the one asked for.
#[test]
fn an_adoption_refuses_to_grant_a_grade_the_certificate_does_not_carry() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    provision::issue(&dir, "devbox", Grade::Operator).expect("an operator-grade leaf");

    let refusal = enroll(&deps, "7", &request("devbox", Grade::Foot)).expect_err("refused");
    assert!(refusal.contains("operator"), "{refusal}");
    assert!(refusal.contains("foot"), "{refusal}");
    assert!(
        crate::registry::registered(
            &deps.state_root,
            &Client::parse("devbox").expect("identity")
        )
        .is_empty(),
        "and a refused enrollment seats nothing"
    );
}

/// A name enrolled through this door already has no key left to hand over — the
/// shred is what makes the first answer the device's only copy — so the refusal
/// says so and names the act that seats a registration by hand (§4.1).
#[test]
fn a_name_already_enrolled_refuses_naming_the_registration_act() {
    let tmp = tempdir().expect("tmp");
    let (deps, _) = provisioned(&tmp);
    enroll(&deps, "7", &request("phone-1", Grade::Operator)).expect("first");

    let refusal = enroll(&deps, "8", &request("phone-1", Grade::Operator)).expect_err("refused");
    assert!(refusal.contains("was enrolled already"), "{refusal}");
    assert!(refusal.contains("touch"), "{refusal}");
    assert!(refusal.contains(crate::registry::WORKSPACES), "{refusal}");
}

/// A key with no certificate beside it is debris from a mint that did not
/// finish: nothing to adopt and nothing to issue over, so the refusal names the
/// file to remove.
#[test]
fn a_key_with_no_certificate_is_named_as_debris() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    std::fs::write(dir.join("devbox.key"), b"a key that lost its leaf").expect("write");
    let refusal = enroll(&deps, "7", &request("devbox", Grade::Operator)).expect_err("refused");
    assert!(refusal.contains("devbox.key"), "{refusal}");
    assert!(refusal.contains("debris"), "{refusal}");
}

/// The grade is read off the certificate, so bytes that are not one refuse
/// naming the file rather than being taken for either grade.
#[test]
fn an_unreadable_certificate_refuses_naming_itself() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    for name in ["devbox.pem", "devbox.key"] {
        std::fs::write(dir.join(name), b"not a certificate").expect("write");
    }
    let refusal = enroll(&deps, "7", &request("devbox", Grade::Operator)).expect_err("refused");
    assert!(refusal.contains("devbox.pem"), "{refusal}");
}
