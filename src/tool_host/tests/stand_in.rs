//! **The stand-in substrate** (REMOTE §5, bl-c907): what the tool-host beats
//! drive against instead of a live engine.
//!
//! Every fixture here holds up exactly one contract — the deposit consumer's,
//! *claim then reply* — because that contract is the whole of what the driver's
//! ask depends on. Nothing here asserts; it is the world the assertions happen
//! in, and it lives in its own file because four sibling modules ask for it.

use super::*;

/// A stand-in engine: answer the first deposit that lands with `reply`, hand
/// the request back over a channel, and stop. It is the deposit consumer's
/// contract and nothing else — claim, then reply — because that contract is
/// the whole of what the driver's ask depends on.
pub(in crate::tool_host) fn engine(
    root: &Path,
    reply: &Value,
) -> (JoinHandle<()>, Receiver<Value>) {
    let root = root.to_path_buf();
    let reply = reply.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        for _ in 0..4000 {
            if let Some((id, path)) = deposit::pending(&root).into_iter().next() {
                let request = std::fs::read(&path)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
                    .unwrap_or(Value::Null);
                let _ = deposit::claim(&root, &id);
                let _ = deposit::write_reply(&root, &id, &reply);
                let _ = tx.send(request);
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    (handle, rx)
}

/// A budget that answers fast when an engine is there and gives up fast when
/// it is not.
pub(in crate::tool_host) fn budget() -> ask::Budget {
    ask::Budget {
        waits: 4000,
        tick: Duration::from_millis(1),
    }
}

/// A budget with no patience at all — the "no engine" path, without the wait.
pub(in crate::tool_host) fn impatient() -> ask::Budget {
    ask::Budget {
        waits: 1,
        tick: Duration::ZERO,
    }
}

pub(in crate::tool_host) fn tool(name: &str) -> Tool {
    Tool {
        name: name.to_owned(),
        description: format!("what {name} does"),
        input_schema: json!({"type": "object"}),
    }
}

pub(in crate::tool_host) fn site(root: &Path, budget: ask::Budget) -> Site {
    Site {
        state_root: root.to_path_buf(),
        workspace: "home".to_owned(),
        agent: "dulcet-mongoose".to_owned(),
        budget,
        patience: budget,
        clock: FakeClock::new().arc(),
    }
}

/// A stand-in engine that answers `replies` in order, one per deposit — the
/// routing leg needs two round trips (queue, then poll) and [`engine`] answers
/// exactly one. Hands every request back over the channel.
pub(in crate::tool_host) fn scripted(
    root: &Path,
    replies: &[Value],
) -> (JoinHandle<()>, Receiver<Value>) {
    let root = root.to_path_buf();
    let replies = replies.to_vec();
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let mut said = 0;
        for _ in 0..40_000 {
            let Some(reply) = replies.get(said) else {
                return;
            };
            if let Some((id, path)) = deposit::pending(&root).into_iter().next() {
                let request = std::fs::read(&path)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
                    .unwrap_or(Value::Null);
                let _ = deposit::claim(&root, &id);
                let _ = deposit::write_reply(&root, &id, reply);
                let _ = tx.send(request);
                said += 1;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    (handle, rx)
}
