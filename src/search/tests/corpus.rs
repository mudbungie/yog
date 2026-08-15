//! What the query spans (§8.5): the snapshot half, the disk half, one row
//! per address however often it matches, the gaps it names, and the disk half
//! an abandoned run never pays for.

use super::*;

#[test]
fn one_query_spans_balls_workspaces_conversations_and_transcripts() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("kraken");
    write(
        &ws.join("agents").join(AGENT).join("goal.md"),
        b"raise the kraken",
    );
    write(
        &ws.join("agents")
            .join(AGENT)
            .join("messages")
            .join("001-user.md"),
        b"the kraken stirred",
    );
    let snap = world(
        &ws,
        vec![agent(AGENT, Some("deep-one"))],
        vec![ball("bl-kraken", "wake it", "body")],
        vec![ball("bl-dead", "kraken, closed", "closed body")],
    );
    let found = run(&snap, "kraken", &always());
    let at: Vec<&Address> = found.hits.iter().map(|h| &h.at).collect();
    assert!(at.contains(&&Address::Ball {
        project: PathBuf::from("/proj"),
        id: "bl-kraken".to_owned()
    }));
    assert!(at.contains(&&Address::Ball {
        project: PathBuf::from("/proj"),
        id: "bl-dead".to_owned()
    }));
    assert!(at.contains(&&Address::Workspace { path: ws.clone() }));
    assert!(at.contains(&&Address::Conversation {
        workspace: ws.clone(),
        agent: AGENT.to_owned()
    }));
    assert_eq!(found.unreadable, Vec::<String>::new());
}

#[test]
fn the_goal_and_the_transcript_are_the_conversations_two_disk_fields() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("kraken");
    write(
        &ws.join("agents").join(AGENT).join("goal.md"),
        b"find abyss",
    );
    let snap = world(&ws, vec![agent(AGENT, None)], vec![], vec![]);
    let goal = run(&snap, "abyss", &always());
    assert_eq!(goal.hits.len(), 1);
    assert_eq!(goal.hits[0].field, Field::Summary);

    write(
        &ws.join("agents")
            .join(AGENT)
            .join("messages")
            .join("001-user.md"),
        b"the abyss gazes back",
    );
    let both = run(&snap, "gazes", &always());
    assert_eq!(both.hits.len(), 1);
    assert_eq!(both.hits[0].field, Field::Text);
    assert_eq!(both.hits[0].excerpt, "the abyss gazes back");
}

#[test]
fn a_conversation_matching_many_times_is_still_one_row_at_its_best_field() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("shoggoth");
    for n in 1..=3 {
        write(
            &ws.join("agents")
                .join(AGENT)
                .join("messages")
                .join(format!("00{n}-user.md")),
            b"tekeli-li tekeli-li",
        );
    }
    let snap = world(&ws, vec![agent(AGENT, Some("tekeli-li"))], vec![], vec![]);
    let found = run(&snap, "tekeli-li", &always());
    assert_eq!(found.hits.len(), 1, "one hit per address: {found:?}");
    // The name is what it *is*, so it outranks the three transcript mentions.
    assert_eq!(found.hits[0].field, Field::Name);
}

#[test]
fn unreadable_sources_are_named_and_the_rest_of_the_world_still_answers() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("kraken");
    let derived = dir.path().join("derived");
    // A goal that exists but cannot be read: a directory where a file belongs.
    std::fs::create_dir_all(ws.join("agents").join(AGENT).join("goal.md")).unwrap();
    write(
        &ws.join("agents")
            .join(AGENT)
            .join("messages")
            .join("001-user.md"),
        b"kraken rises",
    );
    let mut snap = world(&ws, vec![agent(AGENT, None)], vec![], vec![]);
    // A second workspace whose tree failed to derive, and an orphaned project.
    snap.workspaces.push(Workspace {
        path: derived.clone(),
        kind: WorkspaceKind::Foreign,
    });
    snap.join_rows.push(JoinRow {
        project: "gone".to_owned(),
        ball_id: String::new(),
        state: JoinState::OrphanedProject,
        workspace: None,
        claimant: None,
        title: None,
    });
    let found = run(&snap, "kraken", &always());
    assert_eq!(
        found.hits.len(),
        2,
        "workspace name + transcript: {found:?}"
    );
    assert!(found.unreadable.iter().any(|u| u.contains("goal.md")));
    assert!(
        found
            .unreadable
            .iter()
            .any(|u| u.ends_with(": no derived tree") && u.contains("derived"))
    );
    assert!(
        found
            .unreadable
            .iter()
            .any(|u| u == "gone: balls unlistable")
    );
    let mut sorted = found.unreadable.clone();
    sorted.sort();
    assert_eq!(found.unreadable, sorted, "reported in a fixed order");
}

#[test]
fn an_unwanted_search_abandons_before_reading_the_conversations() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("kraken");
    write(
        &ws.join("agents")
            .join(AGENT)
            .join("messages")
            .join("001-user.md"),
        b"kraken",
    );
    let snap = world(
        &ws,
        vec![agent(AGENT, None)],
        vec![ball("bl-kraken", "t", "body")],
        vec![],
    );
    let found = run(&snap, "kraken", &|| false);
    // The snapshot half is already in hand and still answers — the ball and the
    // workspace whose §3.1 name is "kraken". The disk half is what abandonment
    // skips, so the conversation whose transcript says it is absent.
    assert!(
        !found
            .hits
            .iter()
            .any(|h| matches!(h.at, Address::Conversation { .. })),
        "{found:?}"
    );
    assert_eq!(found.hits.len(), 2, "{found:?}");
}
