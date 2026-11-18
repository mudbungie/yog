//! Which flow the spawn is fired in (§8.3 rule 1), over a real wall.

use tempfile::tempdir;

use super::super::Runs;
use super::{lines, script, settled, workspace};
use crate::cli_outbound::Cli;
use crate::config_edit::brazen::{BzRunner as _, RealBzRunner};

/// **The flow follows the row** (§8.3 rule 1 as amended by bl-61bf; bl-7c9f).
/// The argv is the whole assertion, because `--browser` is the one word that
/// selects a flow: a row brazen's own table declares a device endpoint for is
/// fired headless — bz binds no loopback and the verification URL and user code
/// stream down the lane, so the sign-in completes from any seat — and a row
/// declaring none is fired `--browser`, byte for byte as before.
#[test]
fn the_spawn_drops_browser_exactly_where_the_row_declares_a_device_endpoint() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    // A browser-only oauth row, authored in this workspace's own wall beside
    // one NAMED built-in: `openai-chatgpt` is the preset that declares a device
    // endpoint, in the table only because the wall names it (brazen bl-6feb),
    // and the hand-written row deliberately declares none.
    let paths = crate::test_support::world::wall_paths(dir.path());
    std::fs::create_dir_all(paths.config.parent().expect("the wall's brazen dir")).expect("mkdir");
    std::fs::write(
        &paths.config,
        "[[provider]]\nname = \"acme-oauth\"\nprotocol = \"openai_chat\"\n\
         base_url = \"https://acme.test\"\nauth = \"oauth2\"\n\
         api_header = { name = \"Authorization\", scheme = \"bearer\" }\n\
         [provider.oauth]\nclient_id = \"c\"\n\
         authorize_url = \"https://acme.test/authorize\"\n\
         token_url = \"https://acme.test/token\"\n\n\
         [[provider]]\nname = \"openai-chatgpt\"\n",
    )
    .expect("write");
    // bz echoes the argv it was fired with, then the device flow's own line.
    let bz = script(
        dir.path(),
        "bz",
        "#!/bin/sh\nprintf '%s\\n' \"$*\" 1>&2\n\
         printf 'open https://example.test/device and enter CODE-1234\\n' 1>&2\nexit 0\n",
    );
    let runs = Runs::of(Cli::new(bz));
    // The act's own read: this wall's catalog, asked of the linked brazen.
    let table = RealBzRunner::new(crate::world::wall::env(&world, &workspace())).catalog();
    let flow = |row: &str| super::super::flow_of(&table, row);

    runs.start(
        &world,
        &workspace(),
        "openai-chatgpt",
        flow("openai-chatgpt"),
        state.path(),
        "100",
    )
    .expect("started");
    let headless = lines(&settled(&runs, "openai-chatgpt"));
    assert_eq!(
        headless[0], "--login --provider openai-chatgpt",
        "no --browser: bz binds no loopback and serves the row's device flow"
    );
    assert!(
        headless[1].contains("https://example.test/device") && headless[1].contains("CODE-1234"),
        "the URL and the user code stream down the lane: {headless:?}"
    );

    runs.start(
        &world,
        &workspace(),
        "acme-oauth",
        flow("acme-oauth"),
        state.path(),
        "100",
    )
    .expect("started");
    assert_eq!(
        lines(&settled(&runs, "acme-oauth"))[0],
        "--login --provider acme-oauth --browser",
        "a row declaring no device endpoint keeps the loopback flow"
    );
}
