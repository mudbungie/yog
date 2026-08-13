//! The consult end to end: the wire contract, the writable root a real
//! workspace resolves to, and the verdict a `world/tools/` shim would print.

use super::*;
use crate::opslog::{OpEntry, Origin, YOG_CONTROL, append};
use serde_json::json;
use std::io::Write;
use tempfile::{TempDir, tempdir};

/// A world on disk: a workspace named like a yog workspace, a state root
/// carrying `ops.jsonl`, and a balls state root the delivery formula mirrors.
struct World {
    dir: TempDir,
}

impl World {
    fn new() -> World {
        World {
            dir: tempdir().unwrap(),
        }
    }

    fn workspace(&self) -> PathBuf {
        self.dir.path().join("workspaces").join("cobalt-gecko")
    }

    fn state(&self) -> PathBuf {
        self.dir.path().join("state").join("yog")
    }

    fn balls(&self) -> PathBuf {
        self.dir.path().join("state").join("balls")
    }

    fn consult(&self) -> Consult {
        Consult {
            workspace: self.workspace(),
            balls_state_root: self.balls(),
            state_root: self.state(),
            home: self.dir.path().join("home"),
            cwd: None,
            policy: crate::control::policy::Policy::default(),
        }
    }

    /// Log a `bl claim` row exactly as the start flow writes it.
    fn claim(&self, project: &str, id: &str, claimant: &str) {
        self.row(
            &["bl", "claim", id, "--as", claimant],
            project,
            Origin::Balls,
        );
    }

    /// Log one capability-answer row (what bl-765d's boundary action writes).
    fn answer(&self, words: &[&str]) {
        self.row(words, "", Origin::World);
    }

    /// Commit `capability.yaml` onto `config/default` — the workspace's
    /// standing policy, at the tip the control reads it from.
    fn policy(&self, text: &str) {
        let repo = self.workspace().join("repo.git");
        std::fs::create_dir_all(&repo).unwrap();
        let git = |args: &[&str]| {
            let out = crate::git_env::git()
                .arg("--git-dir")
                .arg(&repo)
                .args(args)
                .env("GIT_AUTHOR_DATE", "2026-08-04T00:00:00Z")
                .env("GIT_COMMITTER_DATE", "2026-08-04T00:00:00Z")
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_SYSTEM", "/dev/null")
                .output()
                .unwrap();
            assert!(out.status.success(), "git {args:?}");
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        };
        git(&["init", "--bare", "-q"]);
        let staged = self
            .dir
            .path()
            .join(crate::control::policy::CAPABILITY_YAML);
        std::fs::write(&staged, text).unwrap();
        let blob = git(&["hash-object", "-w", "--", &staged.to_string_lossy()]);
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "100644",
            &blob,
            crate::control::policy::CAPABILITY_YAML,
        ]);
        let tree = git(&["write-tree"]);
        let commit = git(&["commit-tree", &tree, "-m", "policy"]);
        git(&["update-ref", "refs/heads/config/default", &commit]);
    }

    fn row(&self, words: &[&str], cwd: &str, origin: Origin) {
        append(
            &self.state(),
            &OpEntry {
                ts: "TS".to_owned(),
                argv: words.iter().map(|s| (*s).to_owned()).collect(),
                cwd: cwd.to_owned(),
                exit: 0,
                stdout: String::new(),
                stderr: String::new(),
                origin,
            },
        )
        .unwrap();
    }
}

/// A stdout that refuses every write — the closed-pipe half of failing closed.
struct Closed;

impl std::io::Write for Closed {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("closed"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// One request as lernie's seam serializes it.
fn request(name: &str, input: serde_json::Value) -> String {
    json!({
        "id": "toolu_01",
        "name": name,
        "input": input,
        "role": "worker",
        "agent_id": "amber",
    })
    .to_string()
}

#[test]
fn the_request_parse_requires_every_field_and_ignores_the_rest() {
    let r = Request::parse(&request("bash", json!({"command": "ls"}))).unwrap();
    assert_eq!(r.id, "toolu_01");
    assert_eq!(r.name, "bash");
    assert_eq!(r.role, "worker");
    assert_eq!(r.agent_id, "amber");
    assert_eq!(r.field("command"), "ls");
    // Absent or non-string fields of `input` read as empty rather than failing:
    // an off-schema input is an invocation with no operands, not a panic.
    assert_eq!(r.field("path"), "");
    // A field lernie adds later must not brick every tool call.
    let extra = json!({"id":"i","name":"n","input":{},"role":"r","agent_id":"a","sandbox":true});
    assert!(Request::parse(&extra.to_string()).is_some());
    // Each required field, missing in turn.
    for key in ["id", "name", "input", "role", "agent_id"] {
        let mut value = json!({"id":"i","name":"n","input":{},"role":"r","agent_id":"a"});
        value.as_object_mut().unwrap().remove(key);
        assert!(Request::parse(&value.to_string()).is_none(), "{key}");
    }
    assert!(Request::parse("not json").is_none());
}

#[test]
fn a_pass_carries_no_reason_and_the_other_two_require_one() {
    // lernie's own parser rejects `{"verdict":"pass","reason":…}`.
    assert_eq!(Verdict::Pass.json(), r#"{"verdict":"pass"}"#);
    assert_eq!(
        Verdict::Hold("why".to_owned()).json(),
        r#"{"reason":"why","verdict":"hold"}"#
    );
    assert_eq!(
        Verdict::Refuse("why".to_owned()).json(),
        r#"{"reason":"why","verdict":"refuse"}"#
    );
}

#[test]
fn the_writable_root_is_the_agent_worktree_plus_the_claim_this_workspace_made() {
    let w = World::new();
    w.claim("/dev/proj", "bl-1a2b", "cobalt-gecko");
    let entries = opslog::tail(&w.state(), usize::MAX);
    let root = w.consult().root("amber", &entries);
    assert_eq!(root.cwd, w.workspace().join("agents").join("amber"));
    assert!(root.holds(&w.workspace().join("agents/amber/src/x")));
    assert!(
        root.holds(
            &w.balls()
                .join("plugins/bl-delivery/dev/proj/bl-1a2b/README.md")
        )
    );
    assert!(!root.holds(Path::new("/etc/hosts")));
    // Another workspace's claim is not this workspace's root.
    w.claim("/dev/proj", "bl-9999", "other-name");
    let entries = opslog::tail(&w.state(), usize::MAX);
    let root = w.consult().root("amber", &entries);
    assert!(!root.holds(&w.balls().join("plugins/bl-delivery/dev/proj/bl-9999/x")));
}

#[test]
fn the_default_table_decides_an_unanswered_invocation() {
    let w = World::new();
    let c = w.consult();
    let verdict = |name: &str, input: serde_json::Value| {
        adjudicate(&c, &Request::parse(&request(name, input)).unwrap())
    };
    assert_eq!(
        verdict("bash", json!({"command": "cargo test"})),
        Verdict::Pass
    );
    assert_eq!(verdict("read_file", json!({"path": "x"})), Verdict::Pass);
    assert_eq!(
        verdict("dispatch", json!({"role": "worker", "goal": "g"})),
        Verdict::Pass
    );
    // Leaving the world is the job too: the shipped table parks nothing.
    assert_eq!(
        verdict("bash", json!({"command": "curl https://x"})),
        Verdict::Pass
    );
    let Verdict::Refuse(reason) = verdict("bash", json!({"command": "rm -rf /etc"})) else {
        panic!("loss is declined in band");
    };
    assert!(reason.contains("destructive"), "{reason}");
    assert!(reason.contains("bash"), "{reason}");
}

#[test]
fn the_operator_s_answers_fold_over_the_table() {
    let w = World::new();
    // A once-answer to this exact tool_use id releases what the table declined …
    w.answer(&[YOG_CONTROL, "answer", "toolu_01", "pass"]);
    assert_eq!(
        adjudicate(
            &w.consult(),
            &Request::parse(&request("bash", json!({"command": "rm -rf /etc"}))).unwrap()
        ),
        Verdict::Pass
    );
    // … and a floor over the conversation holds everything above read.
    w.answer(&[YOG_CONTROL, "floor", "amber", "raise"]);
    let held = adjudicate(
        &w.consult(),
        &Request::parse(
            &json!({"id":"toolu_02","name":"bash","input":{"command":"cargo test"},
                    "role":"worker","agent_id":"amber"})
            .to_string(),
        )
        .unwrap(),
    );
    assert!(matches!(held, Verdict::Hold(_)), "{held:?}");
}

#[test]
fn a_reason_never_hands_the_reader_a_section_number() {
    let w = World::new();
    let verdict = adjudicate(
        &w.consult(),
        &Request::parse(&request("bash", json!({"command": "rm -rf /etc"}))).unwrap(),
    );
    let Verdict::Refuse(reason) = verdict else {
        panic!("loss is declined in band");
    };
    assert!(!reason.contains('§'), "{reason}");
}

/// The process body — the `world/tools/` shim's own stdin/stdout contract —
/// and the workspace's own standing policy, split from the pure consult at
/// §12's cap along the seam the ruling already draws: this file is *what the
/// control decides*, those are *how it is run* and *what one workspace tells
/// it to be*.
mod policy;
mod shim;

#[test]
fn the_workspace_is_lernie_s_own_env_var_else_the_cwd_it_runs_in() {
    let env = crate::xdg::Env::from_pairs([("LERNIE_CONV_REPO", "/w/ws")]);
    assert_eq!(workspace_of(&env), PathBuf::from("/w/ws"));
    let env = crate::xdg::Env::from_pairs([("LERNIE_CONV_REPO", "")]);
    assert_eq!(workspace_of(&env), std::env::current_dir().unwrap());
}
