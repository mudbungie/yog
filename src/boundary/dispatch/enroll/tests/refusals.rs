//! Every way an enrollment refuses (REMOTE §1.4 as amended, §8) — split from
//! [`super`] at §12's budget, on the seam the codec's and the line's own test
//! corpora are already cut along: one file for what the act does, one for every
//! way it declines to do it.
//!
//! Two of them refuse **before `openssl` runs**, and both assert that nothing
//! was minted: an act that half-happened and then said no would leave a
//! certificate under a name the operator would have to clear by hand.

use super::super::*;
use super::{deps, provisioned, request};
use crate::registry::Grade;
use tempfile::tempdir;

/// A box that provisioned itself wrote `127.0.0.1:0` — a request the listener
/// answered in RAM. Enrolling against that would mint a QR nothing could dial,
/// so it refuses before `openssl` runs, naming the operator's own act.
#[test]
fn a_kernel_chosen_port_refuses_before_anything_is_minted() {
    let tmp = tempdir().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    provision::ensure(&dir).expect("the boot's own mint");
    let deps = deps(&world, &tmp.path().join("state-root"));

    let refusal = enroll(&deps, "7", &request("phone-1", Grade::Operator)).expect_err("refused");
    assert!(
        refusal.contains("names no port a device can dial"),
        "{refusal}"
    );
    assert!(refusal.contains("WIRE_HOST"), "{refusal}");
    assert!(!dir.join("phone-1.pem").exists(), "nothing was minted");
}

/// A box holding no material at all is not a box that can issue anything: the
/// refusal names the target that mints a trust root.
#[test]
fn a_box_with_no_material_refuses_naming_the_remedy() {
    let tmp = tempdir().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let refusal = enroll(
        &deps(&world, &tmp.path().join("state-root")),
        "7",
        &request("phone-1", Grade::Operator),
    )
    .expect_err("refused");
    assert!(refusal.contains("holds no wire material"), "{refusal}");
    assert!(refusal.contains(material::REMEDY), "{refusal}");
}

/// A half-provisioned directory speaks in `material`'s own words — one sentence
/// naming every file that is missing.
#[test]
fn a_half_provisioned_box_refuses_in_materials_words() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    std::fs::remove_file(dir.join("server.key")).expect("remove");
    let refusal = enroll(&deps, "7", &request("phone-1", Grade::Operator)).expect_err("refused");
    assert!(refusal.contains("half-provisioned"), "{refusal}");
}

/// The identity is judged first, so a name that could address the filesystem
/// never reaches the mint.
#[test]
fn an_unusable_identity_refuses_before_the_mint() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    let refusal =
        enroll(&deps, "7", &request("../elsewhere", Grade::Operator)).expect_err("refused");
    assert!(refusal.contains("unusable client identity"), "{refusal}");
    assert!(!dir.join("../elsewhere.pem").exists());
}

/// A registration that cannot be written is a refusal, not a panic — and the
/// key is already shredded by then, so a failed seat leaves none behind.
#[test]
fn an_unwritable_registry_refuses_with_no_key_left_behind() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    std::fs::create_dir_all(&deps.state_root).expect("state root");
    std::fs::write(deps.state_root.join(crate::registry::CLIENTS), b"file").expect("write");
    assert!(enroll(&deps, "7", &request("phone-1", Grade::Operator)).is_err());
    assert!(!dir.join("phone-1.key").exists(), "shredded regardless");
}

/// A trail that cannot be appended to is a refusal for the same reason: no
/// error class is dropped (INV-2).
#[test]
fn an_unwritable_trail_refuses() {
    let tmp = tempdir().expect("tmp");
    let (deps, _) = provisioned(&tmp);
    std::fs::create_dir_all(deps.state_root.join("ops.jsonl")).expect("a directory in its place");
    assert!(enroll(&deps, "7", &request("phone-1", Grade::Operator)).is_err());
}

/// A key that cannot be removed is a key still on the box, so it is an error
/// and the sentence names the file.
#[test]
fn a_key_that_cannot_be_shredded_refuses_naming_it() {
    let tmp = tempdir().expect("tmp");
    let refusal = carry(tmp.path(), "absent").expect_err("nothing to shred");
    assert!(refusal.contains("could not be shredded"), "{refusal}");
}

/// The three reads each refuse in their own words, and the shred stands ahead
/// of all of them: an unreadable key is reported *after* it is gone.
#[test]
fn each_unreadable_pem_refuses_naming_itself() {
    for (missing, marker) in [
        ("phone-1.key", "phone-1.key"),
        (material::ANCHORS, material::ANCHORS),
        ("phone-1.pem", "phone-1.pem"),
    ] {
        let tmp = tempdir().expect("tmp");
        for name in [material::ANCHORS, "phone-1.pem"] {
            std::fs::write(tmp.path().join(name), b"pem").expect("write");
        }
        // Invalid UTF-8 rather than absent, so the key is readable-as-a-file —
        // the shred succeeds and the read is what refuses.
        std::fs::write(tmp.path().join("phone-1.key"), [0xff, 0xfe]).expect("write");
        if missing != "phone-1.key" {
            std::fs::remove_file(tmp.path().join(missing)).expect("remove");
            std::fs::write(tmp.path().join("phone-1.key"), b"pem").expect("write");
        }
        let refusal = carry(tmp.path(), "phone-1").expect_err("refused");
        assert!(refusal.contains(marker), "{refusal}");
        assert!(!tmp.path().join("phone-1.key").exists(), "shredded first");
    }
}
