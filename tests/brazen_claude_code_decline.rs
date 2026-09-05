//! What the LINKED brazen actually says when a yog turn meets a tool-less
//! dialect (bl-5252) — the evidence behind
//! [`dialect_decline`](yog::config_edit::brazen::dialect_decline), and the thing
//! that fails the day brazen stops saying it.
//!
//! The classifier that routes a dead step to the §9.1 editor is keyed on brazen's
//! own words, so those words have to be brazen's and not a memory of them. This
//! drives the pinned crate in-process with the request a yog turn is, takes the
//! sentence it declines with, wraps it exactly as litany's `AdapterError` does,
//! and asserts yog classifies the result. No network, no `claude` CLI, and no
//! second copy of brazen's message written down in prose.
//!
//! Three legs. The decline names no config fault, which is why a marker table
//! could not see it; yog routes it anyway; and the same turn through a
//! tool-carrying row reaches the wire, so what is being classified is the
//! **dialect** and never the request.
//!
//! **Both halves are the linked brazen's since bl-b6c9.** The sentence comes
//! from its encoder, and the JUDGEMENT that the sentence names a tool-less
//! dialect now comes from its `--list-providers --json` `tools` column
//! (upstream bl-5053) rather than from a match yog kept — so [`rows`] runs the
//! same crate's listing route in the same process, and this file is end to end
//! through one brazen with nothing about dialects written down in yog.

#![allow(clippy::unwrap_used)]

use std::io;
use std::sync::OnceLock;

/// The wire seam. It refuses the round trip and records that it was asked at
/// all — the fact leg three is about. `OnceLock` rather than a lock:
/// [`brazen::Transport`] is `Sync`, one request crosses it per run, and the house
/// forbids a `Mutex` outside `state.rs`.
struct Wire {
    reached: OnceLock<()>,
}

impl brazen::Transport for Wire {
    fn send(
        &self,
        _wire: brazen::WireRequest,
    ) -> Result<brazen::TransportResponse, brazen::CanonicalError> {
        let _ = self.reached.set(());
        Err(brazen::CanonicalError {
            kind: brazen::ErrorKind::Transport,
            message: "the wire is not what this file is about".into(),
            provider_detail: None,
            retry_after_seconds: None,
        })
    }
}

/// No credential: both rows driven here are `auth = "none"` — `claude-code`
/// because the CLI carries its own OAuth, `ollama` because it is local.
struct NoStore;

impl brazen::CredStore for NoStore {
    fn get(&self, _provider: &str) -> Option<brazen::Cred> {
        None
    }
    fn put(&self, _provider: &str, _cred: &brazen::Cred) -> io::Result<()> {
        Ok(())
    }
    fn discover(&self, _spec: &brazen::AmbientSpec) -> Option<brazen::Cred> {
        None
    }
}

/// A cold model cache: the request names a full model id, which passes through
/// verbatim.
struct NoCache;

impl brazen::ModelCache for NoCache {
    fn get(&self, _provider: &str) -> Option<brazen::CachedModels> {
        None
    }
    fn put(&self, _provider: &str, _cached: &brazen::CachedModels) {}
}

struct Zero;

impl brazen::Clock for Zero {
    fn now(&self) -> u64 {
        0
    }
}

/// The canonical request a yog turn is: a system prompt, one user message, and
/// the `clients` tool yog injects into **every** turn
/// (`tool_host::Injection::tools` returns it unconditionally, and litany splices
/// the injection into every canonical request). Written as the bytes litany pipes
/// to `bz`, because that is the interface — a struct literal here would be a
/// second spelling of it.
const TURN: &[u8] = br#"{"model":"sonnet","system":[{"type":"text","text":"you are a worker"}],
    "messages":[{"role":"user","content":[{"type":"text","text":"hi"}]}],
    "tools":[{"name":"clients","description":"the machines that can run work",
              "input_schema":{"type":"object","properties":{}}}]}"#;

/// The linked brazen's own effective table, projected as yog consumes it —
/// `bz --list-providers --json` through the same in-process route
/// `config_edit::brazen::effects` spawns, over the shipped default rows. This
/// is the answer `config_remedy` judges with, so the column and the decline
/// below come from one crate at one version.
fn rows() -> Vec<yog::config_edit::brazen::ProviderRow> {
    let args = brazen::Args {
        argv: vec!["--list-providers".to_owned(), "--json".to_owned()],
        env: brazen::EnvSnapshot(std::collections::BTreeMap::new()),
        tty: false,
        stdout_tty: false,
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut io = brazen::ProvidersIo {
        stdout: &mut stdout,
        stderr: &mut stderr,
        store: &NoStore,
    };
    assert_eq!(brazen::list_providers(&args, &mut io), 0);
    let listing = String::from_utf8(stdout).unwrap();
    let rows = yog::config_edit::brazen::provider_rows(&listing);
    assert!(
        rows.iter().any(|r| r.protocol == "claude_code"),
        "the shipped table lost its exec row: {listing}"
    );
    rows
}

/// Run one `bz --provider <row>` turn against brazen's shipped defaults and
/// return what it said and whether the request ever reached the wire. brazen
/// writes an in-band error's message to stderr, one line.
fn turn(provider: &str) -> (String, bool) {
    let args = brazen::Args {
        argv: vec!["--provider".to_owned(), provider.to_owned()],
        env: brazen::EnvSnapshot(std::collections::BTreeMap::new()),
        tty: false,
        stdout_tty: false,
    };
    let dir = tempfile::TempDir::new().unwrap();
    let stash = brazen::ReplayStash::new(dir.path().join("stash"));
    let wire = Wire {
        reached: OnceLock::new(),
    };
    let host = brazen::Host {
        transport: &wire,
        store: &NoStore,
        cache: &NoCache,
        clock: &Zero,
        stash: &stash,
    };
    let mut stdin = TURN;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = brazen::run(args, &mut stdin, &mut stdout, &mut stderr, &host);
    let said = String::from_utf8_lossy(&stderr).trim().to_owned();
    assert!(
        !said.is_empty(),
        "the {provider} turn produced no failure line (exit {code}): {}",
        String::from_utf8_lossy(&stdout)
    );
    (said, wire.reached.get().is_some())
}

/// Leg one — why no marker could find this family. The decline lands at ENCODE,
/// before any transport, and it calls itself nothing config-shaped: brazen stamps
/// it `ErrorKind::ParseInput`, so litany's wrapper carries `ParseInput` and not
/// the `config` word the two [`yog::config_edit::fault`] markers key on.
#[test]
fn the_dialect_declines_at_encode_and_never_calls_it_a_config_fault() {
    let (said, reached) = turn("claude-code");
    assert!(
        !reached,
        "the decline is supposed to precede the wire: {said}"
    );
    assert!(said.contains("claude_code"), "{said}");
    assert!(
        !said.to_ascii_lowercase().contains("config"),
        "brazen now calls this a config fault — the markers reach it and the \
         dialect route can go: {said}"
    );
}

/// Leg two — the acceptance. brazen's own sentence, wrapped as litany wraps it,
/// earns the §9.1 route. The wrapper is litany's `Error::AdapterError` display
/// verbatim — `provider error ({kind}) on provider row {row:?}: {message}`, with
/// `kind` the `Debug` of brazen's `ErrorKind`.
#[test]
fn yog_routes_brazens_own_decline_to_the_config_editor() {
    let (said, _) = turn("claude-code");
    let line = format!("provider error (ParseInput) on provider row \"claude-code\": {said}");
    let remedy = yog::config_edit::fault::config_remedy(&line, &rows())
        .expect("a dialect decline has a way out of the banner");
    assert!(remedy.contains("declares no tools"), "{remedy}");
    assert!(remedy.contains("config.toml"), "{remedy}");
    // The judgement is the column's, so an empty table classifies nothing —
    // a listing yog could not read must not become a refusal.
    assert_eq!(
        yog::config_edit::fault::config_remedy(&line, &[]),
        None,
        "an unanswered table refused a step"
    );
}

/// Leg three — the control, and the reason the sentence blames the dialect. The
/// same tool-bearing turn through a tool-carrying row (`ollama`, `auth = "none"`
/// like the row above, so nothing else differs) encodes and reaches the wire; the
/// transport failure it dies of there earns no config remedy, because a wire that
/// dropped is nobody's file.
#[test]
fn the_same_turn_reaches_the_wire_on_a_tool_carrying_row() {
    let (said, reached) = turn("ollama");
    assert!(
        reached,
        "a tool-carrying row should encode this turn: {said}"
    );
    let line = format!("provider error (Transport) on provider row \"ollama\": {said}");
    assert_eq!(
        yog::config_edit::fault::config_remedy(&line, &rows()),
        None,
        "{said}"
    );
}
