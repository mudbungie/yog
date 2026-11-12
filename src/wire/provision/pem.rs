//! **A DER public key, written as PEM** (REMOTE §15.1; bl-9ea6) — the one
//! direction this crate needs and rustls does not provide. A registration
//! hands over a public key as PEM; it is read to DER once, judged as DER, and
//! what `openssl` is then given must be exactly the bytes that were judged.
//! Re-encoding them here is what makes that true: passing the device's own
//! text through would let two PEM readers disagree about which block it holds.
//!
//! Base64 between two labels is not a certificate library (AGENTS.md rule 6).

/// The base64 alphabet (RFC 4648 §4).
const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
/// How many characters a PEM body line carries (RFC 7468).
const LINE: usize = 64;

/// `der` — a `SubjectPublicKeyInfo` — as a `PUBLIC KEY` PEM block.
pub(crate) fn public_key(der: &[u8]) -> String {
    block("PUBLIC KEY", der)
}

/// One PEM block: `der` in base64 between the two `label` lines, the body
/// wrapped at [`LINE`].
pub(crate) fn block(label: &str, der: &[u8]) -> String {
    let body = base64(der);
    let lines: Vec<String> = body
        .as_bytes()
        .chunks(LINE)
        .map(|line| String::from_utf8_lossy(line).into_owned())
        .collect();
    format!(
        "-----BEGIN {label}-----\n{}\n-----END {label}-----\n",
        lines.join("\n")
    )
}

/// Standard base64, padded.
fn base64(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let group = chunk
            .iter()
            .chain([0u8, 0u8].iter())
            .take(3)
            .fold(0u32, |group, byte| (group << 8) | u32::from(*byte));
        for position in 0..4 {
            let index = usize::try_from((group >> (18 - 6 * position)) & 63).unwrap_or_default();
            let symbol = BASE64.get(index).copied().map_or('=', char::from);
            out.push(if position <= chunk.len() { symbol } else { '=' });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{base64, public_key};

    /// RFC 4648 §10's vectors: every padding length, and the empty string.
    #[test]
    fn base64_matches_the_rfc_vectors() {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(plain.as_bytes()), encoded);
        }
    }

    /// The block rustls reads back is the bytes that went in, and a body
    /// longer than one line is wrapped at 64.
    #[test]
    fn a_public_key_round_trips_through_its_pem() {
        use rustls::pki_types::SubjectPublicKeyInfoDer;
        use rustls::pki_types::pem::PemObject;
        let der: Vec<u8> = (0u8..91).collect();
        let pem = public_key(&der);
        assert!(pem.lines().all(|line| line.len() <= 64), "{pem}");
        assert_eq!(pem.lines().count(), 4, "two body lines between two labels");
        let back = SubjectPublicKeyInfoDer::from_pem_slice(pem.as_bytes()).expect("pem");
        assert_eq!(back.as_ref(), der.as_slice());
    }
}
