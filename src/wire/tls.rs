//! The mTLS wrapper (REMOTE §1.3, §4; bl-b6fa): rustls configurations built
//! from [`Material`], one for each end.
//!
//! **Both ends authenticate with certificates, and that is the entire
//! authentication story.** The server requires a client certificate that chains
//! to the operator CA; the client requires the same of the server and presents
//! its own. There is no password, token or account anywhere in the channel — so
//! there is nothing in it to phish, rotate or leak, and a connection with no
//! certificate gets **no yog reply**: no byte of the boundary is ever reached,
//! and the one thing it may say is the registration door's (REMOTE §15.2).
//!
//! **The provider is named, never defaulted.** `ServerConfig::builder()` reads a
//! process-global default and *panics* when none is installed or two are — a
//! panic path, which prod does not have (AGENTS.md rule 4). Naming `ring`
//! outright removes the global read and the panic with it, and it is the
//! provider already in the graph behind `ureq`, so nothing new links.

use super::material::Material;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::path::Path;
use std::sync::Arc;

/// The server's end: verify a client certificate against the operator CA, and
/// present the server leaf. A leaf the CA did not issue never completes the
/// handshake.
///
/// **A connection presenting nothing does complete it** (REMOTE §15.2,
/// bl-9ea6), and is then not the wire: the listener asks what the peer
/// presented before it reads a byte, and a peer with no certificate is offered
/// the registration door's one request and nothing else. Verification is
/// optional here so that there is one port and one listener; what a certless
/// peer may reach is decided there, not here.
pub fn server_config(m: &Material) -> Result<Arc<ServerConfig>, String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(anchors(&m.anchors)?),
        Arc::clone(&provider),
    )
    .allow_unauthenticated()
    .build()
    .map_err(|e| format!("{}: client verifier: {e}", m.anchors.display()))?;
    let (chain, key) = identity(&m.chain, &m.key)?;
    let config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("tls versions: {e}"))?
        .with_client_cert_verifier(verifier)
        .with_single_cert(chain, key)
        .map_err(|e| format!("{}: server identity: {e}", m.chain.display()))?;
    Ok(Arc::new(config))
}

/// A seat's end: verify the server against the same operator CA, and present
/// the client leaf — the certificate that *is* this client's identity.
pub fn client_config(m: &Material) -> Result<Arc<ClientConfig>, String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let (chain, key) = identity(&m.chain, &m.key)?;
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("tls versions: {e}"))?
        .with_root_certificates(anchors(&m.anchors)?)
        .with_client_auth_cert(chain, key)
        .map_err(|e| format!("{}: client identity: {e}", m.chain.display()))?;
    Ok(Arc::new(config))
}

/// The operator CA as a trust anchor store. Every certificate in the file is
/// an anchor: an operator who put two in meant two.
fn anchors(path: &Path) -> Result<RootCertStore, String> {
    let mut store = RootCertStore::empty();
    for anchor in
        CertificateDer::pem_file_iter(path).map_err(|e| format!("{}: {e}", path.display()))?
    {
        let anchor = anchor.map_err(|e| format!("{}: {e}", path.display()))?;
        store
            .add(anchor)
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if store.is_empty() {
        return Err(format!("{}: no certificate in it", path.display()));
    }
    Ok(store)
}

/// One end's chain and key, read from PEM.
fn identity(
    chain: &Path,
    key: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), String> {
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(chain)
        .map_err(|e| format!("{}: {e}", chain.display()))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{}: {e}", chain.display()))?;
    if certs.is_empty() {
        return Err(format!("{}: no certificate in it", chain.display()));
    }
    let private =
        PrivateKeyDer::from_pem_file(key).map_err(|e| format!("{}: {e}", key.display()))?;
    Ok((certs, private))
}

#[cfg(test)]
mod tests;
