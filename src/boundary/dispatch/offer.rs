//! **The offer's executor** (REMOTE §15.1; bl-9ea6) — beside
//! [`advertise`](super::advertise) and for its reason: everything else in the
//! chokepoint routes, and these *gate*.
//!
//! **The grade gate is spent one layer up** (REMOTE §4.2): none of the four is
//! in the set a foot may say, so `Grade::admits` refuses them in band before
//! this file is reached, and nothing here asks what grade the caller is.
//!
//! **What this file decides is which name may be offered, and which offer may
//! be approved.** Both questions exist because a certificate cannot be
//! recalled: the operator's CA has no revocation, so a leaf once issued is
//! good for the CA's life, and the only thing standing between it and a
//! workspace is that its *name* is seated nowhere.
//!
//! - A name is **offerable** only while nothing has ever been issued under it:
//!   no leaf on disk, no registration, no claim held, and none of this box's
//!   own seats. An offer for a name already registered would hand its claimant
//!   that registration with no approval at all.
//! - An offer is **approvable** only once proved — the holder of the private
//!   key has spoken on the wire, which a copy of the URL never could.

use std::path::Path;

use crate::opslog::Origin;
use crate::registry::offer::store::{self, Record};
use crate::registry::offer::{self, Answer, Row, State, Verb};
use crate::registry::{self, Client, Grade};
use crate::wire::{material, provision};

use super::Deps;

/// One of the four acts. `ts` is the unix stamp every boundary derivation
/// dates against.
pub(super) fn dispatch(deps: &Deps, ts: &str, verb: &Verb) -> Result<Answer, String> {
    let root = deps.state_root.as_path();
    let now: i64 = ts.parse().unwrap_or(0);
    match verb {
        Verb::Mint {
            workspace,
            name,
            grade,
        } => mint(deps, ts, now, workspace, name, *grade),
        Verb::List { workspace } => Ok(Answer::Offers(
            store::pending(root, now)
                .iter()
                .filter(|record| record.workspace == *workspace)
                .map(|record| row(root, record))
                .collect(),
        )),
        Verb::Approve { workspace, name } => {
            let record = find(root, now, workspace, name)?;
            if standing(root, &record) != State::Proved {
                return Err(format!(
                    "{name} is not proved: its device has not yet dialled with the certificate \
                     the offer issued, so nothing shows that whoever claimed the URL holds the \
                     key. Approve it once /offers shows it proved and its word matches the one \
                     on the device"
                ));
            }
            registry::register(root, &Client::parse(name)?, workspace)
                .map_err(|e| e.to_string())?;
            store::remove(root, name);
            log(deps, ts, "approve").map(|()| Answer::Approved)
        }
        Verb::Decline { workspace, name } => {
            let record = find(root, now, workspace, name)?;
            if record.claimed.is_some() {
                // The leaf it issued cannot be recalled, so the hold stays and
                // the name stays spent: a later offer under it, approved,
                // would seat this certificate's holder too.
                let declined = Record {
                    declined: true,
                    ..record
                };
                store::settle(root, &declined)?;
            } else {
                store::remove(root, name);
            }
            log(deps, ts, "decline").map(|()| Answer::Declined)
        }
    }
}

/// Mint an offer, refusing a name that is not offerable and a box that could
/// not honour the claim — every refusal ahead of the code, so no code exists
/// for an offer that could not be claimed.
fn mint(
    deps: &Deps,
    ts: &str,
    now: i64,
    workspace: &str,
    name: &str,
    grade: Grade,
) -> Result<Answer, String> {
    let root = deps.state_root.as_path();
    let dir = material::dir(&deps.world);
    let client = Client::parse(name)?;
    offerable(root, &dir, &client)?;
    let host = host(&dir)?;
    let code =
        offer::code().ok_or("the system's random source answered nothing: no code was minted")?;
    let expires = now + offer::LIFE;
    store::mint(
        root,
        &Record {
            name: name.to_owned(),
            workspace: workspace.to_owned(),
            grade,
            code: code.clone(),
            expires,
            word: None,
            claimed: None,
            declined: false,
        },
    )?;
    log(deps, ts, "offer")?;
    Ok(Answer::Offered {
        url: format!("https://{host}{}{code}", offer::PATH),
        expires,
    })
}

/// Whether a leaf may be issued under `client`'s name by a claim nobody has
/// approved yet — see the module doc for why each of the four refuses.
fn offerable(root: &Path, dir: &Path, client: &Client) -> Result<(), String> {
    let name = client.name();
    provision::admissible(dir, &name)?;
    let own = material::LEAVES
        .iter()
        .any(|role| role.common_name() == name);
    if own || !registry::registered(root, client).is_empty() || store::held(root, client) {
        return Err(format!(
            "{name} cannot be offered: it is this box's own seat, or it is registered already, \
             or an earlier offer under it was claimed — and a certificate cannot be recalled, \
             so whoever holds one under that name would be seated by this offer's approval. \
             State another common name"
        ));
    }
    Ok(())
}

/// The host a registration URL is built from: the first the server states
/// (REMOTE §15.3). A `:0` address is a request only the listener learns the
/// answer to, so it names no host a device could reach and refuses.
fn host(dir: &Path) -> Result<String, String> {
    material::addresses(dir)
        .first()
        .and_then(|address| address.rsplit_once(':'))
        .filter(|(host, port)| !host.is_empty() && *port != "0")
        .map(|(host, _)| host.to_owned())
        .ok_or_else(|| {
            format!(
                "{} names no host a device could reach: state where this server listens — \
                 `WIRE_HOST=<host> WIRE_PORT=<port> yog {}`, then restart it",
                dir.join(material::ADDRESS).display(),
                provision::verb::SUBCMD
            )
        })
}

/// The pending offer for `name` in `workspace`, or the refusal. An offer in
/// another workspace is the same answer as none: scope hides it.
fn find(root: &Path, now: i64, workspace: &str, name: &str) -> Result<Record, String> {
    store::pending(root, now)
        .into_iter()
        .find(|record| record.name == name && record.workspace == workspace)
        .ok_or_else(|| format!("no pending offer for {name} in {workspace}"))
}

/// Where an offer stands. **`proved` is derived, never stored**: it is a
/// claimed offer whose client has been seen on the wire since the claim, and
/// the registry already stamps that at the one door every connection's bytes
/// pass through (`registry::seen`).
fn standing(root: &Path, record: &Record) -> State {
    let Some(claimed) = record.claimed else {
        return State::Open;
    };
    let spoke = Client::parse(&record.name)
        .ok()
        .and_then(|client| registry::seen::read(root, &client));
    if spoke.is_some_and(|at| at >= claimed) {
        State::Proved
    } else {
        State::Claimed
    }
}

fn row(root: &Path, record: &Record) -> Row {
    Row {
        name: record.name.clone(),
        grade: record.grade,
        state: standing(root, record),
        expires: record.expires,
        word: record.word.clone(),
    }
}

/// The trail's row for one act (§4.2): that it happened here, and who asked.
fn log(deps: &Deps, ts: &str, step: &str) -> Result<(), String> {
    crate::actions::verbs::log_step_done(
        &deps.state_root,
        ts,
        &material::dir(&deps.world),
        step,
        Origin::World,
        deps.caller.client.clone(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
