//! **What a certificate says** (REMOTE §4.2, §8; bl-ae05, bl-7ff3): the three
//! X.509 facts that are yog's to decide — the subject, the subject alternative
//! name and the extended key usage. Split from [`openssl`](super) at §12's cap
//! on the seam its own doc draws: everything there is an invocation, and
//! nothing here runs a tool.

use crate::registry::Grade;
use crate::wire::material::Role;
use crate::wire::provision::LOOPBACK;
use std::net::IpAddr;

/// The subject a stated leaf is minted under. Most-general attribute first,
/// which is DER's own order and the reverse of how RFC 4514 renders it — so
/// `OU=foot` precedes the common name, and the walk that reads the name back
/// (which takes the LAST one) is unaffected either way.
pub(super) fn subject(cn: &str, grade: Grade) -> String {
    match grade {
        Grade::Operator => format!("/CN={cn}"),
        Grade::Foot => format!("/OU={}/CN={cn}", crate::registry::peer::FOOT),
    }
}

/// A leaf's subject alternative name. The **server**'s names every way in the
/// box answers to, because a seat verifies what it dialled against this list —
/// an IP literal is an IP identity and anything else a DNS one, the same rule
/// [`client`](super::client) reads it back by, applied to each entry alike. A
/// client leaf's names itself: nothing dials a client.
///
/// **A box is reachable more than one way, and saying so is not a rotation**
/// (bl-52f4). A host on an overlay network has a resolvable name, an overlay
/// address and a LAN address, and different clients reach it differently — a
/// device whose resolver is its emulator's cannot use the name, and an address
/// the certificate omits fails *verification* rather than routing, so the error
/// names trust where the fact is reachability. One host per certificate made
/// the remedy `FORCE=1`, which re-founds the CA and strands every client leaf
/// already carried away. The list is the dissolution: state every spelling
/// once.
///
/// **Loopback is always on the server leaf** (bl-ae05). The local window is a
/// client of `127.0.0.1` unconditionally — that is what the ruling means by the
/// front door — so a server certificate that only named an operator's public
/// host would refuse the one seat that is certain to be there. It costs one
/// SAN entry and removes a whole class of "the window cannot reach its own
/// server". It is appended, and the whole list is de-duplicated as it is built,
/// so a box stated as loopback says it once.
pub(in crate::wire::provision) fn san(role: Role, hosts: &[String]) -> String {
    match role {
        Role::Server => {
            let mut names: Vec<String> = Vec::new();
            for host in hosts.iter().map(String::as_str).chain([LOOPBACK]) {
                let entry = if host.parse::<IpAddr>().is_ok() {
                    format!("IP:{host}")
                } else {
                    format!("DNS:{host}")
                };
                if !names.contains(&entry) {
                    names.push(entry);
                }
            }
            names.join(",")
        }
        _ => format!("DNS:{}", role.common_name()),
    }
}

/// A leaf's extended key usage: the server end authenticates as a server, and
/// both client ends as clients.
pub(in crate::wire::provision) fn eku(role: Role) -> &'static str {
    match role {
        Role::Server => "serverAuth",
        _ => "clientAuth",
    }
}
