//! The §8.5 search reply's own encoding (§12 line budget, mirroring
//! [`board`](super::board)): one hit, its three possible address shapes
//! flattened onto the keys the gestures already take.

use serde_json::{Map, Value, json};

use crate::search::{Address, Hit};

/// One hit as data: the address it names, spread flat so a consumer reads the
/// same `project`/`id`/`workspace`/`agent` keys the gestures take.
pub(super) fn hit_row(hit: &Hit) -> Value {
    let mut map = Map::new();
    map.insert("at".to_owned(), json!(hit.at.token()));
    match &hit.at {
        Address::Ball { project, id } => {
            map.insert("project".to_owned(), json!(path_text(project)));
            map.insert("id".to_owned(), json!(id));
        }
        Address::Workspace { path } => {
            map.insert("workspace".to_owned(), json!(path_text(path)));
        }
        Address::Conversation { workspace, agent } => {
            map.insert("workspace".to_owned(), json!(path_text(workspace)));
            map.insert("agent".to_owned(), json!(agent));
        }
    }
    map.insert("field".to_owned(), json!(hit.field.token()));
    map.insert("offset".to_owned(), json!(hit.offset));
    map.insert("excerpt".to_owned(), json!(hit.excerpt));
    Value::Object(map)
}

fn path_text(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}
