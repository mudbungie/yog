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

use super::{ANCHORS, CA_KEY, CURVE, DAYS, private};
use crate::git_env;
use crate::registry::Grade;
use crate::wire::material::Role;
use std::path::Path;

/// What a certificate says — its subject, its alternative names, its usage.
mod says;
use says::subject;
pub(super) use says::{eku, san};

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
/// server reads back off the presented certificate (REMOTE §2) — and the two
/// X.509 facts below. `hosts` is every way in the server answers to; a client
/// leaf ignores it, because nothing dials a client.
pub(super) fn leaf(dir: &Path, role: Role, hosts: &[String]) -> Result<(), String> {
    let subject = format!("/CN={}", role.common_name());
    issue(
        dir,
        &role.leaf(),
        &subject,
        &san(role, hosts),
        eku(role),
        None,
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
    let (subject, san) = (subject(cn, grade), format!("DNS:{cn}"));
    issue(dir, cn, &subject, &san, eku(Role::Client), None)
}

/// [`stated_leaf`] for a public key **somebody else made** (REMOTE §15.1,
/// bl-9ea6): the same subject, SAN and EKU, signed over `public` — a PEM
/// `SubjectPublicKeyInfo` — instead of over a key minted here. The recipe is
/// the one above with one flag more: the request is still made with a scratch
/// key, because `x509 -req` wants a request to read the subject from, and
/// `-force_pubkey` then replaces what the certificate binds. That spelling is
/// the one OpenSSL and LibreSSL both have; `x509 -new` is OpenSSL 3 alone.
///
/// The scratch key authenticates nothing and is removed whether or not the
/// signature was made, with the rest of the scratch: a `<cn>.key` left behind
/// would read as half a pair to every later act under that name.
pub(super) fn offered_leaf(
    dir: &Path,
    cn: &str,
    grade: Grade,
    public: &Path,
) -> Result<(), String> {
    let (subject, san) = (subject(cn, grade), format!("DNS:{cn}"));
    let signed = issue(dir, cn, &subject, &san, eku(Role::Client), Some(public));
    for scratch in ["key", "csr", EXT] {
        let _ = std::fs::remove_file(dir.join(format!("{cn}.{scratch}")));
    }
    signed
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
fn issue(
    dir: &Path,
    name: &str,
    subject: &str,
    san: &str,
    eku: &str,
    forced: Option<&Path>,
) -> Result<(), String> {
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
    let paths = [
        ext.clone(),
        csr.clone(),
        dir.join(ANCHORS),
        dir.join(CA_KEY),
        dir.join(format!("{name}.pem")),
        forced.map(Path::to_path_buf).unwrap_or_default(),
    ]
    .map(|path| path.to_string_lossy().into_owned());
    let [ext_s, csr_s, ca, ca_key, out, public] = &paths;
    // `-CAcreateserial`: LibreSSL's `x509` refuses to sign when the CA's
    // serial file is absent and it was not told it may make one; OpenSSL 3
    // accepts the flag and does the same thing. Portable, and the file is
    // scratch.
    let mut sign = vec![
        "x509",
        "-req",
        "-sha256",
        "-days",
        DAYS,
        "-extfile",
        ext_s,
        "-extensions",
        SECTION,
        "-in",
        csr_s,
        "-CA",
        ca,
        "-CAkey",
        ca_key,
        "-CAcreateserial",
        "-out",
        out,
    ];
    if forced.is_some() {
        sign.extend(["-force_pubkey", public.as_str()]);
    }
    tool(&sign)?;
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
