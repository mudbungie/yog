//! Authoring the control onto a workspace's `config/default`.

use super::*;
use tempfile::tempdir;

const SHIPPED: &str =
    "events:\n  user_message:\n    - dispatch(worker)\n\nbudgets:\n  max_depth: 4\n";

fn shim() -> PathBuf {
    PathBuf::from("/data/yog/world/tools/tool-control")
}

#[test]
fn authoring_appends_the_block_and_keeps_every_other_default() {
    let out = authored(SHIPPED, &shim());
    assert!(out.contains("events:"), "{out}");
    assert!(out.contains("max_depth: 4"), "{out}");
    assert!(
        out.contains("tool_control:\n  command: /data/yog/world/tools/tool-control\n"),
        "{out}"
    );
}

#[test]
fn authoring_is_a_fixed_point_which_is_the_whole_convergence_test() {
    let once = authored(SHIPPED, &shim());
    assert_eq!(authored(&once, &shim()), once);
}

#[test]
fn a_block_naming_another_shim_is_replaced_not_duplicated() {
    let stale = authored(SHIPPED, Path::new("/old/yog/tools/tool-control"));
    let fresh = authored(&stale, &shim());
    assert_eq!(fresh.matches("tool_control:").count(), 1, "{fresh}");
    assert!(!fresh.contains("/old/yog"), "{fresh}");
    assert!(fresh.contains("max_depth: 4"), "{fresh}");
}

#[test]
fn a_top_level_key_after_the_block_survives_its_removal() {
    let base = "tool_control:\n  command: /old\n  extra: x\nbudgets:\n  max_depth: 4\n";
    let out = authored(base, &shim());
    assert!(out.contains("max_depth: 4"), "{out}");
    assert!(!out.contains("/old"), "{out}");
    assert!(!out.contains("extra: x"), "{out}");
}

#[test]
fn a_workspace_with_no_config_commit_authors_nothing() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();
    assert_eq!(committed(&ws), None);
    let entry = ensure_controlled(
        &crate::cli_outbound::Cli::new("/no/lernie"),
        &ws,
        &shim(),
        Path::new("/no/yog"),
        dir.path(),
        "TS",
        Origin::Balls,
    )
    .unwrap();
    assert!(entry.is_none(), "nothing to author onto is not an error");
}

/// A fake `lernie` at `dir/lernie` exiting `code`, recording its argv and the
/// two environment variables the scripted-editor drive hands it.
fn fake_lernie(dir: &Path, code: i32) -> PathBuf {
    let path = dir.join("lernie");
    let log = dir.join("argv");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{log}'\nprintf '%s\\n' \"$EDITOR\" \
             \"$YOG_EDIT_SRC\" >> '{log}'\nprintf 'boom\\n' 1>&2\nexit {code}\n",
            log = log.display(),
        ),
    )
    .unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    path
}

#[test]
fn a_workspace_whose_tip_lacks_the_block_is_driven_through_lernie_config() {
    let _g = crate::test_support::spawn_guard();
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_workflow(&ws, SHIPPED);
    assert_eq!(committed(&ws).as_deref(), Some(SHIPPED));
    let lernie = crate::cli_outbound::Cli::new(fake_lernie(dir.path(), 0));
    let entry = ensure_controlled(
        &lernie,
        &ws,
        &shim(),
        Path::new("/opt/yog/bin/yog"),
        dir.path(),
        "TS",
        Origin::Balls,
    )
    .unwrap()
    .expect("a tip without the block is authored");
    assert_eq!(entry.exit, 0);
    // The drive is `lernie config <ws> default` — the one lawful writer of
    // `config/*` — with the `$EDITOR` shim and the staging dir.
    let logged = std::fs::read_to_string(dir.path().join("argv")).unwrap();
    let lines: Vec<&str> = logged.lines().collect();
    assert_eq!(lines[0], "config");
    assert_eq!(lines[1], ws.display().to_string());
    assert_eq!(lines[2], "default");
    assert!(lines[3].contains("--editor-apply"), "{logged}");
    // The staged file is the WHOLE workflow: a fragment would truncate policy.
    let staged = PathBuf::from(lines[4]).join("workflow.yaml");
    let text = std::fs::read_to_string(staged).unwrap();
    assert!(text.contains("events:"), "{text}");
    assert!(
        text.contains("command: /data/yog/world/tools/tool-control"),
        "{text}"
    );
}

#[test]
fn a_tip_that_already_names_this_shim_spawns_nothing() {
    let _g = crate::test_support::spawn_guard();
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_workflow(&ws, &authored(SHIPPED, &shim()));
    // A `lernie` that would fail if it ran at all — the steady state reads one
    // file out of git and spawns nothing.
    let lernie = crate::cli_outbound::Cli::new("/no/lernie");
    assert!(
        ensure_controlled(
            &lernie,
            &ws,
            &shim(),
            Path::new("/no/yog"),
            dir.path(),
            "TS",
            Origin::Balls,
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn a_failed_drive_rides_back_as_its_own_ops_entry() {
    let _g = crate::test_support::spawn_guard();
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_workflow(&ws, SHIPPED);
    let lernie = crate::cli_outbound::Cli::new(fake_lernie(dir.path(), 3));
    let entry = ensure_controlled(
        &lernie,
        &ws,
        &shim(),
        Path::new("/no/yog"),
        dir.path(),
        "TS",
        Origin::Balls,
    )
    .unwrap()
    .unwrap();
    assert_eq!(entry.exit, 3);
    assert!(entry.stderr.contains("boom"), "{}", entry.stderr);
}
