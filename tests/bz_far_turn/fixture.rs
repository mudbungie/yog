//! The fixtures of `bz_far_turn`: the hermetic binary, the canonical stream a
//! far `bz` writes, the stand-in intake over the real mailbox, and the foot.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

#[path = "../support/write_exec.rs"]
mod write_exec;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use yog::boundary::deposit;
use yog::boundary::reply::{self, Reply};
use yog::registry::mailbox::stream::offsets_of;
use yog::registry::mailbox::{Call, Capture, Mailbox, Piece, Post, Standing};
use yog::registry::{Client, tools};

pub const FOOT: &str = "foot";

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn hermetic(home: &Path) -> Vec<(String, PathBuf)> {
    ["XDG_DATA_HOME", "XDG_STATE_HOME", "XDG_CONFIG_HOME", "HOME"]
        .into_iter()
        .map(|k| (k.to_owned(), home.join(k.to_lowercase())))
        .chain([
            ("GIT_CONFIG_GLOBAL".to_owned(), home.join("gitconfig")),
            ("GIT_CONFIG_SYSTEM".to_owned(), PathBuf::from("/dev/null")),
        ])
        .collect()
}

pub fn yog(home: &Path, extra: &[(&str, &Path)]) -> std::process::Command {
    let mut command = yog::git_env::command(Path::new(env!("CARGO_BIN_EXE_yog")));
    command.envs(hermetic(home));
    command.envs(extra.iter().copied());
    command
}

/// The canonical stream the fixture's `bz` writes, in brazen's own spelling.
pub fn events() -> Vec<String> {
    let mut usage = brazen::Usage::default();
    (usage.input_tokens, usage.output_tokens) = (Some(11), Some(7));
    [
        brazen::Event::message_start(None, None, brazen::Role::Assistant),
        brazen::Event::ContentStart {
            index: 0,
            kind: brazen::ContentKind::Text {},
        },
        brazen::Event::ContentDelta {
            index: 0,
            delta: brazen::Delta::TextDelta("from the far machine".to_owned()),
        },
        brazen::Event::ContentStop { index: 0 },
        brazen::Event::Usage(usage),
        brazen::Event::Finish {
            reason: brazen::FinishReason::Stop,
        },
        brazen::Event::End,
    ]
    .iter()
    .map(|event| serde_json::to_string(event).unwrap())
    .collect()
}

/// The server's intake for the arm's two gestures, over the real mailbox.
pub fn answer(mail: &Mailbox, request: &Value) -> Value {
    let standing = |invocation: String, standing: Result<Standing, String>| match standing {
        Ok(standing) => reply::encode(&Reply::Routed {
            invocation,
            standing,
        }),
        Err(refusal) => reply::refusal(&refusal),
    };
    if request["op"] == "invoke" {
        let call = Call {
            client: request["client"].as_str().unwrap().to_owned(),
            tool: request["tool"].as_str().unwrap().to_owned(),
            input: request["input"].clone(),
            cwd: None,
        };
        return standing(mail.post(now(), "local", &call), Ok(Standing::default()));
    }
    let id = request["invocation"].as_str().unwrap().to_owned();
    let since = offsets_of(&request["since"]).unwrap();
    let read = mail.collect(now(), "local", &id, Some(since));
    standing(id, read)
}

/// The fixture foot: take this machine's work, run its `bz` entry, post each
/// line the child writes as it is written, then the capture that ends the run.
pub fn serve(mail: &Mailbox, script: &Path, asked: &std::sync::Mutex<Vec<Value>>) {
    for invocation in mail.take(FOOT).unwrap() {
        asked.lock().unwrap().push(invocation.input.clone());
        let mut child = yog::git_env::command(script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        for line in BufReader::new(child.stdout.take().unwrap()).lines() {
            let piece = Piece {
                stdout: format!("{}\n", line.unwrap()),
                stderr: String::new(),
            };
            mail.complete(now(), FOOT, &invocation.id, &Post::Partial(piece))
                .unwrap();
        }
        let capture = Capture {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: child.wait().unwrap().code().unwrap(),
        };
        mail.complete(now(), FOOT, &invocation.id, &Post::Capture(capture))
            .unwrap();
    }
}

/// Every step's `response.json` under the workspace, with its size.
pub fn responses(ws: &Path) -> Vec<(PathBuf, u64)> {
    let Ok(agents) = std::fs::read_dir(ws.join("steps")) else {
        return Vec::new();
    };
    agents
        .flatten()
        .map(|agent| agent.path().join("001/response.json"))
        .filter_map(|path| Some((path.clone(), std::fs::metadata(&path).ok()?.len())))
        .collect()
}

/// Run the stand-in intake until `done`: every deposit answered out of `mail`,
/// every `capture` poll counted.
pub fn intake(
    mail: &Mailbox,
    root: &Path,
    done: &Arc<AtomicBool>,
    polls: &Arc<AtomicUsize>,
) -> std::thread::JoinHandle<()> {
    let (mail, root) = (mail.clone(), root.to_path_buf());
    let (done, polls) = (Arc::clone(done), Arc::clone(polls));
    std::thread::spawn(move || {
        while !done.load(Ordering::Relaxed) {
            for (id, path) in deposit::pending(&root) {
                let request: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                polls.fetch_add(
                    usize::from(request.get("op") == Some(&json!("capture"))),
                    Ordering::Relaxed,
                );
                let _claim = deposit::claim(&root, &id);
                deposit::write_reply(&root, &id, &answer(&mail, &request)).unwrap();
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })
}

/// Register the foot in workspace `ws` under `state_root`, advertising `bz`.
pub fn seat_foot(state_root: &Path) {
    let foot = Client::parse(FOOT).unwrap();
    yog::registry::register(state_root, &foot, "ws").unwrap();
    let advertised = tools::Tool {
        name: "bz".to_owned(),
        description: "this box's model access".to_owned(),
        input_schema: json!({"type": "object"}),
        subject_cwd: false,
    };
    tools::store(state_root, &foot, &[advertised]).unwrap();
}

/// Mint a workspace under `home` whose every role names a row on the foot,
/// through litany's own `new` and `config` — the `$EDITOR` hand-off included.
pub fn workspace(home: &Path) -> PathBuf {
    std::fs::write(
        home.join("gitconfig"),
        "[user]\n\tname = Fixture\n\temail = fixture@example.com\n",
    )
    .unwrap();
    let ws = home.join("ws");
    let ws_s = ws.display().to_string();
    let editor = home.join("editor");
    write_exec::write_exec(
        &editor,
        "#!/bin/sh\nsed -i 's/^\\( *provider:\\).*/\\1 fix@foot/' \"$1/providers.yaml\"\n",
    );
    for (verb, out) in [("new", Stdio::null()), ("config", Stdio::inherit())] {
        let status = yog(home, &[("EDITOR", &editor)])
            .args(["litany", verb, &ws_s])
            .stdout(out)
            .status()
            .unwrap();
        assert!(status.success(), "litany {verb}");
    }
    ws
}

/// The foot's `bz` entry: four events, a wait for `release`, then the rest.
pub fn far_bz(home: &Path, lines: &[String], release: &Path) -> PathBuf {
    let (head, tail) = lines.split_at(4);
    let script = home.join("far-bz");
    write_exec::write_exec(
        &script,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' '{}'\nwhile [ ! -e '{}' ]; do sleep 0.05; done\n\
             printf '%s\\n' '{}'\n",
            head.join("' '"),
            release.display(),
            tail.join("' '"),
        ),
    );
    script
}
