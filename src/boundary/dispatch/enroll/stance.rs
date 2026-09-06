//! **A leaf that already exists is adopted, not refused** (REMOTE §1.4 as
//! amended, §4.1, §8.2; bl-bd48, bl-6b14) — what an enrollment does with the
//! name it was handed, in its own file on the seam [`enroll`](super)'s own doc
//! draws: everything there is the act's order, and this is the one question
//! asked before any of it runs.
//!
//! **The dead end it closes.** `yog wire-certs WIRE_LEAF=<name>` is the act the
//! binary's own help teaches for provisioning another box, and it mints a leaf
//! that is **registered in no workspace** — REMOTE §5.1's advertisement then
//! presents into the empty set, and the foot is connectable and useless. The
//! one act that registers is this one, and it refused: a pair already under
//! that name, because re-issuing distrusts nothing and two live certificates
//! under one identity is the hazard. Both halves were right and the conclusion
//! was wrong. **Registering is not issuing.** Adopting the standing pair mints
//! nothing, distrusts nothing and creates no second certificate — it seats the
//! registration the leaf was always missing, and hands over the material the
//! operator was going to carry by hand anyway.
//!
//! **The grade is read off the certificate and never taken on trust.** A grade
//! is minted into a subject by the operator's own CA (§4.2), so an adoption
//! that believed the word typed at a seat would be a promotion granted by
//! registration — exactly what default-operator exists to make impossible.

use std::path::Path;

use crate::registry::enroll::Request;
use crate::registry::{Grade, leaf};
use crate::wire::provision;

/// Mint this enrollment's leaf, or adopt the one already here — and answer the
/// grade the material actually carries.
///
/// The four states of a name are the four arms, and each names its own remedy:
/// nothing here is the mint; a whole pair is the adoption; a certificate whose
/// key is gone was enrolled through this door already, and its key left the box
/// then (the shred is what makes the answer the device's only copy); a key with
/// no certificate is debris no act can use.
pub(super) fn mint_or_adopt(dir: &Path, request: &Request) -> Result<Grade, String> {
    let cert = dir.join(format!("{}.pem", request.name));
    let key = dir.join(format!("{}.key", request.name));
    match (cert.is_file(), key.is_file()) {
        (false, false) => {
            provision::issue(dir, &request.name, request.grade).map(|()| request.grade)
        }
        (true, true) => adopt(&cert, request),
        (true, false) => Err(format!(
            "{} was enrolled already: its key left this box with that enrollment, so there is no \
             material to hand over a second time and re-issuing would put two live certificates \
             under one identity. The device is registered where it was enrolled; to seat it in \
             another workspace, write the registration on this box — `mkdir -p \
             <state-root>/{}/{}/{} && touch …/{}` — no gesture manages registrations, on \
             this engine or any other. State another common name to enrol a second device",
            request.name,
            crate::registry::CLIENTS,
            request.name,
            crate::registry::WORKSPACES,
            request.workspace
        )),
        (false, true) => Err(format!(
            "{}: a key with no certificate beside it — debris from a mint that did not finish. \
             Remove it and enrol again",
            key.display()
        )),
    }
}

/// The standing pair's own grade, refusing when it is not the one asked for.
///
/// The refusal is the whole of the grade's integrity here: adoption grants the
/// authority a certificate already carries and never a word more, so an
/// operator who asks for a foot and finds an operator leaf is told which one is
/// on disk rather than handed it under the other name.
fn adopt(cert: &Path, request: &Request) -> Result<Grade, String> {
    let grade = leaf::grade_at(cert)?;
    if grade == request.grade {
        return Ok(grade);
    }
    Err(format!(
        "{} is already here and is {} grade, but this enrollment asks for {}: a grade is minted \
         into the subject by the operator's own CA, so registering a certificate cannot change \
         what it says. Enrol it as {}, or state another common name and mint a fresh leaf",
        cert.display(),
        grade.word(),
        request.grade.word(),
        grade.word()
    ))
}
