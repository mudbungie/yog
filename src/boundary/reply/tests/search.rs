//! The §8.5 search reply's own encoding: three address shapes flattened onto
//! the keys — and, since bl-764a, the wire names — the gestures already take,
//! and the unreadable half beside the rows.

use super::*;

#[test]
fn a_search_reply_flattens_each_address_and_names_what_it_could_not_read() {
    use crate::search::{Address, Field, Found, Hit};
    let hit = |at, field| Hit {
        at,
        field,
        offset: 7,
        excerpt: "…tekeli-li…".to_owned(),
    };
    let value = encode(&Reply::Search(Found {
        needle: "tekeli-li".to_owned(),
        hits: vec![
            hit(
                Address::Ball {
                    project: "proj".to_owned(),
                    id: "bl-1f2a".to_owned(),
                },
                Field::Name,
            ),
            hit(
                Address::Workspace {
                    name: "alba".to_owned(),
                },
                Field::Summary,
            ),
            hit(
                Address::Conversation {
                    workspace: "alba".to_owned(),
                    agent: "c-1".to_owned(),
                },
                Field::Text,
            ),
        ],
        unreadable: vec!["gone: balls unlistable".to_owned()],
    }));
    assert_eq!(value["kind"], "search");
    let rows = value["rows"].as_array().expect("rows");
    assert_eq!(rows[0]["at"], "ball");
    assert_eq!(rows[0]["project"], "proj");
    assert_eq!(rows[0]["id"], "bl-1f2a");
    assert_eq!(rows[0]["field"], "name");
    assert_eq!(rows[0]["offset"], 7);
    assert_eq!(rows[0]["excerpt"], "…tekeli-li…");
    assert_eq!(rows[1]["at"], "workspace");
    assert_eq!(rows[1]["workspace"], "alba");
    assert_eq!(rows[1]["field"], "summary");
    assert_eq!(rows[2]["at"], "conversation");
    assert_eq!(rows[2]["workspace"], "alba");
    assert_eq!(rows[2]["agent"], "c-1");
    assert_eq!(rows[2]["field"], "text");
    assert_eq!(value["unreadable"][0], "gone: balls unlistable");
}
