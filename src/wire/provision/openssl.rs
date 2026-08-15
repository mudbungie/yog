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
use crate::wire::material::Role;
use std::net::IpAddr;
use std::path::Path;

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

/// One CA-signed leaf: a key, a request carrying its SAN and EKU, then the
/// signature. The subject common name **is** the client identity the engine
/// reads back off the presented certificate (REMOTE §2), so it comes from
/// [`Role::common_name`] rather than being spelled here.
pub(super) fn leaf(dir: &Path, role: Role, host: &str) -> Result<(), String> {
    let name = role.leaf();
    let key = dir.join(format!("{name}.key"));
    let csr = dir.join(format!("{name}.csr"));
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
        &format!("/CN={}", role.common_name()),
        "-addext",
        &format!("subjectAltName={}", san(role, host)),
        "-addext",
        &format!("extendedKeyUsage={}", eku(role)),
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
        "-copy_extensions",
        "copy",
        "-in",
        &csr.to_string_lossy(),
        "-CA",
        &dir.join(ANCHORS).to_string_lossy(),
        "-CAkey",
        &dir.join(CA_KEY).to_string_lossy(),
        "-out",
        &dir.join(format!("{name}.pem")).to_string_lossy(),
    ])?;
    let _ = std::fs::remove_file(&csr);
    private(&key, 0o600);
    Ok(())
}

/// A leaf's subject alternative name. The **server**'s is derived from the
/// address, because that is the name a seat verifies against what it dialled —
/// an IP literal is an IP identity and anything else a DNS one, the same rule
/// [`client`](super::client) reads it back by. A client leaf's names itself:
/// nothing dials a client.
///
/// **Loopback is always on the server leaf** (bl-ae05). The local window is a
/// client of `127.0.0.1` unconditionally — that is what the ruling means by the
/// front door — so a server certificate that only named an operator's public
/// host would refuse the one seat that is certain to be there. It costs one
/// SAN entry and removes a whole class of "the window cannot reach its own
/// engine".
pub(super) fn san(role: Role, host: &str) -> String {
    let loopback = format!("IP:{LOOPBACK}");
    match role {
        Role::Server if host.parse::<IpAddr>().is_ok() => {
            let named = format!("IP:{host}");
            if named == loopback {
                named
            } else {
                format!("{named},{loopback}")
            }
        }
        Role::Server => format!("DNS:{host},{loopback}"),
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
    let out = git_env::command(program).args(args).output().map_err(|e| {
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
