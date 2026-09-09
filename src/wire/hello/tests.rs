//! The preface, from both ends, over doubles — the end-to-end skew a real
//! listener refuses lives beside the listener it refuses at
//! (`wire::server::tests::protocol`).

use super::*;
use serde_json::Value;

/// A connection standing in for a peer: what it will say, and what was said to
/// it. Both directions on one object, because [`admit`] needs both and a
/// socket is one thing.
struct Peer {
    says: std::io::Cursor<Vec<u8>>,
    heard: Vec<u8>,
}

impl Peer {
    /// A peer whose first frame is `preface`.
    fn stating(preface: &Value) -> Self {
        let mut says = Vec::new();
        frame::write_value(&mut says, preface).expect("frame");
        Self {
            says: std::io::Cursor::new(says),
            heard: Vec::new(),
        }
    }

    /// A peer that says nothing at all — the hang-up, and the whole of what an
    /// unreadable first frame is worth telling apart from.
    fn silent() -> Self {
        Self {
            says: std::io::Cursor::new(Vec::new()),
            heard: Vec::new(),
        }
    }

    /// Every frame written to this peer, decoded.
    fn frames(&self) -> Vec<Value> {
        let mut cursor = std::io::Cursor::new(self.heard.clone());
        let mut out = Vec::new();
        while let Ok(Some(v)) = frame::read_value(&mut cursor) {
            out.push(v);
        }
        out
    }
}

impl Read for Peer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.says.read(buf)
    }
}

impl Write for Peer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.heard.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A peer the socket to which is already gone.
struct Gone;

impl Read for Gone {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("gone"))
    }
}

impl Write for Gone {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("gone"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("gone"))
    }
}

/// This end's own preface, which every beat below compares against: both
/// keys, the major and the edition (REMOTE §3.2).
fn ours() -> Value {
    json!({ "protocol": PROTOCOL, "edition": EDITION })
}

/// The agreement: a peer of this build's own version is admitted, and what it
/// was told is exactly one frame — this end's own preface, and no reply.
#[test]
fn a_peer_of_this_version_is_admitted_and_told_ours() {
    let mut peer = Peer::stating(&ours());
    assert_eq!(admit(&mut peer), Some(EDITION));
    assert_eq!(peer.frames(), vec![ours()]);
}

/// **The additive case, and the whole reason the edition is not adjudicated**
/// (REMOTE §3.2): a peer of this major that states an edition NEWER than this
/// build's is admitted, and its edition is what comes back — an addition ships
/// without a bump, so this is the ordinary posture of a seat one release ahead
/// of the engine, not a skew.
#[test]
fn a_peer_of_a_later_edition_is_admitted_and_its_edition_read() {
    let later = EDITION + 1;
    let mut peer = Peer::stating(&json!({ "protocol": PROTOCOL, "edition": later }));
    assert_eq!(admit(&mut peer), Some(later));
    assert_eq!(peer.frames(), vec![ours()], "no reply, only our preface");
}

/// **A peer that states no edition speaks the floor.** Every build older than
/// bl-1be7 is this peer, and reading it as the floor is not a guess: every
/// engine of this major writes every path stamped at or below it. A stamp that
/// is not one — a string, a number no `u32` can hold — is the same silence,
/// because there is no third thing a reader could do with it.
#[test]
fn a_peer_that_states_no_usable_edition_reads_as_the_floor() {
    for said in [
        json!({ "protocol": PROTOCOL }),
        json!({ "protocol": PROTOCOL, "edition": "nineteen" }),
        json!({ "protocol": PROTOCOL, "edition": 4_294_967_296_u64 }),
    ] {
        let mut peer = Peer::stating(&said);
        assert_eq!(admit(&mut peer), Some(FLOOR), "{said}");
    }
}

/// The skew: refused, and told a sentence naming **both** versions. That is
/// the whole of the upgrade prompt, so the test reads it rather than a code.
#[test]
fn a_peer_of_another_version_is_refused_by_name() {
    // Relative to OUR version, so the beat survives a lawful bump — the
    // constant is the one authority and the fixture derives from it.
    let other = PROTOCOL + 1;
    let mut peer = Peer::stating(&json!({ "protocol": other }));
    assert_eq!(admit(&mut peer), None);
    let frames = peer.frames();
    assert_eq!(frames.len(), 2, "our preface, then the refusal");
    assert_eq!(frames[0], ours());
    assert_eq!(frames[1]["ok"], json!(false));
    let said = frames[1]["error"].as_str().expect("a sentence");
    assert!(said.contains(&format!("version {PROTOCOL}")), "{said}");
    assert!(said.contains(&format!("the peer speaks {other}")), "{said}");
    assert!(said.contains("no negotiation"), "{said}");
}

/// **Three silences are one case.** An unversioned build (a gesture envelope
/// where a preface belongs), a frame that is not an object, and a peer that
/// hung up all state no version — and each earns the same sentence, which
/// names what it could not learn rather than pretending to a number.
#[test]
fn a_peer_that_states_no_version_is_refused_the_same_way() {
    for mut peer in [
        Peer::stating(&json!({ "op": "workspaces" })),
        Peer::stating(&json!("not an object")),
        Peer::silent(),
    ] {
        assert_eq!(admit(&mut peer), None);
        let frames = peer.frames();
        let said = frames[1]["error"].as_str().expect("a sentence");
        assert!(said.contains("the peer speaks no version"), "{said}");
    }
}

/// A key of the right name and the wrong type is not a version either — the
/// strict-decode discipline the framing already keeps, at the preface.
#[test]
fn a_version_that_is_not_a_number_states_none() {
    let mut peer = Peer::stating(&json!({ "protocol": "1" }));
    assert_eq!(admit(&mut peer), None);
    let said = peer.frames()[1]["error"]
        .as_str()
        .expect("a sentence")
        .to_owned();
    assert!(said.contains("the peer speaks no version"), "{said}");
}

/// A peer already gone is refused without a word: there is no socket to say
/// the sentence on, and the answer is the same one.
#[test]
fn a_peer_that_cannot_be_greeted_is_refused() {
    assert_eq!(admit(&mut Gone), None);
}

/// The seat's half of the same rule. Agreement is silent; a skew is the one
/// `Err(String)` every other transport failure already arrives as.
#[test]
fn the_seat_confirms_or_refuses_on_the_same_sentence() {
    let mut agreed = Peer::stating(&ours());
    assert_eq!(confirm(&mut agreed), Ok(()));

    let mut skewed = Peer::stating(&json!({ "protocol": 99 }));
    let said = confirm(&mut skewed).expect_err("a mismatch refuses");
    assert!(said.contains(&format!("version {PROTOCOL}")), "{said}");
    assert!(said.contains("the peer speaks 99"), "{said}");

    let said = confirm(&mut Gone).expect_err("a silent engine refuses");
    assert!(said.contains("the peer speaks no version"), "{said}");
}

/// The preface this build writes is a frame like any other, so a writer that
/// refuses bytes is an error and not a panic.
#[test]
fn stating_onto_a_dead_socket_is_an_error() {
    assert!(state(&mut Gone).is_err());
}
