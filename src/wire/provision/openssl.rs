//! **The `openssl` half of the mint** (REMOTE §1.4, §8; bl-ae05): the tool
//! invocations, and the two facts a certificate carries that are yog's to
//! decide — the subject alternative name and the extended key usage.
//!
//! Split from [`provision`](super) at the seam the 300-line cap and the §12
//! pre-split band both name: **which artifacts a box needs** is a question
//! about the box, and **what one `openssl` run says** is a question about
//! X.509. Nothing here reads the directory's state; nothing there spells a
//! flag.
//!
//! yog links no certificate library (AGENTS.md rule 6) — this shells to the
//! tool an operator would use, through the crate's one command constructor.

use super::{ANCHORS, CA_KEY, CURVE, DAYS, LOOPBACK, private};
use crate::git_env;
use crate::registry::Grade;
use crate::wire::material::Role;
use std::net::IpAddr;
use std::path::Path;

/// The extension file's suffix. Scratch, deleted with the request it rode
/// beside.
const EXT: &str = "ext";
/// The section `-extensions` names inside it. A name rather than the unnamed
/// default section, because "which section" is then stated rather than
/// inferred by two different tools' defaults.
const SECTION: &str = "leaf";
/// The serial counter `-CAcreateserial` derives from the anchor's name — the
/// `openssl` convention, which is why it is spelled here and not beside
/// [`ANCHORS`].
const SERIAL: &str = "ca.srl";

/// The self-signed operator CA both ends verify against.
pub(super) fn ca(dir: &Path) -> Result<(), String> {
    let key = dir.join(CA_KEY);
    tool(&[
        "req",
        "-x509",
        "-newkey",
        "ec",
        "-pkeyopt",
        CURVE,
        "-nodes",
        "-sha256",
        "-days",
        DAYS,
        "-subj",
        "/CN=yog-ca",
        "-keyout",
        &key.to_string_lossy(),
        "-out",
        &dir.join(ANCHORS).to_string_lossy(),
    ])?;
    private(&key, 0o600);
    Ok(())
}

/// One of the three roles' leaves: [`Role`] derives the basename it is written
/// under, the common name it carries — which **is** the client identity the
/// engine reads back off the presented certificate (REMOTE §2) — and the two
/// X.509 facts below. `hosts` is every way in the server answers to; a client
/// leaf ignores it, because nothing dials a client.
pub(super) fn leaf(dir: &Path, role: Role, hosts: &[String]) -> Result<(), String> {
    issue(
        dir,
        &role.leaf(),
        &format!("/CN={}", role.common_name()),
        &san(role, hosts),
        eku(role),
    )
}

/// A client leaf under a **stated** common name (REMOTE §8.2, bl-64a7): the
/// host half of provisioning an entry, and [`Role::Client`]'s own recipe with
/// the operator's name where the role's would be — the client EKU, and a SAN
/// naming the leaf itself, because nothing dials a client. No host is named
/// anywhere in it, which is what lets the pair be carried to whichever box the
/// operator hands it to.
///
/// The pair is written under the common name itself (`<cn>.pem`/`<cn>.key`),
/// because a directory holding several must say which is which. That basename
/// is a filing convenience and nothing more: the name **inside** is the
/// identity (REMOTE §2), and on the client box the pair is placed into
/// `wire/workspaces/<workspace>/` as `client.pem`/`client.key` (§8.2) — that
/// directory named for the workspace it addresses, not for this common name —
/// without changing what it authenticates as.
///
/// **`grade` is the other thing the subject says** (REMOTE §4.2, bl-7ff3), and
/// it is written here because the operator's own CA is the only thing entitled
/// to write it. A foot's subject gains one organizational unit; an operator's
/// is the bare common name it always was, so every leaf minted before the grade
/// existed reads back exactly as it did.
pub(super) fn stated_leaf(dir: &Path, cn: &str, grade: Grade) -> Result<(), String> {
    issue(
        dir,
        cn,
        &subject(cn, grade),
        &format!("DNS:{cn}"),
        eku(Role::Client),
    )
}

/// The subject a stated leaf is minted under. Most-general attribute first,
/// which is DER's own order and the reverse of how RFC 4514 renders it — so
/// `OU=foot` precedes the common name, and the walk that reads the name back
/// (which takes the LAST one) is unaffected either way.
fn subject(cn: &str, grade: Grade) -> String {
    match grade {
        Grade::Operator => format!("/CN={cn}"),
        Grade::Foot => format!("/OU={}/CN={cn}", crate::registry::peer::FOOT),
    }
}

/// The issuance itself: a key, a bare request, then the signature that carries
/// the SAN and EKU it was handed.
///
/// **The extensions are the issuer's, and they are handed over in a file**
/// (bl-8626). The obvious spelling — `req -addext` to put them in the request
/// and `x509 -copy_extensions copy` to carry them across — is OpenSSL-only:
/// macOS ships LibreSSL as `openssl`, whose `x509` has no `-copy_extensions`
/// and refuses the whole invocation (`Unrecognized flag copy_extensions`),
/// which is every wire test on that platform. `-extfile`/`-extensions` is the
/// spelling both toolsets have had for decades, and it is the more honest
/// model besides: what a certificate asserts is decided by whoever signs it,
/// not by whoever asked. One recipe, both toolsets — never a second recipe and
/// never a platform gate.
fn issue(dir: &Path, name: &str, subject: &str, san: &str, eku: &str) -> Result<(), String> {
    let key = dir.join(format!("{name}.key"));
    let csr = dir.join(format!("{name}.csr"));
    let ext = dir.join(format!("{name}.{EXT}"));
    let body = format!("[{SECTION}]\nsubjectAltName={san}\nextendedKeyUsage={eku}\n");
    std::fs::write(&ext, body).map_err(|e| format!("{}: {e}", ext.display()))?;
    tool(&[
        "req",
        "-new",
        "-newkey",
        "ec",
        "-pkeyopt",
        CURVE,
        "-nodes",
        "-sha256",
        "-subj",
        subject,
        "-keyout",
        &key.to_string_lossy(),
        "-out",
        &csr.to_string_lossy(),
    ])?;
    tool(&[
        "x509",
        "-req",
        "-sha256",
        "-days",
        DAYS,
        "-extfile",
        &ext.to_string_lossy(),
        "-extensions",
        SECTION,
        "-in",
        &csr.to_string_lossy(),
        "-CA",
        &dir.join(ANCHORS).to_string_lossy(),
        "-CAkey",
        &dir.join(CA_KEY).to_string_lossy(),
        // LibreSSL's `x509` refuses to sign when the CA's serial file is
        // absent and it was not told it may make one; OpenSSL 3 accepts the
        // flag and does the same thing. Portable, and the file is scratch.
        "-CAcreateserial",
        "-out",
        &dir.join(format!("{name}.pem")).to_string_lossy(),
    ])?;
    // Issuance scratch, not material: the request, the extension file and the
    // serial counter are all inputs to one signature and nothing reads them
    // afterwards, so the directory is left holding exactly what
    // [`artifacts`](super::artifacts) names. Dropping the counter means each
    // leaf is issued under a freshly drawn serial rather than a running one,
    // which is what `-CAcreateserial` writes when it finds no file.
    for scratch in [&csr, &ext, &dir.join(SERIAL)] {
        let _ = std::fs::remove_file(scratch);
    }
    private(&key, 0o600);
    Ok(())
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
/// engine". It is appended, and the whole list is de-duplicated as it is built,
/// so a box stated as loopback says it once.
pub(super) fn san(role: Role, hosts: &[String]) -> String {
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
pub(super) fn eku(role: Role) -> &'static str {
    match role {
        Role::Server => "serverAuth",
        _ => "clientAuth",
    }
}

/// One `openssl` run. The tool is named once, here — [`run`] takes it as a
/// parameter only so a test can drive the two failure paths without
/// uninstalling anything.
pub(super) fn tool(args: &[&str]) -> Result<(), String> {
    run(Path::new("openssl"), args)
}

/// One run of `program`, through the crate's one command constructor
/// (`rules/no-bare-command.yml`). Its stderr is the refusal's text, trimmed:
/// an operator whose mint failed needs the tool's own sentence.
pub(super) fn run(program: &Path, args: &[&str]) -> Result<(), String> {
    let out = git_env::output(git_env::command(program).args(args)).map_err(|e| {
        format!(
            "{}: {e} — the wire's certificates need it",
            program.display()
        )
    })?;
    if out.status.success() {
        return Ok(());
    }
    let said = String::from_utf8_lossy(&out.stderr);
    Err(format!(
        "openssl {}: {}",
        args.first().copied().unwrap_or_default(),
        said.trim()
    ))
}
