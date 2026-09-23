//! The hold mark: the shape litany writes, read back, and every way a mark can
//! fail to say anything reading as "nothing is parked".

use super::*;
use tempfile::tempdir;

#[test]
fn the_shape_litany_writes_reads_back_whole() {
    let held = parse(
        r#"{"tool_use_id":"toolu_01","tool":"bash","reason":"bash {\"command\":\"curl x\"} classified open-world (no rule classifies `curl`)"}"#,
    )
    .expect("the mark's own shape");
    assert_eq!(held.tool_use_id, "toolu_01");
    assert_eq!(held.tool, "bash");
    assert!(held.reason.contains("open-world"));
}

#[test]
fn a_mark_with_no_reason_is_still_a_park() {
    // The id is what the answer is scoped to; a reason yog cannot read costs
    // the operator a sentence, never the ability to release the branch.
    let held = parse(r#"{"tool_use_id":"t","tool":"cd"}"#).expect("id and tool are enough");
    assert_eq!(held.reason, "");
}

#[test]
fn anything_that_is_not_the_shape_is_no_park_at_all() {
    for blob in [
        "",
        "not json",
        "[]",
        r#"{"tool":"bash","reason":"r"}"#,
        r#"{"tool_use_id":"t","reason":"r"}"#,
        r#"{"tool_use_id":7,"tool":"bash"}"#,
    ] {
        assert_eq!(parse(blob), None, "{blob:?}");
    }
}

#[test]
fn a_workspace_with_no_repo_holds_nothing() {
    let dir = tempdir().unwrap();
    assert_eq!(read(dir.path(), "a-1"), None);
}

/// A bare workspace repo with `agent`'s hold mark pointing at `value`.
fn parked(workspace: &std::path::Path, agent: &str, value: &str) {
    let repo = workspace.join("repo.git");
    let git = |args: &[&str]| {
        crate::git_env::output(crate::git_env::git().arg("--git-dir").arg(&repo).args(args))
            .expect("git runs")
    };
    std::fs::create_dir_all(&repo).unwrap();
    git(&["init", "--bare", "-q"]);
    let staged = workspace.join("mark.json");
    std::fs::write(&staged, value).unwrap();
    let hashed = git(&["hash-object", "-w", "--", &staged.to_string_lossy()]);
    let oid = String::from_utf8_lossy(&hashed.stdout).trim().to_owned();
    git(&["update-ref", &format!("{HELD_PREFIX}{agent}"), &oid]);
}

#[test]
fn a_real_mark_reads_off_the_workspace_repo() {
    let dir = tempdir().unwrap();
    parked(
        dir.path(),
        "a-1",
        r#"{"tool_use_id":"toolu_9","tool":"bash","reason":"why"}"#,
    );
    let held = read(dir.path(), "a-1").expect("the mark is there");
    assert_eq!(held.tool_use_id, "toolu_9");
    // Another agent in the same repo wears nothing.
    assert_eq!(read(dir.path(), "a-2"), None);
}

#[test]
fn an_unparseable_mark_reads_as_no_park() {
    let dir = tempdir().unwrap();
    parked(dir.path(), "a-1", "not json at all");
    assert_eq!(read(dir.path(), "a-1"), None);
}

/// The whole namespace at once (bl-53d1): every mark with the agent it parks,
/// an unreadable blob skipped rather than forged, and a workspace with no
/// repo holding nothing.
#[test]
fn every_mark_in_the_workspace_reads_off_the_namespace_at_once() {
    let dir = tempdir().unwrap();
    assert!(all(dir.path()).is_empty(), "no repo, nothing parked");
    parked(
        dir.path(),
        "a-1",
        r#"{"tool_use_id":"t1","tool":"bash","reason":"spend ceiling reached: x"}"#,
    );
    parked(
        dir.path(),
        "a-2",
        r#"{"tool_use_id":"t2","tool":"cd","reason":"floor"}"#,
    );
    parked(dir.path(), "a-3", "not a mark");
    let marks = all(dir.path());
    assert_eq!(
        marks
            .iter()
            .map(|(agent, held)| (agent.as_str(), held.tool_use_id.as_str()))
            .collect::<Vec<_>>(),
        [("a-1", "t1"), ("a-2", "t2")]
    );
}

#[test]
fn the_ref_namespace_is_litanys_own() {
    assert_eq!(HELD_PREFIX, "refs/litany/held/");
}
