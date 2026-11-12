//! **Where pending offers live** (REMOTE §15.1; bl-9ea6): one small file per
//! offered name under the state root, beside the registry they are waiting to
//! become part of.
//!
//! **On disk, not in RAM, and the reason is the hold.** A claimed offer has
//! issued a certificate that chains to the operator's CA and cannot be
//! recalled — there is no revocation here — so the fact *this leaf is not
//! approved* must outlive the process that issued it. Kept in RAM, a restart
//! between the claim and the approval would forget the hold and leave an
//! unapproved operator-grade leaf free to found a workspace (§4's creator
//! auto-registration). Kept here, the hold fails closed across a restart, and
//! an open offer survives one for what is left of its ten minutes.
//!
//! **Two directories, and the move between them is the single use.**
//! `offers/open/<name>` is a code somebody may still claim;
//! `offers/held/<name>` is a name whose leaf has been issued, or is being.
//! A claim is a `rename` from the first to the second, which the filesystem
//! performs for exactly one of two racing claimants.

use crate::registry::{Client, Grade};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The offers directory under the state root, and its two halves.
pub const DIR: &str = "offers";
const OPEN: &str = "open";
const HELD: &str = "held";

/// The word a declined-after-claim record carries. Not a [`State`](super::State):
/// it is never listed, only held.
const DECLINED: &str = "declined";

/// One offer, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The common name offered.
    pub name: String,
    /// The workspace the client will be seated in on approval.
    pub workspace: String,
    pub grade: Grade,
    /// The single-use code. Kept after the claim only as history.
    pub code: String,
    /// When the code stops being claimable, unix seconds.
    pub expires: i64,
    /// The confirmation word, once a device has claimed.
    pub word: Option<String>,
    /// When it was claimed, unix seconds — what `proved` is dated against.
    pub claimed: Option<i64>,
    /// Declined after its leaf was issued: never listed, held forever.
    pub declined: bool,
}

impl Record {
    fn encode(&self) -> String {
        json!({
            "name": self.name, "workspace": self.workspace, "grade": self.grade.word(),
            "code": self.code, "expires": self.expires, "word": self.word,
            "claimed": self.claimed, DECLINED: self.declined,
        })
        .to_string()
    }

    fn decode(text: &str) -> Option<Self> {
        let v: Value = serde_json::from_str(text).ok()?;
        let text_of = |key: &str| v.get(key).and_then(Value::as_str).map(str::to_owned);
        Some(Self {
            name: text_of("name")?,
            workspace: text_of("workspace")?,
            grade: Grade::of(&text_of("grade")?)?,
            code: text_of("code")?,
            expires: v.get("expires")?.as_i64()?,
            word: text_of("word"),
            claimed: v.get("claimed").and_then(Value::as_i64),
            declined: v.get(DECLINED).and_then(Value::as_bool).unwrap_or(false),
        })
    }
}

fn half(state_root: &Path, half: &str) -> PathBuf {
    state_root.join(DIR).join(half)
}

fn write(dir: &Path, record: &Record) -> Result<(), String> {
    let path = dir.join(&record.name);
    std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&path, record.encode()))
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn read_all(dir: &Path) -> Vec<Record> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut records: Vec<Record> = entries
        .flatten()
        .filter_map(|entry| Record::decode(&std::fs::read_to_string(entry.path()).ok()?))
        .collect();
    records.sort_by(|a, b| a.name.cmp(&b.name));
    records
}

/// Record a freshly minted offer, replacing an open one under the same name —
/// minting again is how an operator re-issues a code that expired or was lost.
pub fn mint(state_root: &Path, record: &Record) -> Result<(), String> {
    write(&half(state_root, OPEN), record)
}

/// The open offers whose code has not expired at `now`. Expired ones are
/// deleted on the way past, so nothing sweeps them and nothing needs to.
fn open(state_root: &Path, now: i64) -> Vec<Record> {
    let dir = half(state_root, OPEN);
    let mut live = Vec::new();
    for record in read_all(&dir) {
        if record.expires > now {
            live.push(record);
        } else {
            let _ = std::fs::remove_file(dir.join(&record.name));
        }
    }
    live
}

/// Every offer still pending at `now`: the open ones, then the held ones not
/// declined.
pub fn pending(state_root: &Path, now: i64) -> Vec<Record> {
    let mut live = open(state_root, now);
    live.extend(
        read_all(&half(state_root, HELD))
            .into_iter()
            .filter(|record| !record.declined),
    );
    live
}

/// **Take the open offer `code` names — the single use.** `Some` for exactly
/// one caller: the record moves to the held half by `rename`, so a second
/// claimant of the same code finds nothing, and so does a code that is
/// unknown or expired. The three are one answer by construction.
pub fn take(state_root: &Path, code: &str, now: i64) -> Option<Record> {
    let record = open(state_root, now)
        .into_iter()
        .find(|record| record.code == code)?;
    let held = half(state_root, HELD);
    std::fs::create_dir_all(&held).ok()?;
    std::fs::rename(
        half(state_root, OPEN).join(&record.name),
        held.join(&record.name),
    )
    .ok()?;
    Some(record)
}

/// Put a taken offer back — the claim's own failure path, so a device whose
/// first request could not be signed may try again inside the code's life.
pub fn restore(state_root: &Path, record: &Record) {
    let _ = std::fs::rename(
        half(state_root, HELD).join(&record.name),
        half(state_root, OPEN).join(&record.name),
    );
}

/// Write a held offer's record: the claim's word and stamp, or a decline.
pub fn settle(state_root: &Path, record: &Record) -> Result<(), String> {
    write(&half(state_root, HELD), record)
}

/// Forget an offer entirely: an open one declined, or a held one approved.
pub fn remove(state_root: &Path, name: &str) {
    for dir in [OPEN, HELD] {
        let _ = std::fs::remove_file(half(state_root, dir).join(name));
    }
}

/// **Whether `client`'s leaf is held** — issued by an offer and not approved.
/// A held client is refused every gesture (REMOTE §15.1 step 4), which is what
/// makes an unapproved certificate worth nothing: without the hold it could
/// found a workspace and be registered in it as its creator.
pub fn held(state_root: &Path, client: &Client) -> bool {
    half(state_root, HELD).join(client.name()).is_file()
}

/// What a held client is told, whatever it asked.
pub fn refusal(client: &Client) -> String {
    format!(
        "{name} holds a certificate issued by an offer no operator has approved, so nothing it \
         says is answered yet: show its confirmation word to an operator, who approves it from \
         a seat with /approve {name}",
        name = client.name()
    )
}
