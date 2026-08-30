//! **The wire conformance corpus** (REMOTE §3, bl-32cb): one canonical fixture
//! set, generated from the codec itself, that every client replays against its
//! own encode and decode.
//!
//! The hazard it answers is N implementations of one vocabulary. yog speaks the
//! boundary, and so does every seat, tool host and phone that dials it — in
//! languages that cannot link a shared types crate — so the failure mode is one
//! of them being a quiet miss rather than a loud refusal. A shared crate
//! protects only same-language consumers; a corpus protects every consumer,
//! because a fixture is data.
//!
//! **It is generated, never authored.** The values are the surfaces the codec's
//! own round trips already walk — [`codec::tests::surface`](crate::boundary::codec::tests::surface)
//! for the request half, the reply round trip's own for the answer half — so a
//! fixture a client is judged against and a fixture yog proves itself against
//! are one thing. There is no second list to keep true.
//!
//! **The gate is [`store::check`], run as an ordinary test**, and the
//! regeneration is `make corpus`. Nothing here ships in the binary: the
//! generator is test-gated, and what a client consumes is the committed
//! `corpus/` directory. That shape was chosen over a build script or a
//! published artifact for the reason the 100% floor makes decisive — a
//! generator only CI runs is a generator whose own arms nothing covers, and
//! this one is exercised by the suite on every commit.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

mod ledger;
mod store;
mod tests;

/// The request half's directory, and the answer half's.
const REQUEST: &str = "request";
const REPLY: &str = "reply";

/// The environment variable `make corpus` names its destination with. Absent —
/// every ordinary test run — the gate verifies instead of writing.
const DESTINATION: &str = "YOG_CORPUS_OUT";

/// One wire shape: an `op` token on the request side, a reply `kind` on the
/// answer side, and every fixture the boundary spells for it.
pub(crate) struct Shape {
    direction: &'static str,
    name: String,
    frames: Vec<Value>,
}

impl Shape {
    /// The shape's name in the standing record.
    pub(crate) fn key(&self) -> String {
        format!("{}/{}", self.direction, self.name)
    }

    fn path(&self) -> String {
        format!("{}/{}.json", self.direction, self.name)
    }

    /// The fixture file's canonical bytes. `protocol` is **this shape's** —
    /// the version at which its fields last moved, which is what a client
    /// needs to know and what the standing record keeps.
    fn render(&self, protocol: u32) -> String {
        canonical(&json!({
            "protocol": protocol,
            "direction": self.direction,
            "shape": self.name,
            "frames": self.frames,
        }))
    }
}

/// Canonical JSON: pretty-printed with sorted keys (serde_json's map is
/// ordered, so nothing here has to sort), one trailing newline. Deterministic
/// by construction — no clock, no counter, no address.
fn canonical(value: &Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(value).unwrap_or_default()
    )
}

/// The protocol this build speaks — the wire's own constant, never a second.
fn protocol() -> u32 {
    crate::wire::hello::PROTOCOL
}

/// Every shape, request half then answer half, each sorted by name.
pub(crate) fn shapes() -> Vec<Shape> {
    let requests = crate::boundary::codec::tests::surface::gestures()
        .iter()
        .map(crate::boundary::codec::encode)
        .collect();
    let mut replies: Vec<Value> = crate::boundary::reply::tests::roundtrip::surface::surface()
        .iter()
        .map(crate::boundary::reply::encode)
        .collect();
    // The envelope with no `kind`: a refused gesture. A client that decodes
    // every answer must decode this one, so it is a shape like any other.
    replies.push(crate::boundary::reply::refusal("unknown op \"fhtagn\""));
    let mut out = grouped(REQUEST, requests, op_of);
    out.extend(grouped(REPLY, replies, kind_of));
    out
}

fn op_of(frame: &Value) -> String {
    frame
        .get("op")
        .and_then(Value::as_str)
        .unwrap_or("unspelled")
        .to_owned()
}

fn kind_of(frame: &Value) -> String {
    frame
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("refusal")
        .to_owned()
}

/// Bucket frames by the token that names their shape, dropping a repeat: two
/// gestures that spell one envelope are one fixture.
fn grouped(direction: &'static str, frames: Vec<Value>, key: fn(&Value) -> String) -> Vec<Shape> {
    let mut buckets: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for frame in frames {
        let bucket = buckets.entry(key(&frame)).or_default();
        if !bucket.contains(&frame) {
            bucket.push(frame);
        }
    }
    buckets
        .into_iter()
        .map(|(name, frames)| Shape {
            direction,
            name,
            frames,
        })
        .collect()
}

/// The corpus committed in this repository — the one clients read.
fn committed() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

/// Where `make corpus` is writing, when it is.
fn destination() -> Option<PathBuf> {
    std::env::var_os(DESTINATION).map(PathBuf::from)
}

/// Regenerate when asked, verify when not — one entry, so the bytes the gate
/// demands are the bytes the regeneration writes.
fn run(destination: Option<PathBuf>, committed: &Path) -> Result<(), String> {
    match destination {
        Some(dir) => store::bless(&dir),
        None => store::check(committed),
    }
}
