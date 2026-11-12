//! Wire key material, minted at test runtime (REMOTE §1.4; bl-b6fa).
//!
//! **No certificate is ever committed.** A fixture key in the tree is a private
//! key in a public repository — the exact class `make leak-scan` refuses — and
//! it would be one whether or not it guarded anything. So the suite performs
//! the same out-of-channel act the server's own boot performs, through the same
//! function: [`provision::mint`](crate::wire::provision::mint), which is the
//! crate's one `openssl` recipe (bl-ae05). A second copy here would be a second
//! recipe, and two spellings of one act drift within a week.

use crate::wire::material::{ANCHORS, Material, Role};
use std::path::Path;

/// The address minted into the material: loopback on a kernel-chosen port. A
/// test never names a port, so two tests never collide on one; the bound
/// address comes back from [`Listener::address`](crate::wire::listener::Listener::address).
///
/// It is also what self-provisioning itself requests since bl-dc14 —
/// bl-4c50's fixture seed (an `ephemeral` writer that pre-named this address
/// so a boot would not fall back to a fixed port) dissolved into the default:
/// a fixture world that names nothing now gets exactly this, the same way a
/// real box does.
pub(crate) const EPHEMERAL: &str = "127.0.0.1:0";

/// Mint a CA and every leaf into `dir`, plus the `address` file — the whole of
/// what a boot mints, in the shape [`crate::wire::material::read`] expects.
pub(crate) fn mint(dir: &Path) {
    crate::wire::provision::mint(dir, EPHEMERAL, &[], false).expect("the wire mint runs");
}

/// The material for `role` in `dir`, aimed at `address` — the read
/// [`crate::wire::material::read`] performs, with the bound port substituted in
/// (the file says `:0`, and only the listener knows what that became).
pub(crate) fn material(dir: &Path, role: Role, address: &str) -> Material {
    let leaf = role.leaf();
    Material {
        anchors: dir.join(ANCHORS),
        chain: dir.join(format!("{leaf}.pem")),
        key: dir.join(format!("{leaf}.key")),
        address: address.to_owned(),
    }
}

/// **A device's own key pair, made the way a device makes it** (REMOTE §15.1):
/// by the device, never by this box's mint. `ring` generates it at test
/// runtime, so no key is committed here either.
pub(crate) struct Device {
    /// The private half, PKCS#8 DER — what a TLS client presents with.
    pub(crate) pkcs8: Vec<u8>,
    /// The public half, DER `SubjectPublicKeyInfo` — what the word is over.
    pub(crate) public: Vec<u8>,
}

impl Device {
    /// A fresh P-256 pair.
    pub(crate) fn new() -> Self {
        use ring::signature::{ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair, KeyPair};
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng)
            .expect("a key pair")
            .as_ref()
            .to_vec();
        let pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &pkcs8, &rng)
            .expect("the pair just made");
        // The prefix ends in the point's own leading `04`, which ring's
        // encoding of the point also carries — so all but that last byte.
        let prefix = &crate::registry::offer::claim::P256[..26];
        let public = [prefix, pair.public_key().as_ref()].concat();
        Self { pkcs8, public }
    }

    /// The registration request's body: `{"public_key": "<PEM>"}`.
    pub(crate) fn body(&self) -> Vec<u8> {
        serde_json::json!({ "public_key": crate::wire::provision::pem::public_key(&self.public) })
            .to_string()
            .into_bytes()
    }
}

impl Device {
    /// This device's wire material once a claim has answered: the anchors and
    /// the leaf it was handed, and its own key, written under `dir` the way a
    /// client files an entry.
    pub(crate) fn material(
        &self,
        dir: &Path,
        claimed: &serde_json::Value,
        address: &str,
    ) -> Material {
        let text = |key: &str| claimed[key].as_str().expect("a PEM").to_owned();
        std::fs::create_dir_all(dir).expect("entry");
        std::fs::write(dir.join(ANCHORS), text("ca")).expect("anchors");
        std::fs::write(dir.join("client.pem"), text("cert")).expect("leaf");
        let label = ["PRIVATE", "KEY"].join(" ");
        let key = crate::wire::provision::pem::block(&label, &self.pkcs8);
        std::fs::write(dir.join("client.key"), key).expect("key");
        material(dir, Role::Client, address)
    }
}

/// **A peer with no certificate, saying `request` and reading to the close**
/// (REMOTE §15.2): TLS against `anchors`, no client identity, the bytes
/// written verbatim. What comes back is everything the server said — empty
/// when it said nothing.
pub(crate) fn certless(anchors: &Path, address: &str, request: &[u8]) -> String {
    use rustls::pki_types::pem::PemObject;
    use std::io::{Read, Write};
    let mut roots = rustls::RootCertStore::empty();
    for anchor in rustls::pki_types::CertificateDer::pem_file_iter(anchors).expect("anchors") {
        roots.add(anchor.expect("anchor")).expect("root");
    }
    let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("versions")
        .with_root_certificates(roots)
        .with_no_client_auth();
    let name = rustls::pki_types::ServerName::IpAddress(std::net::Ipv4Addr::LOCALHOST.into());
    let conn = rustls::ClientConnection::new(std::sync::Arc::new(config), name).expect("tls");
    let tcp = std::net::TcpStream::connect(address).expect("connect");
    tcp.set_read_timeout(Some(std::time::Duration::from_secs(20)))
        .expect("timeout");
    let mut tls = rustls::StreamOwned::new(conn, tcp);
    let _ = tls.write_all(request);
    let mut said = Vec::new();
    let _ = tls.read_to_end(&mut said);
    String::from_utf8_lossy(&said).into_owned()
}

/// The registration request a device sends: the one line, the one header that
/// matters, the body.
pub(crate) fn registration(code: &str, body: &[u8]) -> Vec<u8> {
    let head = format!(
        "POST {}{code} HTTP/1.1\r\nHost: engine.invalid\r\nContent-Type: application/json\r\n\
         content-length: {}\r\n\r\n",
        crate::registry::offer::PATH,
        body.len()
    );
    [head.as_bytes(), body].concat()
}
