//! **The two acts over a trust root that already exists** (REMOTE §8, §8.2;
//! bl-64a7, bl-52f4): one extra client leaf under a stated common name, and
//! this box's own server leaf re-issued over more hosts.
//!
//! Split from [`provision`](super) at the seam the module's own vocabulary
//! already draws: **what a box lacks** is a question the mint answers, once,
//! and it may found a CA to answer it; **one more leaf, now** is a question
//! only a box that already holds the CA key can be asked, and answering it
//! founds nothing, writes no address and touches no other leaf. The rotation
//! guard standing in front of a mint would be exactly backwards in front of
//! either of these — both are performed *because* the directory already holds
//! material.

use super::openssl;
use super::{CA_KEY, Role};
use std::path::Path;

/// Issue **one extra client leaf** under a stated common name — the host half
/// of provisioning an entry (REMOTE §8.2, bl-64a7). The operator mints a leaf
/// for a visiting box; the anchors, that leaf and its key are then carried to
/// it by hand, which is §1.4 verbatim and forever. Nothing here is reachable
/// from the channel, and nothing here founds a trust root: this is one more
/// artifact the one recipe can be asked for, over a CA that already exists.
///
/// It refuses three ways, each naming its remedy:
///
/// - **An identity the registry would refuse.** The same rule, spent once
///   ([`Client::parse`](crate::registry::Client::parse)): a common name is one
///   path component and `local` is reserved for the certificate-less in-world
///   callers (§4.1). A name that could carry a separator is a name that could
///   address the filesystem — here, and again on the box that files the pair.
/// - **No CA key.** A box holding an operator's `ca.pem` with no key beside it
///   is a *client* machine, and the mint never replaces an operator's trust
///   root (§8). `ca.key`'s presence is exactly the question "can this box
///   mint?", so it is exactly the question asked here.
/// - **A pair already under that name.** Re-issuing distrusts nothing — the
///   certificate already carried away stays valid until the CA behind it is
///   rotated — so it would put two live certificates under one identity for no
///   gain. A fresh common name is the remedy; rotating the trust root stays the
///   verb's `FORCE` over the whole directory.
///
/// **`grade` is minted into the subject** (REMOTE §4.2, bl-7ff3). Making a foot
/// is minting a certificate for it, out of channel, on the operator's own CA —
/// which is exactly the friction the out-of-channel ruling wants, and the
/// reason the grade is not a registration field a gesture over the wire could
/// widen, nor a config file on the box being trusted.
pub(crate) fn issue(dir: &Path, cn: &str, grade: crate::registry::Grade) -> Result<(), String> {
    crate::registry::Client::parse(cn).map_err(|refusal| {
        format!(
            "{refusal} — a common name is one path component, and {:?} is reserved for the              in-world callers; state another one",
            crate::registry::LOCAL
        )
    })?;
    founded(dir)?;
    let pair = [format!("{cn}.pem"), format!("{cn}.key")];
    if pair.iter().any(|name| dir.join(name).is_file()) {
        // The operator most likely to hit this is one act past done: they
        // enrolled a device as a seat and are now enrolling its "tool side",
        // which the first leaf already serves (REMOTE §5 — one identity, two
        // connections). The refusal teaches that, because without it the
        // block reads as arbitrary (bl-7a4a).
        return Err(format!(
            "{} already holds {}: re-issuing distrusts nothing, so both would be live under one \
             identity. One device is one name and one leaf, whatever its grade — an \
             operator-grade leaf already serves the seat AND the tool host, so a device that \
             chats and tools enrolls once; a foot leaf is for a tools-only device under its own \
             name. State another common name, or rotate the whole directory with FORCE=1",
            dir.display(),
            pair.join(" or ")
        ));
    }
    openssl::stated_leaf(dir, cn, grade)
}

/// Re-issue **this box's own server leaf** over the CA already here (REMOTE §8,
/// bl-52f4), covering every host stated. It is the same kind of act as
/// [`issue`] and carries the same guard: a trust root that already exists, no
/// CA founded, no address written, no other leaf touched.
///
/// **It is not a rotation, and that is the whole point.** A client verifies the
/// CA, so the one artifact whose replacement strands nobody is the server's own
/// leaf — while `FORCE=1` re-founds the CA and distrusts every leaf already
/// carried to another box. Widening what a certificate covers used to cost that
/// fleet; here it costs one signature.
///
/// The pair is removed before it is re-minted so a failure leaves NO leaf
/// rather than a new key beside an old certificate: absence is a state
/// [`ensure`](super::ensure) heals on the next boot, and a mismatched pair is a handshake that
/// fails for a reason nothing can read.
pub(crate) fn reissue(dir: &Path, hosts: &[String]) -> Result<(), String> {
    founded(dir)?;
    let leaf = Role::Server.leaf();
    for name in [format!("{leaf}.pem"), format!("{leaf}.key")] {
        let _ = std::fs::remove_file(dir.join(name));
    }
    openssl::leaf(dir, Role::Server, hosts)
}

/// The one guard in front of every act over a trust root that already exists
/// ([`issue`], [`reissue`]). A box holding an operator's `ca.pem` with no key
/// beside it is a *client* machine, and the mint never replaces an operator's
/// trust root (§8) — so `ca.key`'s presence is exactly the question "can this
/// box issue?", asked once for both acts.
fn founded(dir: &Path) -> Result<(), String> {
    if dir.join(CA_KEY).is_file() {
        return Ok(());
    }
    Err(format!(
        "{} holds no {CA_KEY}: only the box that founded this trust root can issue under it \
         — run this where the CA lives",
        dir.display()
    ))
}
