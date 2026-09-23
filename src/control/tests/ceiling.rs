//! §3.5's spend ceiling at §8.6's consult (bl-4b48): the world's number read
//! once per consult, and the park it puts on every tool call while the world is
//! at or over it. The beats sit on the consult's own pure seam — [`Consult`] is
//! owned, so the judgment is a function of it — plus the resolution that fills
//! the one field it gained ([`ceiling::parked`]).

use super::world::{World, request};
use crate::control::wire::{Request, Verdict};
use crate::control::{Consult, adjudicate, ceiling};
use crate::opslog::YOG_CONTROL;
use crate::xdg::Env;
use serde_json::json;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// The head the `/ceiling` release selects a parked conversation by, and the
/// one word of the sentence no seat may re-spell.
const HEAD: &str = "spend ceiling reached:";
const CONV: &str = "20260803T120000Z-root";
/// `$1/Mtok` input, so a workspace that spent 3 Mtok has spent exactly $3.
const TABLE: &str = r#""prices":{"anthropic":{"opus":{"input":1}}}"#;

/// A world on disk: a `ui.json` at the state root and one named workspace the
/// §3.1 roster finds — the shape [`ceiling::parked`] resolves against.
struct Priced {
    dir: TempDir,
}

impl Priced {
    /// A world whose `ui.json` carries `keys`, with one workspace.
    fn new(keys: &str) -> Priced {
        let it = Priced {
            dir: tempdir().unwrap(),
        };
        std::fs::create_dir_all(it.state_root()).unwrap();
        std::fs::write(
            it.state_root().join("ui.json"),
            format!(r#"{{"v":1,{keys}}}"#),
        )
        .unwrap();
        std::fs::create_dir_all(it.workspace().join("repo.git")).unwrap();
        it
    }

    fn state_root(&self) -> PathBuf {
        self.dir.path().join("state").join("yog")
    }

    fn workspace(&self) -> PathBuf {
        self.dir
            .path()
            .join("data")
            .join("yog")
            .join("workspaces")
            .join("cobalt-gecko")
    }

    fn env(&self) -> Env {
        let at = |leaf: &str| self.dir.path().join(leaf).to_string_lossy().into_owned();
        Env::from_pairs([
            ("HOME", at("home")),
            ("XDG_STATE_HOME", at("state")),
            ("XDG_DATA_HOME", at("data")),
        ])
    }

    /// One step whose usage the table prices — `input` tokens on `opus`.
    fn spent(&self, input: u64) {
        let step = self.workspace().join("steps").join(CONV).join("001");
        std::fs::create_dir_all(&step).unwrap();
        std::fs::write(
            step.join("response.json"),
            format!(r#"{{"type":"usage","input_tokens":{input}}}"#),
        )
        .unwrap();
        std::fs::write(step.join("request.json"), r#"{"model":"opus"}"#).unwrap();
    }

    /// A world whose `steps/` **is not a tree**: the name a walk would descend
    /// into is a regular file, so a fold that tried could read nothing at all
    /// and an answer produced over this world came from the key, never from the
    /// tree. No mode bit, so it tears down and it reads the same as root.
    fn unwalkable(&self) {
        std::fs::create_dir_all(self.workspace()).unwrap();
        std::fs::write(self.workspace().join("steps"), "not a tree").unwrap();
    }
}

/// A consult over `w`'s world, parked by `sentence`.
fn parked_consult(w: &World, sentence: &str) -> Consult {
    Consult {
        ceiling: Some(sentence.to_owned()),
        ..w.consult()
    }
}

fn verdict(consult: &Consult, tool: &str, input: serde_json::Value) -> Verdict {
    adjudicate(consult, &Request::parse(&request(tool, input)).unwrap())
}

/// Under the number the seat is not there at all: every verdict is the one the
/// table, the floor and the answers would have given anyway.
#[test]
fn under_the_number_leaves_every_verdict_unchanged() {
    let w = Priced::new(&format!(r#"{TABLE},"ceiling":5"#));
    w.spent(3_000_000);
    assert_eq!(ceiling::parked(&w.env()), None, "$3 under a $5 ceiling");

    let shipped = World::new();
    let ordinary = verdict(&shipped.consult(), "bash", json!({"command": "ls"}));
    assert_eq!(ordinary, Verdict::Pass);
}

/// At the number every class holds — `read` included, which the shipped table
/// passes and the §4.9 floor exempts — and the reason is `Ceiling::verdict`'s
/// own sentence, unaltered.
#[test]
fn at_the_number_every_class_holds_with_the_ceiling_s_own_sentence() {
    let w = Priced::new(&format!(r#"{TABLE},"ceiling":2"#));
    w.spent(3_000_000);
    let sentence = ceiling::parked(&w.env()).expect("$3 is at or over a $2 ceiling");
    assert!(sentence.starts_with(HEAD), "{sentence}");
    assert!(sentence.contains("$3.00"), "{sentence}");

    let shipped = World::new();
    let consult = parked_consult(&shipped, &sentence);
    // One invocation per reach the shipped table answers differently: a read it
    // passes, an open-world reach it passes, and a loss it refuses outright.
    for input in [
        json!({"command": "ls"}),
        json!({"command": "curl https://x"}),
        json!({"command": "rm -rf /tmp/x"}),
    ] {
        assert_eq!(
            verdict(&consult, "bash", input.clone()),
            Verdict::Hold(sentence.clone()),
            "{input}"
        );
    }
    // And a name off the closed map, which the routed lane would have held with
    // a sentence of its own: the ceiling's is the one that lands.
    assert_eq!(
        verdict(&consult, "box2_fetch", json!({"url": "https://x"})),
        Verdict::Hold(sentence.clone())
    );
}

/// The one exception, and it is the precedence the floor already states: an
/// answer to *this exact* `tool_use` id is the operator looking at the call in
/// front of them, so `/answer pass` walks one call through a parked world.
#[test]
fn an_answer_pass_for_this_tool_use_id_still_walks_through() {
    let shipped = World::new();
    let consult = parked_consult(&shipped, "spend ceiling reached: …");
    assert!(matches!(
        verdict(&consult, "bash", json!({"command": "ls"})),
        Verdict::Hold(_)
    ));
    shipped.answer(&[YOG_CONTROL, "answer", "toolu_01", "pass"]);
    assert_eq!(
        verdict(&consult, "bash", json!({"command": "ls"})),
        Verdict::Pass,
        "the operator answered this id"
    );
}

/// The severability, both spellings, over a world whose `steps/` a fold could
/// not read even if it tried: an absent key and an empty table each answer from
/// [`Ceiling::armed`](crate::spend::Ceiling::armed) — one `ui.json` read, the
/// roster not enumerated and no `steps/` walk.
#[test]
fn an_absent_key_or_an_empty_table_costs_one_read_and_no_walk() {
    for (keys, why) in [
        (TABLE, "no `ceiling` key"),
        (r#""ceiling":1"#, "no `prices` table"),
    ] {
        let world = Priced::new(keys);
        world.unwalkable();
        assert_eq!(ceiling::parked(&world.env()), None, "{why}");
    }
}

/// The overshoot is **one step per live conversation**: the park lands before
/// the invocation executes, so the branch keeps its tree and every uncommitted
/// byte, and a step that ends with no tool call ends the branch on its own. The
/// seat can only ever answer `hold` — never a refusal the model steps past,
/// never a stop, and it writes nothing.
#[test]
fn the_overshoot_is_one_step_per_live_conversation_because_the_park_is_a_hold() {
    let shipped = World::new();
    let consult = parked_consult(&shipped, "spend ceiling reached: …");
    let answered = verdict(&consult, "bash", json!({"command": "rm -rf /tmp/x"}));
    assert!(
        matches!(answered, Verdict::Hold(_)),
        "a loss the table refuses still parks: {answered:?}"
    );
    assert!(
        !shipped.state().join("ops.jsonl").exists(),
        "the consult writes nothing"
    );
}
