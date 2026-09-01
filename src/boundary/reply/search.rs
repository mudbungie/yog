//! The §8.5 search reply's own encoding (§12 line budget, mirroring
//! [`board`](super::board)): one hit, its three possible address shapes
//! flattened onto the keys the gestures already take — and, since bl-764a,
//! onto the **values** they take too: the §3.1 workspace leaf and the §5.1 #1
//! project name, never an engine-local path (REMOTE §8.1).

use serde_json::{Map, Value, json};

use crate::search::{Address, Field, Hit};

/// One hit as data: the address it names, spread flat so a consumer reads the
/// same `project`/`id`/`workspace`/`agent` keys — and words — the gestures
/// take, and can post a hit straight back as an address.
pub(super) fn hit_row(hit: &Hit) -> Value {
    let mut map = Map::new();
    map.insert("at".to_owned(), json!(hit.at.token()));
    match &hit.at {
        Address::Ball { project, id } => {
            map.insert("project".to_owned(), json!(project));
            map.insert("id".to_owned(), json!(id));
        }
        Address::Workspace { name } => {
            map.insert("workspace".to_owned(), json!(name));
        }
        Address::Conversation { workspace, agent } => {
            map.insert("workspace".to_owned(), json!(workspace));
            map.insert("agent".to_owned(), json!(agent));
        }
    }
    map.insert("field".to_owned(), json!(hit.field.token()));
    map.insert("offset".to_owned(), json!(hit.offset));
    map.insert("excerpt".to_owned(), json!(hit.excerpt));
    Value::Object(map)
}

/// One hit read back (bl-7067): the `at` token says which address shape the
/// flat keys spell, which is why the token rides at all — the keys alone are
/// ambiguous between a workspace hit and a conversation hit that named none.
pub(super) fn hit_of(v: &Value) -> Result<Hit, String> {
    use crate::boundary::codec::fields::{str_of, usize_of};
    let o = v.as_object().ok_or("hit: not an object")?;
    let at = match str_of(o, "at")?.as_str() {
        "ball" => Address::Ball {
            project: str_of(o, "project")?,
            id: str_of(o, "id")?,
        },
        "workspace" => Address::Workspace {
            name: str_of(o, "workspace")?,
        },
        "conversation" => Address::Conversation {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        other => return Err(format!("hit: unknown address {other:?}")),
    };
    let word = str_of(o, "field")?;
    let field = [Field::Name, Field::Summary, Field::Text]
        .into_iter()
        .find(|f| f.token() == word)
        .ok_or_else(|| format!("hit: unknown field {word:?}"))?;
    Ok(Hit {
        at,
        field,
        offset: usize_of(o, "offset")?,
        excerpt: str_of(o, "excerpt")?,
    })
}
