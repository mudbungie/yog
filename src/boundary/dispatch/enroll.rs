//! **Enrollment's executor** (REMOTE §1.4 as amended, §4.2, §8; bl-f4e3) —
//! beside [`advertise`](super::advertise) and [`delete_exec`](super::delete_exec)
//! for their reason: everything else in the chokepoint routes, and these
//! *gate*.
//!
//! **The grade gate is already spent, one layer up, and this file adds none**
//! (REMOTE §4.2). A foot may say three gestures — `advertise`, `invocations`,
//! `complete` — and the set is *enumerated* in
//! [`Grade::admits`](crate::registry::Grade::admits) rather than subtracted
//! from the operator's roster, so an [`Action`](crate::boundary::Action) added
//! today is operator-only by construction: `answer_as` refuses it in band,
//! naming the grade, ahead of the dispatch and ahead of the auto-registration.
//! That is why nothing here asks what grade the caller is. A check written
//! again here would be a second authority for one fact, and the second one is
//! the one that drifts.
//!
//! **An in-world caller is admitted, and that is the act's own class.** The
//! `gestures/` inbox and `yog gesture` carry no certificate (§4.1) and are the
//! world's own residents — the operator, at a terminal, on the box that holds
//! the CA. Enrolling from there is `yog wire-certs WIRE_LEAF=…` reached through
//! the boundary, which is exactly what §3 asks for: one surface, every face.
//!
//! **Nothing here dials anything.** The material is answered, never delivered;
//! §1.4 stands because the new device performs no channel act — holding no
//! certificate, it could not open a connection at all — and the bytes reach it
//! out of channel, in the operator's hand.

use std::path::Path;

use crate::opslog::Origin;
use crate::registry::enroll::{Enrolled, Request};
use crate::registry::{self, Client};
use crate::wire::{material, provision};

use super::Deps;
use crate::boundary::reply::Reply;

/// The trail's word for this act (§4.2) — `["yog-step","enroll"]` at exit 0.
/// **The row names no material**: an ops line is `argv`, `cwd` and a status,
/// and the fact worth recording is that an enrollment happened here.
const STEP: &str = "enroll";

/// Mint, read, shred, seat, log — each step refusing with its own sentence.
///
/// The order is the fail-closed one. The identity is parsed first, so an
/// unusable name refuses before `openssl` runs; the address second, because
/// material a device cannot dial is not worth minting; the mint third, being
/// the only step that can fail for a reason outside yog; and the registration
/// last, its input already validated by the chokepoint's own resolution.
/// Neither half-landing is a hazard — a registration with no certificate grants
/// nothing and a certificate with no registration sees nothing — but a mint
/// whose material never reached the answer would leave a live key on disk, and
/// [`carry`] closes that by shredding whether or not the reads succeeded.
pub(super) fn enroll(deps: &Deps, ts: &str, request: &Request) -> Result<Reply, String> {
    let client = Client::parse(&request.name)?;
    let dir = material::dir(&deps.world);
    let address = dialable(&dir)?;
    provision::issue(&dir, &request.name, request.grade)?;
    let (ca, cert, key) = carry(&dir, &request.name)?;
    registry::register(&deps.state_root, &client, &request.workspace).map_err(|e| e.to_string())?;
    crate::actions::verbs::log_step_done(&deps.state_root, ts, &dir, STEP, Origin::World)
        .map_err(|e| e.to_string())?;
    Ok(Reply::Enrolled(Enrolled {
        grade: request.grade,
        name: request.name.clone(),
        address,
        ca,
        cert,
        key,
    }))
}

/// The address a device will dial (§8), or the refusal naming the remedy.
///
/// `wire/address` is that fact's one home, so it is the one read — and a
/// **`:0` port refuses**. A boot that provisioned this box itself wrote
/// `127.0.0.1:0`, which is a *request* the listener answers with whatever was
/// free (bl-dc14): only the listener knows the number, and it is a different
/// number after the next boot. Putting that in a QR would mint a code that was
/// stale before it was scanned. The remedy is the operator's own statement of
/// intent, which is the act §8 has always named.
///
/// **The remedy is spelled the way it must be typed HERE** (bl-a6b7). Reaching
/// this arm means the material was read, so the re-mint is a *rotation* and the
/// bare command refuses ("already holds material … Re-run with FORCE=1") — a box
/// that can produce this sentence is, by construction, a box the bare remedy
/// turns away. And the address is read at bind time, so the engine standing on
/// the `:0` listener keeps it until it is restarted; an enrollment retried
/// before that would hand a device a port nothing is listening on. Both facts
/// are known at the refusal, and a remedy that cannot succeed is worse than
/// none — it is spent first.
///
/// It reads the **server** end because that is the end a client dials, and
/// reading it as material rather than as a file is what makes a
/// half-provisioned box say so in `material`'s own words.
fn dialable(dir: &Path) -> Result<String, String> {
    let address = material::read_dir(dir, material::Role::Server)?
        .ok_or_else(|| {
            format!(
                "{} holds no wire material: an enrollment issues a leaf under a trust root that \
                 already exists — run `{}` where the CA lives",
                dir.display(),
                material::REMEDY
            )
        })?
        .address;
    if address.rsplit_once(':').map(|(_, port)| port) == Some("0") {
        return Err(format!(
            "{address} names no port a device can dial: a `:0` is a request the listener answers \
             in RAM, and its answer changes at every boot. State the endpoint — \
             `FORCE=1 WIRE_HOST=<host> WIRE_PORT=<port> yog {}` (a rotation, because this box \
             already holds the material its own boot minted) — then restart the engine, which \
             binds the address as it starts",
            provision::verb::SUBCMD
        ));
    }
    Ok(address)
}

/// The three PEMs the device carries away — anchors, its certificate, its key —
/// with the key **shredded** before they are handed back, so this box retains
/// none of it. The shred is unconditional on the reads, which is what makes a
/// failed read leave no key behind either.
///
/// The **certificate stays**, deliberately. It is public material, and its
/// presence on disk is what makes a second enrollment under one name refuse
/// ([`provision::issue`]) — re-issuing distrusts nothing, so two live
/// certificates under one identity would be the result. Keeping it is the
/// guard; keeping the key would be the leak.
fn carry(dir: &Path, name: &str) -> Result<(String, String, String), String> {
    let path = dir.join(format!("{name}.key"));
    // Read, then shred, then judge the read: the two lines are in this order so
    // no failure between them can leave the key behind.
    let key = read(&path);
    shred(&path)?;
    let key = key?;
    Ok((
        read(&dir.join(material::ANCHORS))?,
        read(&dir.join(format!("{name}.pem")))?,
        key,
    ))
}

/// One PEM off disk, or the refusal naming the file it could not read.
fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Remove the minted key, or refuse loudly. A key that could not be shredded is
/// a key still on the box, so it is an error and never a warning — and the
/// sentence names the file, because the remaining act is the operator's.
fn shred(key: &Path) -> Result<(), String> {
    std::fs::remove_file(key).map_err(|e| {
        format!(
            "{}: {e} — the leaf was minted and its key could not be shredded; remove it by hand \
             before that name is trusted",
            key.display()
        )
    })
}

#[cfg(test)]
mod tests;
