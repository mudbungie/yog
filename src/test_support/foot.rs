//! **A registered machine serving `bz`, for a test** (REMOTE §5.7): the
//! registration and advertisement files a server reads a qualified provider
//! name against, and a thread that plays the foot over a real
//! [`Mailbox`] — it parks on the follow-class read, is handed each call, and
//! posts the capture it was scripted with.
//!
//! Nothing here stands in for the mailbox: `serving`, the hand-off and the
//! post are the production ones, so a read that asks a machine is tested
//! against the predicate it will meet.

use std::path::Path;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::{Value, json};

use crate::registry::mailbox::{Capture, Mailbox, Post};
use crate::registry::{Client, tools};

/// A mailbox whose follow-class read holds two seconds, a millisecond a look.
pub(crate) fn mailbox() -> Mailbox {
    Mailbox::holding(2000, Duration::from_millis(1))
}

/// Register `client` in `workspace`, advertising each of `tools`.
pub(crate) fn seat(state_root: &Path, client: &str, workspace: &str, advertised: &[&str]) {
    let client = Client::parse(client).expect("client");
    crate::registry::register(state_root, &client, workspace).expect("register");
    let advertised: Vec<tools::Tool> = advertised
        .iter()
        .map(|name| tools::Tool {
            name: (*name).to_owned(),
            description: "this box's model access".to_owned(),
            input_schema: json!({"type": "object"}),
            subject_cwd: false,
        })
        .collect();
    tools::store(state_root, &client, &advertised).expect("advertise");
}

/// A capture a far `bz` exited 0 with.
pub(crate) fn printed(stdout: &str) -> Capture {
    Capture {
        stdout: stdout.to_owned(),
        stderr: String::new(),
        exit_code: 0,
    }
}

/// Play `client`'s foot for `script.len()` calls: each is taken off the real
/// read, its input sent down the channel, and answered with the scripted
/// capture — or, for a `None`, taken and never answered. The thread ends when
/// the script does, which is when the machine stops serving.
pub(crate) fn serve(
    mail: &Mailbox,
    client: &str,
    script: Vec<Option<Capture>>,
) -> (JoinHandle<()>, mpsc::Receiver<Value>) {
    let (mail, client) = (mail.clone(), client.to_owned());
    let (tx, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        for answer in script {
            let call = (0..30)
                .find_map(|_| mail.take(&client).expect("one reader").work().pop())
                .expect("the foot was never handed its call");
            tx.send(call.input).expect("the test is listening");
            if let Some(capture) = answer {
                mail.complete(1, &client, &call.id, &Post::Capture(capture))
                    .expect("posted");
            }
        }
    });
    (handle, rx)
}

/// Wait until `client` is parked on its read — the moment a listing would
/// reach it.
pub(crate) fn parked(mail: &Mailbox, client: &str) {
    for _ in 0..5000 {
        if mail.serving(client) {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("{client} never parked on its read");
}
