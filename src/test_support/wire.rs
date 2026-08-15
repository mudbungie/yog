//! Wire key material, minted at test runtime (REMOTE §1.4; bl-b6fa).
//!
//! **No certificate is ever committed.** A fixture key in the tree is a private
//! key in a public repository — the exact class `make leak-scan` refuses — and
//! it would be one whether or not it guarded anything. So the suite performs
//! the same out-of-channel act an operator performs, with the same tool: a
//! local CA and two leaves, minted by the `openssl` CLI into a scratch
//! directory that dies with the test.
//!
//! P-256 rather than RSA because three keygens run per test that needs a wire,
//! and an EC keygen is a millisecond where an RSA one is a fifth of a second.

use crate::git_env;
use crate::wire::material::{ADDRESS, ANCHORS, Material, Role};
use std::path::Path;

/// The address minted into the material: loopback on a kernel-chosen port. A
/// test never names a port, so two tests never collide on one; the bound
/// address comes back from [`Listener::address`](crate::wire::server::Listener::address).
pub(crate) const EPHEMERAL: &str = "127.0.0.1:0";

/// Mint a CA and a server/client leaf pair into `dir`, plus the `address`
/// file — the whole of what `make wire-certs` mints, in the shape
/// [`crate::wire::material::read`] expects.
pub(crate) fn mint(dir: &Path) {
    let _guard = super::spawn_guard();
    std::fs::create_dir_all(dir).expect("material dir");
    let ca_key = dir.join("ca.key");
    openssl(&[
        "req",
        "-x509",
        "-newkey",
        "ec",
        "-pkeyopt",
        "ec_paramgen_curve:P-256",
        "-nodes",
        "-sha256",
        "-days",
        "1",
        "-subj",
        "/CN=yog-test-ca",
        "-keyout",
        &ca_key.to_string_lossy(),
        "-out",
        &dir.join(ANCHORS).to_string_lossy(),
    ]);
    leaf(dir, &ca_key, Role::Server, "IP:127.0.0.1", "serverAuth");
    leaf(dir, &ca_key, Role::Client, "DNS:yog-client", "clientAuth");
    std::fs::write(dir.join(ADDRESS), EPHEMERAL).expect("address");
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

/// One CA-signed leaf: key, request with its SAN and EKU, then the signature.
fn leaf(dir: &Path, ca_key: &Path, role: Role, san: &str, eku: &str) {
    let leaf = role.leaf();
    let key = dir.join(format!("{leaf}.key"));
    let csr = dir.join(format!("{leaf}.csr"));
    openssl(&[
        "req",
        "-new",
        "-newkey",
        "ec",
        "-pkeyopt",
        "ec_paramgen_curve:P-256",
        "-nodes",
        "-sha256",
        "-subj",
        &format!("/CN=yog-{leaf}"),
        "-addext",
        &format!("subjectAltName={san}"),
        "-addext",
        &format!("extendedKeyUsage={eku}"),
        "-keyout",
        &key.to_string_lossy(),
        "-out",
        &csr.to_string_lossy(),
    ]);
    openssl(&[
        "x509",
        "-req",
        "-sha256",
        "-days",
        "1",
        "-copy_extensions",
        "copy",
        "-in",
        &csr.to_string_lossy(),
        "-CA",
        &dir.join(ANCHORS).to_string_lossy(),
        "-CAkey",
        &ca_key.to_string_lossy(),
        "-out",
        &dir.join(format!("{leaf}.pem")).to_string_lossy(),
    ]);
}

/// One `openssl` invocation, through the crate's one command constructor.
fn openssl(args: &[&str]) {
    let out = git_env::command(Path::new("openssl"))
        .args(args)
        .output()
        .expect("openssl runs");
    assert!(
        out.status.success(),
        "openssl {:?}: {}",
        args.first(),
        String::from_utf8_lossy(&out.stderr)
    );
}
