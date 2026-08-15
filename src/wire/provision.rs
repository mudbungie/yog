//! **The mint** (REMOTE §1.4, §8 as amended; bl-ae05): the one recipe that
//! writes a local CA and this box's leaves, and the one place `openssl` is
//! spelled.
//!
//! **It is still out-of-channel, and that is the whole of the ruling.** REMOTE
//! §1.4 forbids an *in-channel* bootstrap — no enrollment, pairing or account
//! protocol, ever — and §8 already blessed this exact act as operator tooling
//! (`make wire-certs`, shelling to `openssl`). The operator ruling 2026-08-14
//! moved the trigger and nothing else: the engine's own boot performs the act
//! on the operator's own box, before anything has been dialled, so no byte
//! crosses a wire unauthenticated and nothing about the channel changed. yog
//! still links no certificate library (AGENTS.md rule 6: no `rcgen`, no new
//! dependency) — it shells to the same tool an operator would.
//!
//! **Why boot mints at all.** REMOTE §1.2 rules the window a client of the
//! boundary over the real wire. A window on an unprovisioned box would then
//! have no read path — a window that paints nothing — and §8 has already
//! rejected the two ways around that: refusing with a remedy *"puts a terminal
//! instruction in front of a desktop launch that has no terminal"*, and leaving
//! the letter of §1.2 aspirational was the option the ruling declined. So the
//! absence of material stopped being the off switch for the **local** listener:
//! a box with nothing provisioned founds its own loopback trust root and serves
//! itself.
//!
//! **What still distinguishes wider listening is the address, and nothing
//! else.** Self-provisioning writes `127.0.0.1:<port>` — loopback only, which
//! is the safe default: material yog minted itself grants exactly the window
//! that minted it. An operator who wants a seat on another machine performs the
//! explicit act (`yog wire-certs WIRE_HOST=…`, [`verb`]), and *that* address is
//! the operator's statement of intent. One fact with one home (§8), no second
//! knob, and no flag deciding how far a listener reaches.
//!
//! **Nothing complete is ever overwritten.** Every step here asks whether its
//! artifact is already there, so a second call mints nothing — which is what
//! makes it safe on every boot. A rotation distrusts every certificate already
//! issued, so it stays the operator's deliberate `FORCE=1` ([`verb`]).

use super::material::{ADDRESS, ANCHORS, LEAVES, Role};
use std::path::Path;

/// The `openssl` invocations and the two X.509 facts they carry.
mod openssl;

/// The CA's private key. [`material`](super::material) never names it: it is
/// what issues the *next* leaf, and nothing but issuance reads it — so its
/// presence is exactly the question "can this box mint?"
pub const CA_KEY: &str = "ca.key";
/// The host self-provisioning binds and writes into `address`.
pub const LOOPBACK: &str = "127.0.0.1";
/// The port self-provisioning binds. A default, not a knob — `address` is the
/// one home of what is bound, and an operator who wants another edits it.
pub const PORT: &str = "7737";
/// How long a minted certificate is good for.
pub(super) const DAYS: &str = "825";
/// P-256 rather than RSA: a mint runs on a desktop launch, and four EC keygens
/// are milliseconds where four RSA ones are a second.
pub(super) const CURVE: &str = "ec_paramgen_curve:P-256";

/// Mint whatever `dir` is missing, taking the address it already names — the
/// boot's call, and idempotent by construction. A box provisioned by an
/// operator gains only the leaves it lacks; a box with nothing gains the lot,
/// aimed at loopback.
pub fn ensure(dir: &Path) -> Result<(), String> {
    let address = address_at(dir).unwrap_or_else(|| format!("{LOOPBACK}:{PORT}"));
    mint(dir, &address, false)
}

/// [`ensure`] against a stated address, optionally rotating. `force` deletes
/// every artifact first, which is what makes a rotation a rotation: the CA that
/// issued the old leaves is gone, so nothing holding one connects again.
pub fn mint(dir: &Path, address: &str, force: bool) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    private(dir, 0o700);
    if force {
        for name in artifacts() {
            let _ = std::fs::remove_file(dir.join(name));
        }
    }
    let ca_key = dir.join(CA_KEY);
    // A CA is minted only when BOTH halves are absent. A box holding `ca.pem`
    // and no key is a *client* machine — the operator copied the anchor and a
    // leaf onto it — and re-minting there would replace the operator's trust
    // root with one that verifies nothing they issued.
    if !ca_key.is_file() && !dir.join(ANCHORS).is_file() {
        openssl::ca(dir)?;
    }
    if ca_key.is_file() {
        let host = host_of(address);
        for role in LEAVES {
            if !leaf_present(dir, role) {
                openssl::leaf(dir, role, &host)?;
            }
        }
    }
    if address_at(dir).is_none() {
        std::fs::write(dir.join(ADDRESS), format!("{address}\n"))
            .map_err(|e| format!("{}: {e}", dir.join(ADDRESS).display()))?;
    }
    Ok(())
}

/// Every file the mint writes — the rotation's delete list, and the summary a
/// caller prints.
pub fn artifacts() -> Vec<String> {
    let mut names = vec![ANCHORS.to_owned(), CA_KEY.to_owned(), ADDRESS.to_owned()];
    for role in LEAVES {
        let leaf = role.leaf();
        names.push(format!("{leaf}.pem"));
        names.push(format!("{leaf}.key"));
    }
    names
}

/// The address `dir` already names, if it names one — the same read
/// [`material::read`](super::material::read) performs, so a half-written or
/// empty file is no address here either.
fn address_at(dir: &Path) -> Option<String> {
    let text = std::fs::read_to_string(dir.join(ADDRESS)).ok()?;
    let text = text.trim().to_owned();
    (!text.is_empty()).then_some(text)
}

/// The host half of a `host:port`, unbracketed — what a SAN is derived from.
fn host_of(address: &str) -> String {
    let host = address.rsplit_once(':').map_or(address, |(head, _)| head);
    host.trim_start_matches('[')
        .trim_end_matches(']')
        .to_owned()
}

/// Whether this role's leaf is present in full. Half a leaf is no leaf: the
/// pair is minted together and is useless apart.
fn leaf_present(dir: &Path, role: Role) -> bool {
    let leaf = role.leaf();
    dir.join(format!("{leaf}.pem")).is_file() && dir.join(format!("{leaf}.key")).is_file()
}

/// Narrow a path to its owner. A private key the rest of the box can read is
/// the disclosure this whole channel exists to prevent, and `openssl` writes
/// through the ambient umask. Unix-only because the mode bits are; elsewhere
/// the directory's own inheritance is what there is.
#[cfg(unix)]
fn private(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn private(_path: &Path, _mode: u32) {}

/// The `yog wire-certs` verb — the operator's explicit act over [`mint`].
pub mod verb;

#[cfg(test)]
mod tests;
