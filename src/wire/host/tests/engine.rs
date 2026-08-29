//! **What comes back down the channel**: every way an engine can answer
//! something that is not this host's work, each named rather than guessed at. A
//! real intake cannot produce these, which is why the stand-in answerer is the
//! point — a client that could not tell them apart would loop forever against
//! an engine that had stopped making sense.

use super::*;
/// Every way an engine can answer something that is not this host's work, each
/// named rather than guessed at. The stand-in answerer is the point: a real
/// intake cannot produce these, and a client that could not tell them apart
/// would loop forever against an engine that had stopped making sense.
#[test]
fn an_answer_that_is_not_this_machines_work_names_itself() {
    struct Says(Vec<Value>);
    impl crate::wire::server::Answerer for Says {
        fn answer(
            &self,
            _peer: &crate::registry::Peer,
            _request: Value,
        ) -> Box<dyn Iterator<Item = Value>> {
            Box::new(self.0.clone().into_iter())
        }
    }

    for (said, needle) in [
        (Vec::new(), "closed the stream"),
        (
            vec![json!({"ok": true, "kind": "teleported"})],
            "undecodable",
        ),
        (vec![json!({"ok": false, "error": "no"})], "no"),
        (
            vec![json!({"ok": true, "kind": "acked"})],
            "not this machine's work",
        ),
    ] {
        let tmp = TempDir::new().expect("tmp");
        let world = provision(&tmp);
        let m = material::read(&world, material::Role::Server)
            .expect("material")
            .expect("provisioned");
        let listener =
            Listener::bind(&m, Arc::new(Says(said)), Presence::default()).expect("bound");
        fs::write(
            material::dir(&world).join(material::ADDRESS),
            listener.address(),
        )
        .expect("address");
        let e = serve(&world);
        assert!(e.contains(needle), "{e}");
    }
}

/// **A refused completion stops the host**, rather than being dropped on the
/// floor: an engine that will not take this machine's answers — an expired
/// handle, a slot addressed elsewhere — is one there is no point running
/// against. The stand-in answers each request in turn, so the failure lands on
/// the `complete` and nowhere earlier.
#[test]
fn a_completion_the_engine_refuses_stops_the_host() {
    struct InTurn {
        said: Vec<Value>,
        at: std::sync::atomic::AtomicUsize,
    }
    impl crate::wire::server::Answerer for InTurn {
        fn answer(
            &self,
            _peer: &crate::registry::Peer,
            _request: Value,
        ) -> Box<dyn Iterator<Item = Value>> {
            let at = self.at.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Box::new(std::iter::once(
                self.said
                    .get(at)
                    .cloned()
                    .unwrap_or(json!({"ok": false, "error": "nothing left to say"})),
            ))
        }
    }

    let tmp = TempDir::new().expect("tmp");
    let world = provision(&tmp);
    let engine = InTurn {
        said: vec![
            json!({"ok": true, "kind": "advertised"}),
            json!({"ok": true, "kind": "invocations",
                   "rows": [{"invocation": "inv-1", "tool": "Bash",
                             "input": {"command": "ls"}}]}),
            json!({"ok": false, "error": "no invocation \"inv-1\" is in flight"}),
        ],
        at: std::sync::atomic::AtomicUsize::new(0),
    };
    let m = material::read(&world, material::Role::Server)
        .expect("material")
        .expect("provisioned");
    let listener = Listener::bind(&m, Arc::new(engine), Presence::default()).expect("bound");
    fs::write(
        material::dir(&world).join(material::ADDRESS),
        listener.address(),
    )
    .expect("address");

    let stopped = serve(&world);
    assert!(
        stopped.contains("inv-1"),
        "the refusal rides back: {stopped}"
    );
}
