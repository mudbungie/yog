//! **The headless workspace binding** (DESIGN §8.4 as amended, bl-b589), driven
//! through the real `yog` binary — the seat a teleoperator actually has.
//!
//! Sign-in is the case that motivated it: `yog bz --login` outside a workspace
//! refuses (§16.2 — providers, credentials and the model cache belong to a
//! sphere, and nothing is ambient to fall back to), and before this there was
//! no spelling that named one. `yog exec --ws <workspace>` is that spelling.
//!
//! The proof here is offline and needs no browser. A **credential is a file**
//! in the wall (`<wall>/brazen/credentials/<provider>.json`, §5.1 #22) and
//! `bz --list-providers` reads exactly that dir to say whether a row is signed
//! in — so seeding one wall and asking through each spelling proves where a
//! sign-in would land, without a network round-trip.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run the built `yog` with a hermetic `$XDG_DATA_HOME`, returning
/// `(exit, stdout, stderr)`. The world — and so every wall under it — is rooted
/// inside `home`, so nothing here can read or write the operator's own state.
fn yog(home: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_yog"))
        .args(args)
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_STATE_HOME", home.join("state"))
        .env("XDG_CONFIG_HOME", home.join("config"))
        .env("XDG_CACHE_HOME", home.join("cache"))
        .env("HOME", home.join("home"))
        // Deliberately standing and deliberately inert: if the wall fold ever
        // fell through to brazen's own env knob, this is the value it would
        // find, and every assertion below would notice.
        .env("BRAZEN_CONFIG", home.join("ambient-brazen.toml"))
        .env_remove("YOG_WALL")
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The wall of `workspace` under this fixture's world — the same pure fold the
/// hatch performs (`<world>/walls/<leaf>`), so the test writes where the
/// binary reads without restating a path shape.
fn wall(home: &Path, workspace: &Path) -> PathBuf {
    home.join("data/yog/world/walls")
        .join(workspace.file_name().unwrap())
}

/// Seed one provider row into `workspace`'s wall, and — when `signed_in` — the
/// credential file `bz` reads presence from.
fn seed(home: &Path, workspace: &Path, signed_in: bool) {
    let brazen = wall(home, workspace).join("brazen");
    fs::create_dir_all(brazen.join("credentials")).unwrap();
    fs::write(
        brazen.join("config.toml"),
        "[[provider]]\nname = \"acme\"\nprotocol = \"openai_chat\"\n\
         base_url = \"https://acme.test\"\nauth = \"api_key\"\n\
         api_header = { name = \"x-api-key\", scheme = \"raw\" }\n",
    )
    .unwrap();
    if signed_in {
        fs::write(
            brazen.join("credentials/acme.json"),
            // brazen's own `Cred` shape: the variant IS the token-kind
            // discriminant and the file path is the provider name.
            "{\"ApiKey\":{\"key\":\"seeded-by-the-test\"}}\n",
        )
        .unwrap();
    }
}

#[test]
fn a_sign_in_reaches_only_the_wall_the_hatch_names() {
    let home = tempfile::TempDir::new().unwrap();
    let (corp, personal) = (Path::new("/spheres/corp"), Path::new("/spheres/personal"));
    // One sphere is signed in to `acme`; the other knows the row and is not.
    seed(home.path(), corp, true);
    seed(home.path(), personal, false);

    // The defect, verbatim: the advertised bare spelling has no wall and is
    // refused — and that refusal is *kept*, so a credential can never land
    // somewhere shared.
    let (exit, _, err) = yog(home.path(), &["bz", "--login", "--provider", "acme"]);
    assert_eq!(exit, 64, "a wall-less sign-in still refuses: {err}");
    assert!(err.contains("no workspace in this environment"), "{err}");

    // The fix: the hatch names the sphere, and each sphere answers for itself.
    let asked = |ws: &Path| {
        let (exit, out, err) = yog(
            home.path(),
            &[
                "exec",
                "--ws",
                ws.to_str().unwrap(),
                "bz",
                "--list-providers",
            ],
        );
        assert_eq!(exit, 0, "{ws:?}: {err}");
        out
    };
    let in_corp = asked(corp);
    let in_personal = asked(personal);
    // Both spheres declare the row — so any difference below is the credential,
    // not the config.
    assert!(in_corp.contains("acme"), "{in_corp}");
    assert!(in_personal.contains("acme"), "{in_personal}");
    // …and only the sphere whose wall holds the credential file reads as signed
    // in. This is the whole claim: a sign-in fired through `--ws corp` writes
    // into corp's wall and is invisible from personal's.
    let signed = |table: &str| {
        table
            .lines()
            .find(|l| l.contains("acme"))
            .unwrap_or_default()
            .to_owned()
    };
    assert_ne!(
        signed(&in_corp),
        signed(&in_personal),
        "the same row must not read alike in a signed-in and a signed-out sphere\n\
         corp: {in_corp}\npersonal: {in_personal}"
    );
}

#[test]
fn the_env_hatch_hands_out_the_named_wall_and_only_when_named() {
    let home = tempfile::TempDir::new().unwrap();
    let corp = Path::new("/spheres/corp");

    // `eval "$(yog env --ws …)"` is the other half of the spelling: a whole
    // shell inside the sphere, where a bare `bz --login` is the world's own
    // shim and reaches that wall.
    let (exit, script, err) = yog(home.path(), &["env", "--ws", corp.to_str().unwrap()]);
    assert_eq!(exit, 0, "{err}");
    let expected = wall(home.path(), corp);
    assert!(
        script.contains(&format!("export YOG_WALL='{}'", expected.display())),
        "{script}"
    );

    // Bare, it is exactly what it was: the world, no wall — so a shell that did
    // not ask for a sphere cannot accidentally sign one in.
    let (exit, bare, _) = yog(home.path(), &["env"]);
    assert_eq!(exit, 0);
    assert!(!bare.contains("YOG_WALL"), "{bare}");

    // And a word the hatch cannot read is refused, never silently dropped.
    let (exit, _, err) = yog(home.path(), &["env", "--ws"]);
    assert_eq!(exit, 2, "{err}");
    assert!(err.contains("--ws requires a workspace path"), "{err}");
}
