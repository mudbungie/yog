//! What the LINKED brazen actually puts on the wire for an Ollama row (bl-671d)
//! — the evidence behind
//! [`ProviderRow::context_caveat`](yog::config_edit::brazen::ProviderRow::context_caveat)
//! and its [`CONTEXT_REMEDY`](yog::config_edit::brazen::CONTEXT_REMEDY), and the
//! thing that fails the day either stops being true.
//!
//! A drive through the offered Ollama provider reached local inference and could
//! not produce one useful agent turn: 4095 input tokens, one generated token, and
//! `finish_reason: length`, against a model whose own context was 262144. The
//! reason is not the model and not the box — it is the request. brazen's
//! `ollama_chat` encoder maps the output cap to `options.num_predict` and emits
//! no `options.num_ctx` at all, so the Ollama server's own default context
//! governs every yog turn, and a turn's tool payload alone can fill a small one.
//!
//! Three legs, because a caveat that only states the defect leaves the operator
//! nowhere: the plain turn carries no context size; the one-line config fix
//! **lands** (it did not, before brazen 0.0.10 folded the row's `extra` one
//! namespace deep — bl-f19d, and leg two is what caught the day it changed);
//! and the longer recipe, which clears the typed cap, lands too. Each is
//! asserted against yog's own linked brazen — no network, no Ollama, and no
//! second copy of brazen's behaviour written down in prose.
//!
//! The transport is the only seam involved: [`Capture`] takes the encoded
//! request, keeps the body, and refuses the round trip, so the pipeline runs
//! exactly as far as the wire and no further.

#![allow(clippy::unwrap_used)]

use std::io;
use std::sync::OnceLock;

/// The transport that answers with the request. `OnceLock` rather than a lock:
/// [`brazen::Transport`] is `Sync`, one request crosses it per run, and the
/// house forbids a `Mutex` outside `state.rs`.
struct Capture {
    body: OnceLock<String>,
}

impl brazen::Transport for Capture {
    fn send(
        &self,
        wire: brazen::WireRequest,
    ) -> Result<brazen::TransportResponse, brazen::CanonicalError> {
        drop(
            self.body
                .set(String::from_utf8_lossy(&wire.body).into_owned()),
        );
        Err(brazen::CanonicalError {
            kind: brazen::ErrorKind::Transport,
            message: "captured".into(),
            provider_detail: None,
            retry_after_seconds: None,
        })
    }
}

/// No credential: the built-in `ollama` row is `auth = "none"`, so nothing is
/// ever asked of the store.
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

/// The canonical request a yog turn is: a system prompt, one user message, the
/// `clients` tool yog injects into every turn, and litany's per-call output cap
/// (`build_request` sets `max_tokens: Some(4096)` and no `stop`). Written as the
/// bytes litany pipes to `bz`, because that is the interface — a struct literal
/// here would be a second spelling of it.
const TURN: &[u8] =
    br#"{"model":"qwen3-coder","system":[{"type":"text","text":"you are a worker"}],
    "messages":[{"role":"user","content":[{"type":"text","text":"hi"}]}],
    "tools":[{"name":"clients","description":"the machines that can run work",
              "input_schema":{"type":"object","properties":{}}}],
    "max_tokens":4096}"#;

/// Run one `bz --json --provider ollama` turn against `config` (the wall's own
/// brazen `config.toml` text) and return the request body that reached the wire.
fn wire_body(config_text: &str) -> serde_json::Value {
    let dir = tempfile::TempDir::new().unwrap();
    let config = dir.path().join("config.toml");
    std::fs::write(&config, config_text).unwrap();
    let mut env = std::collections::BTreeMap::new();
    env.insert(
        "BRAZEN_CONFIG".to_owned(),
        config.to_string_lossy().into_owned(),
    );
    let args = brazen::Args {
        argv: vec![
            "--json".to_owned(),
            "--provider".to_owned(),
            "ollama".to_owned(),
        ],
        env: brazen::EnvSnapshot(env),
        tty: false,
        stdout_tty: false,
    };
    let transport = Capture {
        body: OnceLock::new(),
    };
    let stash = brazen::ReplayStash::new(dir.path().join("stash"));
    let host = brazen::Host {
        transport: &transport,
        store: &NoStore,
        cache: &NoCache,
        clock: &Zero,
        stash: &stash,
    };
    let mut stdin = TURN;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    // The run always fails — `Capture` refuses the round trip after keeping the
    // body — so the exit code carries nothing this test wants.
    let _code = brazen::run(args, &mut stdin, &mut stdout, &mut stderr, &host);
    let body = transport.body.get().cloned().unwrap_or_default();
    assert!(
        !body.is_empty(),
        "the turn never reached the transport seam: {}",
        String::from_utf8_lossy(&stdout)
    );
    serde_json::from_str(&body).unwrap()
}

/// Leg one — the defect. The wire carries the output cap and no context size, so
/// the server's default governs. `num_predict` is asserted beside the absence:
/// the two limits are distinct, and a fix must not collapse them.
#[test]
fn a_yog_turn_reaches_ollama_with_no_context_size() {
    let body = wire_body("");
    let options = &body["options"];
    assert_eq!(options["num_predict"], 4096, "{body}");
    assert!(
        options.get("num_ctx").is_none(),
        "brazen now sends a context size — delete the caveat and the remedy: {body}"
    );
    // The tool payload the 4K default was spent on is really on the wire, so the
    // caveat's "a turn's tool payload alone can exhaust it" is about this request.
    assert_eq!(body["tools"][0]["function"]["name"], "clients", "{body}");
}

/// Leg two — the obvious fix IS the fix, since brazen 0.0.10. A nested
/// `options` in the row's `body_defaults` used to be dropped **whole and
/// silently**, because the encoder inserted the typed `options` (holding
/// `num_predict`) first and folded config passthrough with `or_insert`; that is
/// why [`CONTEXT_REMEDY`] once told the operator to clear the typed cap and
/// restate it inside the object. brazen bl-f19d folds the `extra` one namespace
/// deep instead, so the two keys compose per key and the operator writes one
/// line. This leg is what fails the day that stops being true — the same
/// two-direction discipline it had when it proved the drop.
#[test]
fn a_nested_options_default_composes_with_the_typed_cap() {
    let body = wire_body(
        "[[provider]]\nname = \"ollama\"\n\
         body_defaults = { options = { num_ctx = 32768 } }\n",
    );
    assert_eq!(
        body["options"]["num_ctx"], 32768,
        "the passthrough valve no longer composes with the typed options — the \
         one-line remedy is wrong again: {body}"
    );
    assert_eq!(
        body["options"]["num_predict"], 4096,
        "the typed cap survives the fold beside it: {body}"
    );
}

/// Leg three — the remedy the caveat hands the operator, run. Clearing the typed
/// cap empties the encoder's own `options`, so the passthrough object lands
/// whole; the cap is restated inside it, so both limits are present and
/// distinct, and the operator's own numbers are the ones on the wire.
#[test]
fn clearing_the_typed_cap_lets_an_explicit_context_through() {
    let body = wire_body(
        "[[provider]]\nname = \"ollama\"\n\
         unsupported_body_keys = [\"max_tokens\"]\n\
         body_defaults = { options = { num_ctx = 32768, num_predict = 2048 } }\n",
    );
    assert_eq!(body["options"]["num_ctx"], 32768, "{body}");
    assert_eq!(
        body["options"]["num_predict"], 2048,
        "the operator's explicit output cap wins over litany's request: {body}"
    );
}
