use super::*;

fn ball(id: &str) -> Ball {
    Ball {
        id: id.to_owned(),
        title: format!("t-{id}"),
        body: String::new(),
        claimant: None,
        blockers: Vec::new(),
        parent: None,
        priority: 0,
        tags: Vec::new(),
        created: None,
        updated: None,
        root_commit: None,
    }
}

#[test]
fn parse_list_is_forgiving_over_the_bedrock_array() {
    let json = r#"[
        {"id":"bl-1","title":"T","body":"B","claimant":"me","priority":3,
         "parent":"bl-p","created":10,"updated":20,"root_commit":"abc",
         "tags":["x",7,"y"],
         "blockers":[{"id":"bl-9","on":"claim"},{"id":"bl-3","on":"close"},
                     "x",{"bad":1},{"id":5,"on":"claim"},{"id":"bl-2"},
                     {"id":"z","on":5}]},
        {"title":"no id, dropped"},
        "not-an-object",
        {"id":"bl-2"},
        {"id":"bl-3","claimant":""}
    ]"#;
    let balls = parse_list(json);
    assert_eq!(balls.len(), 3, "id-less and non-object entries dropped");
    let b1 = &balls[0];
    assert_eq!(b1.id, "bl-1");
    assert_eq!(b1.claimant.as_deref(), Some("me"));
    assert_eq!(b1.priority, 3);
    assert_eq!(b1.parent.as_deref(), Some("bl-p"));
    assert_eq!((b1.created, b1.updated), (Some(10), Some(20)));
    assert_eq!(
        b1.tags,
        vec!["x".to_owned(), "y".to_owned()],
        "non-strings skipped"
    );
    assert_eq!(
        b1.blockers,
        vec![
            Blocker {
                id: "bl-9".into(),
                on: "claim".into()
            },
            Blocker {
                id: "bl-3".into(),
                on: "close".into()
            },
        ],
        "malformed blocker entries skipped",
    );
    // Defaults for a bare ball, and an empty-string claimant reads as unclaimed.
    let b2 = &balls[1];
    assert_eq!(
        (b2.title.as_str(), b2.priority, b2.claimant.clone()),
        ("", 0, None)
    );
    assert!(b2.blockers.is_empty() && b2.tags.is_empty() && b2.root_commit.is_none());
    assert_eq!(balls[2].claimant, None, "empty claimant normalizes to None");
}

#[test]
fn parse_rejects_a_non_array_document() {
    assert!(parse_list("{}").is_empty());
    assert!(parse_list("not json").is_empty());
}

#[test]
fn ladder_walks_claimant_then_live_claim_blocker_then_ready() {
    let live: HashSet<&str> = ["bl-1", "bl-5"].into_iter().collect();
    let mut claimed = ball("x");
    claimed.claimant = Some("me".to_owned());
    assert_eq!(ladder(&claimed, &live), Status::Claimed);

    assert_eq!(ladder(&ball("y"), &live), Status::Ready, "no blockers");

    let mut blocked = ball("y");
    blocked.blockers = vec![Blocker {
        id: "bl-1".into(),
        on: "claim".into(),
    }];
    assert_eq!(
        ladder(&blocked, &live),
        Status::Blocked,
        "target still live"
    );

    let mut resolved = ball("y");
    resolved.blockers = vec![Blocker {
        id: "bl-404".into(),
        on: "claim".into(),
    }];
    assert_eq!(
        ladder(&resolved, &live),
        Status::Ready,
        "target closed ⇒ resolved"
    );

    let mut non_claim = ball("y");
    non_claim.blockers = vec![Blocker {
        id: "bl-1".into(),
        on: "close".into(),
    }];
    assert_eq!(
        ladder(&non_claim, &live),
        Status::Ready,
        "close-blocker ignored"
    );
}
