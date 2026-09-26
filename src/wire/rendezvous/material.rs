//! The rendezvous material (REMOTE §13.2, bl-4263): two files the mint grows
//! beside `ca.pem`, and everything both ends derive from them.
//!
//! **Minted out of channel, like every other fact in `wire/`** (REMOTE §1.4).
//! [`mint`] writes a 32-byte ed25519 seed and a 32-byte pairing salt, hex, at
//! the same moment [`provision`](crate::wire::provision) mints the CA — and
//! only for a box whose `address` is not loopback, because a box only a local
//! seat dials has nothing to rendezvous for and REMOTE §13.4's severability says it
//! starts no thread. The public key and the salt cross inside a client's
//! entry exactly as `ca.pem` does.
//!
//! **One salt, four derivations, all HKDF** — so the commons stores nothing
//! that names the pairing: the DHT salt each item is filed under, the
//! keypair the inbox is signed with (derived, so a client needs no keypair of
//! its own to write it and the engine needs no client key to poll it), and
//! the AEAD key both items are sealed under. Deriving the inbox key from the
//! pairing salt is what "a key derived from the pairing salt" means literally,
//! and it dissolves a per-client key the engine would otherwise have to hold.

use crate::dht::Keypair;
use ring::hkdf::{HKDF_SHA256, Salt};
use std::path::Path;

/// The ed25519 seed the engine publishes presence under, hex.
pub const KEY: &str = "rendezvous.key";
/// The pairing salt, hex — the secret a client and this engine share.
pub const SALT: &str = "pairing.salt";

/// What the two files hold, decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pairing {
    pub(crate) seed: [u8; 32],
    pub(crate) salt: [u8; 32],
}

impl Pairing {
    /// The engine's rendezvous keypair — presence is signed under it.
    pub(crate) fn keypair(&self) -> Result<Keypair, String> {
        Keypair::from_seed(self.seed)
    }

    /// The keypair the inbox is signed with, which both ends can derive.
    pub(crate) fn inbox_keypair(&self) -> Result<Keypair, String> {
        Keypair::from_seed(self.derive(b"inbox key"))
    }

    /// The DHT salt the presence item is filed under.
    pub(crate) fn presence_salt(&self) -> Vec<u8> {
        self.derive(b"presence salt").to_vec()
    }

    /// The DHT salt the inbox item is filed under.
    pub(crate) fn inbox_salt(&self) -> Vec<u8> {
        self.derive(b"inbox salt").to_vec()
    }

    /// The AEAD key both items are sealed under.
    pub(crate) fn seal_key(&self) -> [u8; 32] {
        self.derive(b"seal key")
    }

    /// HKDF-SHA256 over the pairing salt, one label per derived fact.
    fn derive(&self, label: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        // HKDF-SHA256 expands to exactly 32 bytes for a 32-byte buffer, so
        // neither step can refuse; the fallback is unreachable and named.
        if let Ok(okm) = Salt::new(HKDF_SHA256, b"yog rendezvous")
            .extract(&self.salt)
            .expand(&[label], HKDF_SHA256)
        {
            let _ = okm.fill(&mut out);
        }
        out
    }
}

/// Read the pairing out of `dir`: `Ok(None)` is nothing minted (no thread),
/// `Err` is half of it, which is the same misconfiguration a half-provisioned
/// wire is and earns the same remedy.
pub(crate) fn read_dir(dir: &Path) -> Result<Option<Pairing>, String> {
    let (seed, salt) = (hex_file(&dir.join(KEY)), hex_file(&dir.join(SALT)));
    match (seed, salt) {
        (None, None) => Ok(None),
        (Some(seed), Some(salt)) => Ok(Some(Pairing { seed, salt })),
        _ => Err(format!(
            "the rendezvous material at {} is half there: {KEY} and {SALT} are minted \
             together — run `{}`",
            dir.display(),
            super::super::material::REMEDY
        )),
    }
}

/// Mint whatever of the two `dir` lacks. Idempotent like the rest of the mint:
/// a file already there is never rewritten, because a re-keyed engine is an
/// engine no client can find.
pub(crate) fn mint(dir: &Path) -> Result<(), String> {
    for name in [KEY, SALT] {
        let path = dir.join(name);
        if path.is_file() {
            continue;
        }
        let mut bytes = [0u8; 32];
        crate::dht::random(&mut bytes)?;
        std::fs::write(&path, format!("{}\n", hex(&bytes)))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        // The mint's own narrowing (`provision::private`): a seed the rest of
        // the box can read is the disclosure the wire exists to prevent.
        super::super::provision::private(&path, 0o600);
    }
    Ok(())
}

/// The two files, for the rotation's delete list.
pub(crate) fn artifacts() -> Vec<String> {
    vec![KEY.to_owned(), SALT.to_owned()]
}

/// Exactly 32 bytes of hex in `path`, or nothing — a file that will not read
/// or does not decode is no material, the same as an absent one.
fn hex_file(path: &Path) -> Option<[u8; 32]> {
    let text = std::fs::read_to_string(path).ok()?;
    let bytes = unhex(text.trim())?;
    <[u8; 32]>::try_from(bytes).ok()
}

/// Lowercase hex.
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// The inverse of [`hex`]; `None` on any byte that is not one.
pub(crate) fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    text.as_bytes()
        .chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests;
