//! **A certificate leaf name** (REMOTE §2, bl-8bbc): the client identity read
//! off the certificate the peer presented, and nothing else.
//!
//! §2 is exact about what a client is — *"One certificate = one client identity
//! (its leaf name)"* — so the identity is the leaf's subject **common name**,
//! which is what `make wire-certs` mints (`/CN=yog-client`) and what an
//! operator types when they seat a registration by hand. A fingerprint would
//! have been cheaper to compute and wrong on both counts: it is unreadable in a
//! `clients/` listing, and it changes on renewal, silently de-scoping every
//! registration the operator wrote.
//!
//! **yog links no certificate library, so this is a DER walk** (AGENTS.md rule
//! 6: zero new dependencies). It is ~60 lines of structural ASN.1 rather than a
//! byte search, and the structure is the point: the **issuer** carries a common
//! name too, and it comes FIRST — a scan for the CN object identifier would
//! return the operator CA's name for every client on the box.
//!
//! What it reads, per RFC 5280:
//!
//! ```text
//! Certificate     ::= SEQUENCE { tbsCertificate, signatureAlgorithm, signature }
//! TBSCertificate  ::= SEQUENCE { [0] version OPTIONAL, serialNumber INTEGER,
//!                                signature, issuer, validity, subject, … }
//! Name            ::= SEQUENCE OF SET OF SEQUENCE { type OID, value ANY }
//! ```
//!
//! The optional `[0] version` is why `subject` is located **relative to the
//! serial number** rather than at a fixed index: the serial is the first field
//! certainly present, and `subject` is four constructed values past it. A
//! version-1 certificate and a version-3 one then take one path, not two.

/// DER tags this walk names.
const INTEGER: u8 = 0x02;
const OID: u8 = 0x06;
/// `id-at-commonName` — ASN.1 `{joint-iso-itu-t(2) ds(5) attributeType(4)
/// commonName(3)}`, in its DER encoding. Spelled as bytes rather than as the
/// dotted arc string because the dotted form of four small arcs is
/// indistinguishable from an IPv4 address, to a reader and to `make leak-scan`.
const COMMON_NAME: [u8; 3] = [0x55, 0x04, 0x03];
/// How many constructed fields separate `serialNumber` from `subject`:
/// signature, issuer, validity, subject.
const SERIAL_TO_SUBJECT: usize = 4;

/// The subject common name of a DER-encoded certificate, or `None` when the
/// bytes are not a certificate or carry no readable one.
///
/// The **last** common name wins. A distinguished name is written most-general
/// first in DER and most-specific last (RFC 4514 renders it reversed), so the
/// final `CN` is the leaf's own; a certificate minted by `make wire-certs` has
/// exactly one and the question does not arise.
pub fn common_name(der: &[u8]) -> Option<String> {
    let (_, certificate, _) = tlv(der)?;
    let (_, tbs, _) = tlv(certificate)?;
    let fields = elements(tbs);
    let serial = fields.iter().position(|(tag, _)| *tag == INTEGER)?;
    let &(_, subject) = fields.get(serial + SERIAL_TO_SUBJECT)?;
    last_common_name(subject)
}

/// The last `CN` attribute value in a `Name`, decoded as UTF-8. Every string
/// type a CN is minted in — `UTF8String`, `PrintableString`, `IA5String` — is
/// UTF-8 or a subset of it, and one that is not (`BMPString` is UTF-16) fails
/// the decode and is skipped rather than mis-read.
fn last_common_name(name: &[u8]) -> Option<String> {
    let mut found: Option<String> = None;
    for (_, rdn) in elements(name) {
        for (_, attribute) in elements(rdn) {
            let parts = elements(attribute);
            let (Some(&(tag, oid)), Some(&(_, value))) = (parts.first(), parts.get(1)) else {
                continue;
            };
            if tag != OID || oid != COMMON_NAME {
                continue;
            }
            if let Ok(text) = std::str::from_utf8(value) {
                found = Some(text.to_owned());
            }
        }
    }
    found
}

/// One DER type-length-value off the front of `bytes`: its tag, its contents,
/// and what follows it. `None` for a truncated header, a truncated value, or a
/// length DER does not permit — the indefinite form (`0x80`), which BER allows
/// and DER forbids, and a length wider than this walk will serve.
fn tlv(bytes: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let tag = *bytes.first()?;
    let first = *bytes.get(1)?;
    let (len, header) = if first < 0x80 {
        (usize::from(first), 2)
    } else {
        let width = usize::from(first & 0x7f);
        if width == 0 || width > 4 {
            return None;
        }
        let mut len: usize = 0;
        for i in 0..width {
            len = (len << 8) | usize::from(*bytes.get(2 + i)?);
        }
        (len, 2 + width)
    };
    // Saturating rather than checked: an unreachable overflow arm is an
    // untestable branch, and a saturated end simply fails the read below.
    let end = header.saturating_add(len);
    let value = bytes.get(header..end)?;
    Some((tag, value, bytes.get(end..).unwrap_or_default()))
}

/// Every element of a constructed value, in order. A trailing byte run that is
/// not a whole TLV ends the walk — a malformed tail yields the elements read
/// before it, which is what makes every read above total.
fn elements(mut body: &[u8]) -> Vec<(u8, &[u8])> {
    let mut out = Vec::new();
    while let Some((tag, value, rest)) = tlv(body) {
        out.push((tag, value));
        body = rest;
    }
    out
}

#[cfg(test)]
mod tests;
